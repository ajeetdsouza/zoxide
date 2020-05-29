use anyhow::{bail, Context, Result};
use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DbVersion(u32);

pub struct Db {
    pub dirs: Vec<Dir>,
    pub modified: bool,
    data_dir: PathBuf,
}

impl Db {
    const CURRENT_VERSION: DbVersion = DbVersion(3);
    const MAX_SIZE: u64 = 8 * 1024 * 1024; // 8 MiB

    pub fn open(data_dir: PathBuf) -> Result<Db> {
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("unable to create data directory: {}", data_dir.display()))?;

        let path = Self::get_path(&data_dir);

        let buffer = match fs::read(&path) {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(Db {
                    dirs: Vec::new(),
                    modified: false,
                    data_dir,
                })
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("could not read from database: {}", path.display()))
            }
        };

        if buffer.is_empty() {
            return Ok(Db {
                dirs: Vec::new(),
                modified: false,
                data_dir,
            });
        }

        let version_size = bincode::serialized_size(&Self::CURRENT_VERSION)
            .context("could not determine size of database version field")?
            as _;

        if buffer.len() < version_size {
            bail!("database is corrupted: {}", path.display());
        }

        let (buffer_version, buffer_dirs) = buffer.split_at(version_size);

        let mut deserializer = bincode::config();
        deserializer.limit(Self::MAX_SIZE);

        let version = deserializer.deserialize(buffer_version).with_context(|| {
            format!("could not deserialize database version: {}", path.display())
        })?;

        let dirs = match version {
            Self::CURRENT_VERSION => deserializer
                .deserialize(buffer_dirs)
                .with_context(|| format!("could not deserialize database: {}", path.display()))?,
            DbVersion(version_num) => bail!(
                "zoxide {} does not support schema v{}: {}",
                env!("ZOXIDE_VERSION"),
                version_num,
                path.display(),
            ),
        };

        Ok(Db {
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
        .context("could not serialize database")?;

        let db_path_tmp = Self::get_path_tmp(&self.data_dir);

        let mut db_file_tmp = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&db_path_tmp)
            .with_context(|| {
                format!(
                    "could not create temporary database: {}",
                    db_path_tmp.display()
                )
            })?;

        // File::set_len() can fail on some filesystems, so we ignore errors
        let _ = db_file_tmp.set_len(buffer_size);

        (|| -> anyhow::Result<()> {
            db_file_tmp.write_all(&buffer).with_context(|| {
                format!(
                    "could not write to temporary database: {}",
                    db_path_tmp.display()
                )
            })?;

            let db_path = Self::get_path(&self.data_dir);

            fs::rename(&db_path_tmp, &db_path)
                .with_context(|| format!("could not create database: {}", db_path.display()))
        })()
        .map_err(|e| {
            fs::remove_file(&db_path_tmp)
                .with_context(|| {
                    format!(
                        "could not remove temporary database: {}",
                        db_path_tmp.display()
                    )
                })
                .err()
                .unwrap_or(e)
        })?;

        self.modified = true;

        Ok(())
    }

    pub fn matches<'a>(&'a mut self, now: Epoch, keywords: &[String]) -> DbMatches<'a> {
        DbMatches::new(self, now, keywords)
    }

    fn get_path<P: AsRef<Path>>(data_dir: P) -> PathBuf {
        data_dir.as_ref().join("db.zo")
    }

    fn get_path_tmp<P: AsRef<Path>>(data_dir: P) -> PathBuf {
        let file_name = format!("db-{}.zo.tmp", Uuid::new_v4());
        data_dir.as_ref().join(file_name)
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            eprintln!("{:#}", e);
        }
    }
}

/// Streaming iterator for matching entries
pub struct DbMatches<'a> {
    db: &'a mut Db,
    idxs: std::iter::Rev<std::ops::Range<usize>>,
    keywords: Vec<String>,
}

impl<'a> DbMatches<'a> {
    pub fn new(db: &'a mut Db, now: Epoch, keywords: &[String]) -> DbMatches<'a> {
        db.dirs
            .sort_unstable_by_key(|dir| FloatOrd(dir.get_frecency(now)));

        let idxs = (0..db.dirs.len()).rev();
        let keywords = keywords
            .iter()
            .map(|keyword| keyword.to_lowercase())
            .collect();

        DbMatches { db, idxs, keywords }
    }

    pub fn next(&mut self) -> Option<&Dir> {
        for idx in &mut self.idxs {
            let dir = &self.db.dirs[idx];

            if !dir.is_match(&self.keywords) {
                continue;
            }

            if !dir.is_valid() {
                self.db.dirs.swap_remove(idx);
                self.db.modified = true;
                continue;
            }

            let dir = &self.db.dirs[idx];
            return Some(dir);
        }

        None
    }
}

pub type Rank = f64;
pub type Epoch = i64; // use a signed integer so subtraction can be performed on it

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir {
    pub path: String,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir {
    pub fn is_valid(&self) -> bool {
        self.rank.is_finite() && self.rank >= 1.0 && Path::new(&self.path).is_dir()
    }

    pub fn is_match(&self, query: &[String]) -> bool {
        let path_lower = self.path.to_lowercase();

        let get_filenames = || {
            let query_name = Path::new(query.last()?).file_name()?.to_str()?;
            let dir_name = Path::new(&path_lower).file_name()?.to_str()?;
            Some((query_name, dir_name))
        };

        if let Some((query_name, dir_name)) = get_filenames() {
            if !dir_name.contains(query_name) {
                return false;
            }
        }

        let mut subpath = path_lower.as_str();

        for subquery in query.iter() {
            match subpath.find(subquery) {
                Some(idx) => subpath = &subpath[idx + subquery.len()..],
                None => return false,
            }
        }

        true
    }

    pub fn get_frecency(&self, now: Epoch) -> Rank {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

        let duration = now - self.last_accessed;
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
