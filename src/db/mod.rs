mod dir;
mod stream;

use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{bail, Context, Result};
use bincode::Options;
use ouroboros::self_referencing;

pub use crate::db::dir::{Dir, Epoch, Rank};
pub use crate::db::stream::{Stream, StreamOptions};
use crate::{config, util};

#[self_referencing]
pub struct Database {
    path: PathBuf,
    bytes: Vec<u8>,
    #[borrows(bytes)]
    #[covariant]
    pub dirs: Vec<Dir<'this>>,
    dirty: bool,
}

impl Database {
    const VERSION: u32 = 3;

    pub fn open() -> Result<Self> {
        let data_dir = config::data_dir()?;
        Self::open_dir(data_dir)
    }

    pub fn open_dir(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let path = data_dir.join("db.zo");

        match fs::read(&path) {
            Ok(bytes) => Self::try_new(path, bytes, |bytes| Self::deserialize(bytes), false),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Create data directory, but don't create any file yet. The file will be
                // created later by [`Database::save`] if any data is modified.
                fs::create_dir_all(data_dir).with_context(|| {
                    format!("unable to create data directory: {}", data_dir.display())
                })?;
                Ok(Self::new(path, Vec::new(), |_| Vec::new(), false))
            }
            Err(e) => {
                Err(e).with_context(|| format!("could not read from database: {}", path.display()))
            }
        }
    }

    pub fn save(&mut self) -> Result<()> {
        // Only write to disk if the database is modified.
        if !self.dirty() {
            return Ok(());
        }

        let bytes = Self::serialize(self.dirs())?;
        util::write(self.borrow_path(), bytes).context("could not write to database")?;
        self.with_dirty_mut(|dirty| *dirty = false);

        Ok(())
    }

    /// Increments the rank of a directory, or creates it if it does not exist.
    pub fn add(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => dir.rank = (dir.rank + by).max(0.0),
            None => {
                dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now })
            }
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Creates a new directory. This will create a duplicate entry if this
    /// directory is always in the database, it is expected that the user either
    /// does a check before calling this, or calls `dedup()` afterward.
    pub fn add_unchecked(&mut self, path: impl AsRef<str> + Into<String>, rank: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| {
            dirs.push(Dir { path: path.into().into(), rank, last_accessed: now })
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Increments the rank and updates the last_accessed of a directory, or
    /// creates it if it does not exist.
    pub fn add_update(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => {
                dir.rank = (dir.rank + by).max(0.0);
                dir.last_accessed = now;
            }
            None => {
                dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now })
            }
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Removes the directory with `path` from the store. This does not preserve
    /// ordering, but is O(1).
    pub fn remove(&mut self, path: impl AsRef<str>) -> bool {
        match self.dirs().iter().position(|dir| dir.path == path.as_ref()) {
            Some(idx) => {
                self.swap_remove(idx);
                true
            }
            None => false,
        }
    }

    pub fn swap_remove(&mut self, idx: usize) {
        self.with_dirs_mut(|dirs| dirs.swap_remove(idx));
        self.with_dirty_mut(|dirty| *dirty = true);
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
            dirs.sort_unstable_by(|dir1: &Dir, dir2: &Dir| {
                dir1.score(now).total_cmp(&dir2.score(now))
            })
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn dirty(&self) -> bool {
        *self.borrow_dirty()
    }

    pub fn dirs(&self) -> &[Dir] {
        self.borrow_dirs()
    }

    fn serialize(dirs: &[Dir<'_>]) -> Result<Vec<u8>> {
        (|| -> bincode::Result<_> {
            // Preallocate buffer with combined size of sections.
            let buffer_size =
                bincode::serialized_size(&Self::VERSION)? + bincode::serialized_size(&dirs)?;
            let mut buffer = Vec::with_capacity(buffer_size as usize);

            // Serialize sections into buffer.
            bincode::serialize_into(&mut buffer, &Self::VERSION)?;
            bincode::serialize_into(&mut buffer, &dirs)?;

            Ok(buffer)
        })()
        .context("could not serialize database")
    }

    fn deserialize(bytes: &[u8]) -> Result<Vec<Dir>> {
        // Assume a maximum size for the database. This prevents bincode from throwing
        // strange errors when it encounters invalid data.
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
            Self::VERSION => {
                deserializer.deserialize(bytes_dirs).context("could not deserialize database")?
            }
            version => {
                bail!("unsupported version (got {version}, supports {})", Self::VERSION)
            }
        };

        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let db = Database::open_dir(data_dir.path()).unwrap();
            assert_eq!(db.dirs().len(), 1);

            let dir = &db.dirs()[0];
            assert_eq!(dir.path, path);
            assert!((dir.rank - 2.0).abs() < 0.01);
            assert_eq!(dir.last_accessed, now);
        }
    }

    #[test]
    fn remove() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.remove(path));
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.dirs().is_empty());
            assert!(!db.remove(path));
            db.save().unwrap();
        }
    }
}
