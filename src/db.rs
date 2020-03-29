use crate::config;
use crate::dir::{Dir, Epoch, Rank};

use anyhow::{anyhow, bail, Context, Result};
use indoc::indoc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub use i32 as DBVersion;

pub struct DB {
    data: DBData,
    modified: bool,
    path: PathBuf,
    path_old: Option<PathBuf>,
}

impl DB {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DB> {
        let data = match File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(&file);
                bincode::config()
                    .limit(config::DB_MAX_SIZE)
                    .deserialize_from(reader)
                    .context("could not deserialize database")?
            }
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => DBData::default(),
                _ => return Err(err).context("could not open database file"),
            },
        };

        if data.version != config::DB_VERSION {
            bail!("this database version ({}) is unsupported", data.version);
        }

        Ok(DB {
            data,
            modified: false,
            path: path.as_ref().to_path_buf(),
            path_old: None,
        })
    }

    pub fn open_old<P1, P2>(path_old: P1, path: P2) -> Result<DB>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let file = File::open(&path_old).context("could not open old database file")?;
        let reader = BufReader::new(&file);

        let dirs = bincode::config()
            .limit(config::DB_MAX_SIZE)
            .deserialize_from(reader)
            .context("could not deserialize old database")?;

        let data = DBData {
            version: config::DB_VERSION,
            dirs,
        };

        Ok(DB {
            data,
            modified: true,
            path: path.as_ref().to_path_buf(),
            path_old: Some(path_old.as_ref().to_path_buf()),
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if self.modified {
            let path_tmp = self.get_path_tmp();

            let file_tmp =
                File::create(&path_tmp).context("could not open temporary database file")?;

            let writer = BufWriter::new(&file_tmp);
            bincode::serialize_into(writer, &self.data).context("could not serialize database")?;

            if let Err(e) = fs::rename(&path_tmp, &self.path) {
                fs::remove_file(&path_tmp)
                    .context("could not move or delete temporary database file")?;
                return Err(e).context("could not move temporary database file");
            }
        }

        Ok(())
    }

    pub fn import<P: AsRef<Path>>(&mut self, path: P, merge: bool) -> Result<()> {
        if !self.data.dirs.is_empty() && !merge {
            bail!(indoc!(
                "To prevent conflicts, you can only import from z with an empty zoxide database!
                If you wish to merge the two, specify the `--merge` flag."
            ));
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

            let split_line = line.rsplitn(3, '|').collect::<Vec<&str>>();

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
                    let path_abs = match Path::new(path_str).canonicalize() {
                        Ok(path) => path,
                        Err(e) => {
                            eprintln!("invalid path '{}' at line {}: {}", path_str, line_number, e);
                            continue;
                        }
                    };

                    if merge {
                        // If the path exists in the database, add the ranks and set the epoch to
                        // the largest of the parsed epoch and the already present epoch.
                        if let Some(dir) =
                            self.data.dirs.iter_mut().find(|dir| dir.path == path_abs)
                        {
                            dir.rank += rank;
                            dir.last_accessed = Epoch::max(epoch, dir.last_accessed);

                            continue;
                        };
                    }

                    self.data.dirs.push(Dir {
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
        let path_abs = path
            .as_ref()
            .canonicalize()
            .with_context(|| anyhow!("could not access directory: {}", path.as_ref().display()))?;

        match self.data.dirs.iter_mut().find(|dir| dir.path == path_abs) {
            None => self.data.dirs.push(Dir {
                path: path_abs,
                last_accessed: now,
                rank: 1.0,
            }),
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1.0;
            }
        };

        let sum_age = self.data.dirs.iter().map(|dir| dir.rank).sum::<Rank>();

        if sum_age > max_age {
            let factor = 0.9 * max_age / sum_age;
            for dir in &mut self.data.dirs {
                dir.rank *= factor;
            }

            self.data.dirs.retain(|dir| dir.rank >= 1.0);
        }

        self.modified = true;
        Ok(())
    }

    pub fn query(&mut self, keywords: &[String], now: Epoch) -> Option<Dir> {
        let (idx, dir, _) = self
            .data
            .dirs
            .iter()
            .enumerate()
            .filter(|(_, dir)| dir.is_match(&keywords))
            .map(|(idx, dir)| (idx, dir, dir.get_frecency(now)))
            .max_by(|(_, _, frecency1), (_, _, frecency2)| {
                frecency1.partial_cmp(frecency2).unwrap_or(Ordering::Equal)
            })?;

        if dir.is_dir() {
            Some(dir.to_owned())
        } else {
            self.data.dirs.swap_remove(idx);
            self.modified = true;
            self.query(keywords, now)
        }
    }

    pub fn query_all(&mut self, keywords: &[String]) -> Vec<Dir> {
        let orig_len = self.data.dirs.len();
        self.data.dirs.retain(Dir::is_dir);

        if orig_len != self.data.dirs.len() {
            self.modified = true;
        }

        self.data
            .dirs
            .iter()
            .filter(|dir| dir.is_match(&keywords))
            .cloned()
            .collect()
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_abs = match path.as_ref().canonicalize() {
            Ok(path_abs) => path_abs,
            Err(_) => path.as_ref().to_path_buf(),
        };

        if let Some(idx) = self.data.dirs.iter().position(|dir| dir.path == path_abs) {
            self.data.dirs.swap_remove(idx);
            self.modified = true;
        }

        Ok(())
    }

    fn get_path_tmp(&self) -> PathBuf {
        let file_name = format!(".{}.zo", Uuid::new_v4());

        let mut path_tmp = self.path.clone();
        path_tmp.set_file_name(file_name);

        path_tmp
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            eprintln!("{:#}", e);
        } else if let Some(path_old) = &self.path_old {
            if let Err(e) = fs::remove_file(path_old).context("could not remove old database") {
                eprintln!("{:#}", e);
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DBData {
    version: DBVersion,
    dirs: Vec<Dir>,
}

impl Default for DBData {
    fn default() -> DBData {
        DBData {
            version: config::DB_VERSION,
            dirs: Vec::new(),
        }
    }
}
