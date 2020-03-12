use crate::dir::Dir;
use crate::types::{Rank, Timestamp};
use crate::util;
use anyhow::{anyhow, Context, Result};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub struct DB {
    path: PathBuf,
    path_tmp: PathBuf,

    file_tmp: File,

    dirs: Vec<Dir>,
    modified: bool,
}

impl DB {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DB> {
        let path = path.as_ref().to_path_buf();

        let mut path_tmp = path.clone();
        path_tmp.set_file_name(".zo.tmp");

        let file_tmp = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path_tmp)
            .with_context(|| anyhow!("could not open temporary database"))?;

        file_tmp
            .lock_exclusive()
            .with_context(|| anyhow!("could not lock temporary database"))?;

        let dirs = match File::open(&path) {
            Ok(file) => {
                let rd = BufReader::new(&file);
                bincode::deserialize_from(rd)
                    .with_context(|| anyhow!("could not deserialize database"))?
            }
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Vec::<Dir>::new(),
                _ => return Err(err).with_context(|| anyhow!("could not open database")),
            },
        };

        Ok(DB {
            path,
            path_tmp,
            file_tmp,
            dirs,
            modified: false,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if self.modified {
            self.file_tmp
                .set_len(0)
                .with_context(|| "could not truncate temporary database")?;

            let wr = BufWriter::new(&self.file_tmp);
            bincode::serialize_into(wr, &self.dirs)
                .with_context(|| anyhow!("could not serialize database"))?;
            fs::rename(&self.path_tmp, &self.path)
                .with_context(|| anyhow!("could not move temporary database"))?;
        }

        Ok(())
    }

    pub fn migrate<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if !self.dirs.is_empty() {
            return Err(anyhow!(
                "To prevent conflicts, you can only migrate from z with an empty \
                 zoxide database!"
            ));
        }

        let zdata = File::open(path)?;
        let reader = BufReader::new(zdata);

        for line in reader.lines().filter_map(|l| l.ok()) {
            let first_pipe_idx = line
                .find('|')
                .with_context(|| anyhow!("missing separator"))?;
            let last_pipe_idx = line
                .rfind('|')
                .with_context(|| anyhow!("missing separator"))?;

            if first_pipe_idx == last_pipe_idx {
                return Err(anyhow!("invalid data file format -- only 1 separator"));
            }

            let path = PathBuf::from(&line[..first_pipe_idx]);
            let rank = line[first_pipe_idx + 1..last_pipe_idx]
                .parse::<f64>()
                .with_context(|| anyhow!("could not parse rank"))?;

            // otherwise, the rank will get scaled down, depending on how old
            // the entry is
            let epoch = util::get_current_time()?;

            let path_abs = match path.canonicalize() {
                Ok(path) => path,
                Err(_) => continue, // ignore dead paths
            };

            let path_str = path_abs
                .to_str()
                .ok_or_else(|| anyhow!("invalid unicode in path: {}", path_abs.display()))?;

            self.dirs.push(Dir {
                path: path_str.to_owned(),
                last_accessed: epoch,
                rank,
            });
        }

        self.modified = true;

        Ok(())
    }

    pub fn add<P: AsRef<Path>>(&mut self, path: P, now: Timestamp) -> Result<()> {
        let path_abs = path
            .as_ref()
            .canonicalize()
            .with_context(|| anyhow!("could not access directory: {}", path.as_ref().display()))?;

        let path_str = path_abs
            .to_str()
            .ok_or_else(|| anyhow!("invalid unicode in path: {}", path_abs.display()))?;

        match self.dirs.iter_mut().find(|dir| dir.path == path_str) {
            None => self.dirs.push(Dir {
                path: path_str.to_owned(),
                last_accessed: now,
                rank: 1.0,
            }),
            Some(dir) => {
                dir.last_accessed = now;
                dir.rank += 1.0;
            }
        };

        let max_age = util::get_zo_maxage()?;
        let sum_age = self.dirs.iter().map(|dir| dir.rank).sum::<Rank>();

        if sum_age > max_age {
            let factor = 0.9 * max_age / sum_age;
            for dir in &mut self.dirs {
                dir.rank *= factor;
            }
        }

        self.dirs.retain(|dir| dir.rank >= 1.0);
        self.modified = true;

        Ok(())
    }

    pub fn query(&mut self, keywords: &[String], now: Timestamp) -> Option<Dir> {
        loop {
            let (idx, dir) = self
                .dirs
                .iter()
                .enumerate()
                .filter(|(_, dir)| dir.is_match(keywords))
                .max_by_key(|(_, dir)| dir.get_frecency(now) as i64)?;

            if dir.is_dir() {
                return Some(dir.to_owned());
            } else {
                self.dirs.remove(idx);
                self.modified = true;
            }
        }
    }

    pub fn query_all(&mut self, mut keywords: Vec<String>) -> Vec<Dir> {
        self.remove_invalid();

        for keyword in &mut keywords {
            keyword.make_ascii_lowercase();
        }

        self.dirs
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

        let path_str = path_abs
            .to_str()
            .ok_or_else(|| anyhow!("invalid unicode in path"))?;

        if let Some(idx) = self.dirs.iter().position(|dir| dir.path == path_str) {
            self.dirs.remove(idx);
            self.modified = true;
        }

        Ok(())
    }

    fn remove_invalid(&mut self) {
        let orig_len = self.dirs.len();
        self.dirs.retain(Dir::is_dir);
        if orig_len != self.dirs.len() {
            self.modified = true;
        }
    }
}

impl Drop for DB {
    fn drop(&mut self) {
        if let Err(e) = self.save() {
            eprintln!("{:#}", e);
        }
    }
}
