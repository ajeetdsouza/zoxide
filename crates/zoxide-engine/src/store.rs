use crate::dir::{Dir, Epoch, Rank};
use crate::query::Query;

use anyhow::{bail, Context, Result};
use bincode::Options;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use std::cmp::Reverse;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Store {
    pub dirs: Vec<Dir>,
    pub modified: bool,
    data_dir: PathBuf,
}

impl Store {
    pub const CURRENT_VERSION: StoreVersion = StoreVersion(3);
    const MAX_SIZE: u64 = 8 * 1024 * 1024; // 8 MiB

    pub fn open<P: Into<PathBuf>>(data_dir: P) -> Result<Store> {
        let data_dir = data_dir.into();
        let path = Self::get_path(&data_dir);

        let buffer = match fs::read(&path) {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(&data_dir).with_context(|| {
                    format!("unable to create data directory: {}", path.display())
                })?;
                return Ok(Store {
                    dirs: Vec::new(),
                    modified: false,
                    data_dir,
                });
            }
            Err(e) => {
                Err(e).with_context(|| format!("could not read from store: {}", path.display()))?
            }
        };

        let deserializer = &mut bincode::options()
            .with_fixint_encoding()
            .with_limit(Self::MAX_SIZE);

        let version_size = deserializer
            .serialized_size(&Self::CURRENT_VERSION)
            .unwrap() as _;

        if buffer.len() < version_size {
            bail!("data store may be corrupted: {}", path.display());
        }

        let (buffer_version, buffer_dirs) = buffer.split_at(version_size);

        let version = deserializer
            .deserialize(buffer_version)
            .with_context(|| format!("could not deserialize store version: {}", path.display()))?;

        let dirs = match version {
            Self::CURRENT_VERSION => deserializer
                .deserialize(buffer_dirs)
                .with_context(|| format!("could not deserialize store: {}", path.display()))?,
            version => bail!(
                "unsupported store version, got={}, supported={}: {}",
                version.0,
                Self::CURRENT_VERSION.0,
                path.display()
            ),
        };

        Ok(Store {
            dirs,
            modified: false,
            data_dir,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }

        let (buffer, buffer_size) = (|| -> bincode::Result<_> {
            let version_size = bincode::serialized_size(&Self::CURRENT_VERSION)?;
            let dirs_size = bincode::serialized_size(&self.dirs)?;

            let buffer_size = version_size + dirs_size;
            let mut buffer = Vec::with_capacity(buffer_size as _);

            bincode::serialize_into(&mut buffer, &Self::CURRENT_VERSION)?;
            bincode::serialize_into(&mut buffer, &self.dirs)?;

            Ok((buffer, buffer_size))
        })()
        .context("could not serialize store")?;

        let mut file = NamedTempFile::new_in(&self.data_dir).unwrap();
        let _ = file.as_file().set_len(buffer_size);
        file.write_all(&buffer).unwrap();
        file.persist(Self::get_path(&self.data_dir)).unwrap();

        self.modified = false;
        Ok(())
    }

    pub fn add<S: AsRef<str>>(&mut self, path: S, now: Epoch) {
        let path = path.as_ref();
        debug_assert!(Path::new(path).is_absolute());

        match self.dirs.iter_mut().find(|dir| dir.path == path) {
            None => self.dirs.push(Dir {
                path: path.into(),
                last_accessed: now,
                rank: 1.0,
            }),
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1.0;
            }
        };

        self.modified = true;
    }

    pub fn iter_matches<'a>(
        &'a mut self,
        query: &'a Query,
        now: Epoch,
    ) -> impl DoubleEndedIterator<Item = &'a Dir> {
        self.dirs
            .sort_unstable_by_key(|dir| Reverse(OrderedFloat(dir.get_score(now))));
        self.dirs.iter().filter(move |dir| dir.is_match(&query))
    }

    pub fn remove<S: AsRef<str>>(&mut self, path: S) -> bool {
        let path = path.as_ref();

        if let Some(idx) = self.dirs.iter().position(|dir| dir.path == path) {
            self.dirs.swap_remove(idx);
            self.modified = true;
            return true;
        }

        false
    }

    pub fn age(&mut self, max_age: Rank) {
        let sum_age = self.dirs.iter().map(|dir| dir.rank).sum::<Rank>();

        if sum_age > max_age {
            let factor = 0.9 * max_age / sum_age;

            for idx in (0..self.dirs.len()).rev() {
                let dir = &mut self.dirs[idx];

                dir.rank *= factor;
                if dir.rank < 1.0 {
                    self.dirs.swap_remove(idx);
                }
            }

            self.modified = true;
        }
    }

    fn get_path<P: AsRef<Path>>(data_dir: P) -> PathBuf {
        data_dir.as_ref().join("db.zo")
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            println!("Error: {}", e)
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StoreVersion(pub u32);
