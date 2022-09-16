mod dir;
mod stream;

use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, Result};
pub use dir::{Dir, DirList, Epoch, Rank};
pub use stream::Stream;

use crate::util;

#[derive(Debug)]
pub struct Database<'file> {
    pub dirs: DirList<'file>,
    pub modified: bool,
    pub data_dir: &'file Path,
}

impl<'file> Database<'file> {
    pub fn save(&mut self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }

        let buffer = self.dirs.to_bytes()?;
        let path = db_path(&self.data_dir);
        util::write(&path, &buffer).context("could not write to database")?;
        self.modified = false;
        Ok(())
    }

    /// Adds a new directory or increments its rank. Also updates its last accessed time.
    pub fn add<S: AsRef<str>>(&mut self, path: S, now: Epoch) {
        let path = path.as_ref();

        match self.dirs.iter_mut().find(|dir| dir.path == path) {
            None => {
                self.dirs.push(Dir { path: path.to_string().into(), last_accessed: now, rank: 1.0 });
            }
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1.0;
            }
        };

        self.modified = true;
    }

    pub fn dedup(&mut self) {
        // Sort by path, so that equal paths are next to each other.
        self.dirs.sort_by(|dir1, dir2| dir1.path.cmp(&dir2.path));

        for idx in (1..self.dirs.len()).rev() {
            // Check if curr_dir and next_dir have equal paths.
            let curr_dir = &self.dirs[idx];
            let next_dir = &self.dirs[idx - 1];
            if next_dir.path != curr_dir.path {
                continue;
            }

            // Merge curr_dir's rank and last_accessed into next_dir.
            let rank = curr_dir.rank;
            let last_accessed = curr_dir.last_accessed;
            let next_dir = &mut self.dirs[idx - 1];
            next_dir.last_accessed = next_dir.last_accessed.max(last_accessed);
            next_dir.rank += rank;

            // Delete curr_dir.
            self.dirs.swap_remove(idx);
            self.modified = true;
        }
    }

    // Streaming iterator for directories.
    pub fn stream(&mut self, now: Epoch) -> Stream<'_, 'file> {
        Stream::new(self, now)
    }

    /// Removes the directory with `path` from the store. This does not preserve ordering, but is
    /// O(1).
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
}

pub struct DatabaseFile {
    buffer: Vec<u8>,
    data_dir: PathBuf,
}

impl DatabaseFile {
    pub fn new<P: Into<PathBuf>>(data_dir: P) -> Self {
        DatabaseFile { buffer: Vec::new(), data_dir: data_dir.into() }
    }

    pub fn open(&mut self) -> Result<Database> {
        // Read the entire database to memory. For smaller files, this is faster than
        // mmap / streaming, and allows for zero-copy deserialization.
        let path = db_path(&self.data_dir);
        match fs::read(&path) {
            Ok(buffer) => {
                self.buffer = buffer;
                let dirs = DirList::from_bytes(&self.buffer)
                    .with_context(|| format!("could not deserialize database: {}", path.display()))?;
                Ok(Database { dirs, modified: false, data_dir: &self.data_dir })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Create data directory, but don't create any file yet. The file will be created
                // later by [`Database::save`] if any data is modified.
                fs::create_dir_all(&self.data_dir)
                    .with_context(|| format!("unable to create data directory: {}", self.data_dir.display()))?;
                Ok(Database { dirs: DirList::new(), modified: false, data_dir: &self.data_dir })
            }
            Err(e) => Err(e).with_context(|| format!("could not read from database: {}", path.display())),
        }
    }
}

fn db_path<P: AsRef<Path>>(data_dir: P) -> PathBuf {
    const DB_FILENAME: &str = "db.zo";
    data_dir.as_ref().join(DB_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        let data_dir = tempfile::tempdir().unwrap();
        {
            let mut db = DatabaseFile::new(data_dir.path());
            let mut db = db.open().unwrap();
            db.add(path, now);
            db.add(path, now);
            db.save().unwrap();
        }
        {
            let mut db = DatabaseFile::new(data_dir.path());
            let db = db.open().unwrap();
            assert_eq!(db.dirs.len(), 1);

            let dir = &db.dirs[0];
            assert_eq!(dir.path, path);
            assert_eq!(dir.last_accessed, now);
        }
    }

    #[test]
    fn remove() {
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        let data_dir = tempfile::tempdir().unwrap();
        {
            let mut db = DatabaseFile::new(data_dir.path());
            let mut db = db.open().unwrap();
            db.add(path, now);
            db.save().unwrap();
        }
        {
            let mut db = DatabaseFile::new(data_dir.path());
            let mut db = db.open().unwrap();
            assert!(db.remove(path));
            db.save().unwrap();
        }
        {
            let mut db = DatabaseFile::new(data_dir.path());
            let mut db = db.open().unwrap();
            assert!(db.dirs.is_empty());
            assert!(!db.remove(path));
            db.save().unwrap();
        }
    }
}
