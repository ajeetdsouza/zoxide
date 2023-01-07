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
        let path = db_path(self.data_dir);
        util::write(path, buffer).context("could not write to database")?;
        self.modified = false;
        Ok(())
    }

    // Streaming iterator for directories.
    pub fn stream(&mut self, now: Epoch) -> Stream<'_, 'file> {
        Stream::new(self, now)
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
