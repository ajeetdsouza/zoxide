use crate::dir::{Dir, Epoch, Rank};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub use i32 as DBVersion;

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

        let file_path = Self::get_path(&data_dir);

        let buffer = match fs::read(&file_path) {
            Ok(buffer) => buffer,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(Db {
                    dirs: Vec::new(),
                    modified: false,
                    data_dir,
                })
            }
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("could not read from database: {}", file_path.display())
                })
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
            bail!("database is corrupted: {}", file_path.display());
        }

        let (buffer_version, buffer_dirs) = buffer.split_at(version_size);

        let mut deserializer = bincode::config();
        deserializer.limit(Self::MAX_SIZE);

        let version = deserializer.deserialize(buffer_version).with_context(|| {
            format!(
                "could not deserialize database version: {}",
                file_path.display(),
            )
        })?;

        let dirs = match version {
            Self::CURRENT_VERSION => deserializer.deserialize(buffer_dirs).with_context(|| {
                format!("could not deserialize database: {}", file_path.display())
            })?,
            DbVersion(version_num) => bail!(
                "zoxide {} does not support schema v{}: {}",
                env!("ZOXIDE_VERSION"),
                version_num,
                file_path.display(),
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

    pub fn import<P: AsRef<Path>>(&mut self, path: P, merge: bool) -> Result<()> {
        if !self.dirs.is_empty() && !merge {
            bail!(
                "To prevent conflicts, you can only import from z with an empty zoxide database!\n\
                 If you wish to merge the two, specify the `--merge` flag."
            );
        }

        let z_db_file = File::open(path).context("could not open z database file")?;
        let reader = BufReader::new(z_db_file);

        for (idx, read_line) in reader.lines().enumerate() {
            let line_number = idx + 1;
            let line = if let Ok(line) = read_line {
                line
            } else {
                eprintln!(
                    "could not read entry at line {}: {:?}",
                    line_number, read_line
                );
                continue;
            };

            let split_line = line.rsplitn(3, '|').collect::<Vec<_>>();

            match split_line.as_slice() {
                [epoch_str, rank_str, path_str] => {
                    let epoch = match epoch_str.parse::<i64>() {
                        Ok(epoch) => epoch,
                        Err(e) => {
                            eprintln!(
                                "invalid epoch '{}' at line {}: {}",
                                epoch_str, line_number, e
                            );
                            continue;
                        }
                    };
                    let rank = match rank_str.parse::<f64>() {
                        Ok(rank) => rank,
                        Err(e) => {
                            eprintln!("invalid rank '{}' at line {}: {}", rank_str, line_number, e);
                            continue;
                        }
                    };
                    let path_abs = match dunce::canonicalize(path_str) {
                        Ok(path) => path,
                        Err(e) => {
                            eprintln!("invalid path '{}' at line {}: {}", path_str, line_number, e);
                            continue;
                        }
                    };

                    if merge {
                        // If the path exists in the database, add the ranks and set the epoch to
                        // the largest of the parsed epoch and the already present epoch.
                        if let Some(dir) = self.dirs.iter_mut().find(|dir| dir.path == path_abs) {
                            dir.rank += rank;
                            dir.last_accessed = Epoch::max(epoch, dir.last_accessed);

                            continue;
                        };
                    }

                    self.dirs.push(Dir {
                        path: path_abs,
                        rank,
                        last_accessed: epoch,
                    });
                }
                [] | [""] => {} // ignore blank lines
                line => {
                    eprintln!("invalid entry at line {}: {:?}", line_number, line);
                    continue;
                }
            };
        }

        self.modified = true;

        Ok(())
    }

    pub fn add<P: AsRef<Path>>(&mut self, path: P, max_age: Rank, now: Epoch) -> Result<()> {
        let path_abs = dunce::canonicalize(&path)
            .with_context(|| format!("could not access directory: {}", path.as_ref().display()))?;

        match self.dirs.iter_mut().find(|dir| dir.path == path_abs) {
            None => self.dirs.push(Dir {
                path: path_abs,
                last_accessed: now,
                rank: 1.0,
            }),
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1.0;
            }
        };

        let sum_age = self.dirs.iter().map(|dir| dir.rank).sum::<Rank>();

        if sum_age > max_age {
            let factor = 0.9 * max_age / sum_age;
            for dir in &mut self.dirs {
                dir.rank *= factor;
            }

            self.dirs.retain(|dir| dir.rank >= 1.0);
        }

        self.modified = true;
        Ok(())
    }

    pub fn query_many<'a>(&'a mut self, keywords: &'a [String]) -> impl Iterator<Item = &'a Dir> {
        self.query_all()
            .iter()
            .filter(move |dir| dir.is_match(keywords))
    }

    pub fn query_all(&mut self) -> &[Dir] {
        let orig_len = self.dirs.len();
        self.dirs.retain(Dir::is_valid);

        if orig_len != self.dirs.len() {
            self.modified = true;
        }

        self.dirs.as_slice()
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if let Ok(path_abs) = dunce::canonicalize(&path) {
            self.remove_exact(path_abs)
                .or_else(|_| self.remove_exact(path))
        } else {
            self.remove_exact(path)
        }
    }

    pub fn remove_exact<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if let Some(idx) = self.dirs.iter().position(|dir| dir.path == path.as_ref()) {
            self.dirs.swap_remove(idx);
            self.modified = true;
            Ok(())
        } else {
            bail!(
                "could not find path in database: {}",
                path.as_ref().display()
            )
        }
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
