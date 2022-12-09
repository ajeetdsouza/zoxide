use std::borrow::Cow;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{bail, Context, Result};
use bincode::Options;
use ouroboros::self_referencing;
use serde::{Deserialize, Serialize};

use crate::{config, util};

#[self_referencing]
pub struct Store {
    path: PathBuf,
    bytes: Vec<u8>,
    #[borrows(bytes)]
    #[covariant]
    dirs: Vec<Dir<'this>>,
    dirty: bool,
}

impl Store {
    const VERSION: u32 = 3;

    pub fn open() -> Result<Self> {
        let data_dir = config::data_dir()?;
        let path = data_dir.join("db.zo");

        match fs::read(&path) {
            Ok(bytes) => Self::try_new(path, bytes, |bytes| Self::deserialize(bytes), false),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Create data directory, but don't create any file yet. The file will be created
                // later by [`Database::save`] if any data is modified.
                fs::create_dir_all(&data_dir)
                    .with_context(|| format!("unable to create data directory: {}", data_dir.display()))?;
                Ok(Self::new(data_dir, Vec::new(), |_| Vec::new(), false))
            }
            Err(e) => Err(e).with_context(|| format!("could not read from database: {}", path.display())),
        }
    }

    pub fn save(&mut self) -> Result<()> {
        // Only write to disk if the database is modified.
        if !self.borrow_dirty() {
            return Ok(());
        }

        let bytes = Self::serialize(self.borrow_dirs())?;
        util::write(self.borrow_path(), &bytes).context("could not write to database")?;
        self.with_dirty_mut(|dirty| *dirty = false);

        Ok(())
    }

    /// Increments the rank of a directory, or creates it if it does not exist.
    pub fn increment(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => dir.rank = (dir.rank + by).max(0.0),
            None => dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now }),
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Increments the rank and updates the last_accessed of a directory, or
    /// creates it if it does not exist.
    pub fn increment_update(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => {
                dir.rank = (dir.rank + by).max(0.0);
                dir.last_accessed = now;
            }
            None => dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now }),
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn remove(&mut self, path: impl AsRef<str>) -> bool {
        let deleted = self.with_dirs_mut(|dirs| match dirs.iter().position(|dir| dir.path == path.as_ref()) {
            Some(idx) => {
                dirs.swap_remove(idx);
                true
            }
            None => false,
        });
        self.with_dirty_mut(|dirty| *dirty |= deleted);
        deleted
    }

    pub fn age(&mut self, max_age: Rank) {
        let mut dirty = false;
        self.with_dirs_mut(|dirs| {
            let total_age = dirs.iter().map(|dir| dir.rank).sum::<Rank>();
            if total_age > max_age {
                let factor = 0.9 * max_age / total_age;
                for idx in (0..dirs.len()).rev() {
                    let dir = &mut dirs[idx];
                    dir.rank *= factor;
                    if dir.rank < 1.0 {
                        dirs.swap_remove(idx);
                    }
                }
                dirty = true;
            }
        });
        self.with_dirty_mut(|dirty_prev| *dirty_prev |= dirty);
    }

    pub fn dedup(&mut self) {
        // Sort by path, so that equal paths are next to each other.
        self.sort_by_path();

        let mut dirty = false;
        self.with_dirs_mut(|dirs| {
            for idx in (1..dirs.len()).rev() {
                // Check if curr_dir and next_dir have equal paths.
                let curr_dir = &dirs[idx];
                let next_dir = &dirs[idx - 1];
                if next_dir.path != curr_dir.path {
                    continue;
                }

                // Merge curr_dir's rank and last_accessed into next_dir.
                let rank = curr_dir.rank;
                let last_accessed = curr_dir.last_accessed;
                let next_dir = &mut dirs[idx - 1];
                next_dir.last_accessed = next_dir.last_accessed.max(last_accessed);
                next_dir.rank += rank;

                // Delete curr_dir.
                dirs.swap_remove(idx);
                dirty = true;
            }
        });
        self.with_dirty_mut(|dirty_prev| *dirty_prev |= dirty);
    }

    pub fn sort_by_path(&mut self) {
        self.with_dirs_mut(|dirs| dirs.sort_unstable_by(|dir1, dir2| dir1.path.cmp(&dir2.path)));
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn sort_by_score(&mut self, now: Epoch) {
        self.with_dirs_mut(|dirs| {
            dirs.sort_unstable_by(|dir1: &Dir, dir2: &Dir| dir1.score(now).total_cmp(&dir2.score(now)))
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn dirs(&self) -> &[Dir] {
        self.borrow_dirs()
    }

    fn serialize(dirs: &[Dir<'_>]) -> Result<Vec<u8>> {
        (|| -> bincode::Result<_> {
            // Preallocate buffer with combined size of sections.
            let buffer_size = bincode::serialized_size(&Self::VERSION)? + bincode::serialized_size(&dirs)?;
            let mut buffer = Vec::with_capacity(buffer_size as usize);

            // Serialize sections into buffer.
            bincode::serialize_into(&mut buffer, &Self::VERSION)?;
            bincode::serialize_into(&mut buffer, &dirs)?;

            Ok(buffer)
        })()
        .context("could not serialize database")
    }

    fn deserialize(bytes: &[u8]) -> Result<Vec<Dir>> {
        // Assume a maximum size for the database. This prevents bincode from throwing strange
        // errors when it encounters invalid data.
        const MAX_SIZE: u64 = 32 << 20; // 32 MiB
        let deserializer = &mut bincode::options().with_fixint_encoding().with_limit(MAX_SIZE);

        // Split bytes into sections.
        let version_size = deserializer.serialized_size(&Self::VERSION).unwrap() as _;
        if bytes.len() < version_size {
            bail!("could not deserialize database: corrupted data");
        }
        let (bytes_version, bytes_dirs) = bytes.split_at(version_size);

        // Deserialize sections.
        let version = deserializer.deserialize(bytes_version)?;
        let dirs = match version {
            Self::VERSION => deserializer.deserialize(bytes_dirs).context("could not deserialize database")?,
            version => {
                bail!("unsupported version (got {version}, supports {})", Self::VERSION)
            }
        };

        Ok(dirs)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dir<'a> {
    #[serde(borrow)]
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir<'_> {
    pub fn score(&self, now: Epoch) -> Rank {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

        // The older the entry, the lesser its importance.
        let duration = now.saturating_sub(self.last_accessed);
        if duration < HOUR {
            self.rank * 4.0
        } else if duration < DAY {
            self.rank * 2.0
        } else if duration < WEEK {
            self.rank * 0.5
        } else {
            self.rank * 0.25
        }
    }
}

pub type Rank = f64;
pub type Epoch = u64;
