mod dir;
mod query;

pub use dir::{Dir, DirList, Epoch, Rank};
pub use query::Query;

use anyhow::{Context, Result};
use ordered_float::OrderedFloat;
use tempfile::{NamedTempFile, PersistError};

use std::borrow::Cow;
use std::cmp::Reverse;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Store<'a> {
    pub dirs: DirList<'a>,
    pub modified: bool,
    data_dir: &'a Path,
}

impl<'a> Store<'a> {
    pub fn save(&mut self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }

        let buffer = self.dirs.to_bytes()?;
        let mut file = NamedTempFile::new_in(&self.data_dir).with_context(|| {
            format!(
                "could not create temporary store in: {}",
                self.data_dir.display()
            )
        })?;

        // Preallocate enough space on the file, preventing copying later on.
        // This optimization may fail on some filesystems, but it is safe to
        // ignore it and proceed.
        let _ = file.as_file().set_len(buffer.len() as _);
        file.write_all(&buffer).with_context(|| {
            format!(
                "could not write to temporary store: {}",
                file.path().display()
            )
        })?;

        let path = store_path(&self.data_dir);
        persist(file, &path)
            .with_context(|| format!("could not replace store: {}", path.display()))?;

        self.modified = false;
        Ok(())
    }

    pub fn add<S: AsRef<str>>(&mut self, path: S, now: Epoch) {
        let path = path.as_ref();
        debug_assert!(Path::new(path).is_absolute());

        match self.dirs.iter_mut().find(|dir| dir.path == path) {
            None => self.dirs.push(Dir {
                path: Cow::Owned(path.into()),
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

    pub fn iter_matches<'b>(
        &'b mut self,
        query: &'b Query,
        now: Epoch,
    ) -> impl DoubleEndedIterator<Item = &'b Dir> {
        self.dirs
            .sort_unstable_by_key(|dir| Reverse(OrderedFloat(dir.score(now))));
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
}

impl Drop for Store<'_> {
    fn drop(&mut self) {
        // Since the error can't be properly handled here,
        // pretty-print it instead.
        if let Err(e) = self.save() {
            println!("Error: {}", e)
        }
    }
}

#[cfg(windows)]
fn persist<P: AsRef<Path>>(mut file: NamedTempFile, path: P) -> Result<(), PersistError> {
    use rand::distributions::{Distribution, Uniform};
    use std::thread;
    use std::time::Duration;

    // File renames on Windows are not atomic and sometimes fail with `PermissionDenied`.
    // This is extremely unlikely unless it's running in a loop on multiple threads.
    // Nevertheless, we guard against it by retrying the rename a fixed number of times.
    const MAX_TRIES: usize = 10;
    let mut rng = None;

    for _ in 0..MAX_TRIES {
        match file.persist(&path) {
            Ok(_) => break,
            Err(e) if e.error.kind() == io::ErrorKind::PermissionDenied => {
                let mut rng = rng.get_or_insert_with(rand::thread_rng);
                let between = Uniform::from(50..150);
                let duration = Duration::from_millis(between.sample(&mut rng));
                thread::sleep(duration);
                file = e.file;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

#[cfg(unix)]
fn persist<P: AsRef<Path>>(file: NamedTempFile, path: P) -> Result<(), PersistError> {
    file.persist(&path)?;
    Ok(())
}

pub struct StoreBuilder {
    data_dir: PathBuf,
    buffer: Vec<u8>,
}

impl StoreBuilder {
    pub fn new<P: Into<PathBuf>>(data_dir: P) -> StoreBuilder {
        StoreBuilder {
            data_dir: data_dir.into(),
            buffer: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Result<Store> {
        // Read the entire store to memory. For smaller files, this is faster
        // than mmap / streaming, and allows for zero-copy deserialization.
        let path = store_path(&self.data_dir);
        match fs::read(&path) {
            Ok(buffer) => {
                self.buffer = buffer;
                let dirs = DirList::from_bytes(&self.buffer)
                    .with_context(|| format!("could not deserialize store: {}", path.display()))?;
                Ok(Store {
                    dirs,
                    modified: false,
                    data_dir: &self.data_dir,
                })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Create data directory, but don't create any file yet.
                // The file will be created later by [`Store::save`]
                // if any data is modified.
                fs::create_dir_all(&self.data_dir).with_context(|| {
                    format!(
                        "unable to create data directory: {}",
                        self.data_dir.display()
                    )
                })?;
                Ok(Store {
                    dirs: DirList::new(),
                    modified: false,
                    data_dir: &self.data_dir,
                })
            }
            Err(e) => {
                Err(e).with_context(|| format!("could not read from store: {}", path.display()))
            }
        }
    }
}

fn store_path<P: AsRef<Path>>(data_dir: P) -> PathBuf {
    const STORE_FILENAME: &str = "db.zo";
    data_dir.as_ref().join(STORE_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let path = if cfg!(windows) {
            r"C:\foo\bar"
        } else {
            "/foo/bar"
        };
        let now = 946684800;

        let data_dir = tempfile::tempdir().unwrap();
        {
            let mut store = StoreBuilder::new(data_dir.path());
            let mut store = store.build().unwrap();
            store.add(path, now);
            store.add(path, now);
        }
        {
            let mut store = StoreBuilder::new(data_dir.path());
            let store = store.build().unwrap();
            assert_eq!(store.dirs.len(), 1);

            let dir = &store.dirs[0];
            assert_eq!(dir.path, path);
            assert_eq!(dir.last_accessed, now);
        }
    }

    #[test]
    fn remove() {
        let path = if cfg!(windows) {
            r"C:\foo\bar"
        } else {
            "/foo/bar"
        };
        let now = 946684800;

        let data_dir = tempfile::tempdir().unwrap();
        {
            let mut store = StoreBuilder::new(data_dir.path());
            let mut store = store.build().unwrap();
            store.add(path, now);
        }
        {
            let mut store = StoreBuilder::new(data_dir.path());
            let mut store = store.build().unwrap();
            assert!(store.remove(path));
        }
        {
            let mut store = StoreBuilder::new(data_dir.path());
            let mut store = store.build().unwrap();
            assert!(store.dirs.is_empty());
            assert!(!store.remove(path));
        }
    }
}
