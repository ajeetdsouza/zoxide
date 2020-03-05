use crate::dir::Dir;
use crate::error::AppError;
use crate::types::Timestamp;
use failure::ResultExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::io::{Read, Write};
use std::path::Path;
use fs2::FileExt;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DB {
    pub dirs: Vec<Dir>,

    #[serde(skip)]
    pub modified: bool,
}

impl DB {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DB, failure::Error> {
        match File::open(path) {
            Ok(file) => {
                file.lock_shared().with_context(|_| AppError::FileLockError)?;
                let rd = BufReader::new(file);
                let db = DB::read_from(rd).with_context(|_| AppError::DBReadError)?;
                Ok(db)
            }
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Ok(DB::default()),
                _ => Err(err).with_context(|_| AppError::FileOpenError)?,
            },
        }
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), failure::Error> {
        if self.modified {
            let file = File::create(path).with_context(|_| AppError::FileOpenError)?;
            file.lock_exclusive().with_context(|_| AppError::FileLockError)?;
            let wr = BufWriter::new(file);
            self.write_into(wr)
                .with_context(|_| AppError::DBWriteError)?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(rd: R) -> Result<DB, bincode::Error> {
        bincode::deserialize_from(rd)
    }

    pub fn write_into<W: Write>(&self, wr: W) -> Result<(), bincode::Error> {
        bincode::serialize_into(wr, &self)
    }

    pub fn add<P: AsRef<Path>>(&mut self, path: P, now: Timestamp) -> Result<(), failure::Error> {
        let path_abs = path
            .as_ref()
            .canonicalize()
            .with_context(|_| AppError::PathAccessError)?;
        let path_str = path_abs.to_str().ok_or_else(|| AppError::UnicodeError)?;

        match self.dirs.iter_mut().find(|dir| dir.path == path_str) {
            None => self.dirs.push(Dir {
                path: path_str.to_owned(),
                last_accessed: now,
                rank: 1,
            }),
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1;
            }
        };

        self.modified = true;
        Ok(())
    }

    pub fn query(&mut self, keywords: &[String], now: Timestamp) -> Option<Dir> {
        // TODO: expand "~" in queries

        loop {
            let (idx, dir) = self
                .dirs
                .iter()
                .enumerate()
                .filter(|(_, dir)| dir.is_match(keywords))
                .max_by_key(|(_, dir)| dir.get_frecency(now))?;

            if dir.is_dir() {
                return Some(dir.to_owned());
            } else {
                self.dirs.remove(idx);
                self.modified = true;
            }
        }
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<(), failure::Error> {
        let path_abs = path
            .as_ref()
            .canonicalize()
            .with_context(|_| AppError::PathAccessError)?;
        let path_str = path_abs.to_str().ok_or_else(|| AppError::UnicodeError)?;

        if let Some(idx) = self.dirs.iter().position(|dir| dir.path == path_str) {
            self.dirs.remove(idx);
            self.modified = true;
        }

        Ok(())
    }

    pub fn remove_invalid(&mut self) {
        let dirs_len = self.dirs.len();
        self.dirs.retain(|dir| dir.is_dir());

        if self.dirs.len() != dirs_len {
            self.modified = true;
        }
    }
}
