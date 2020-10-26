use super::Import;
use crate::util;

use crate::store::{Dir, Epoch, Store};
use anyhow::{Context, Result};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Autojump {
    pub resolve_symlinks: bool,
    pub now: Epoch,
}

impl Import for Autojump {
    fn import<P: AsRef<Path>>(&self, store: &mut Store, path: P) -> Result<()> {
        let file = File::open(path).context("could not open autojump database")?;
        let reader = BufReader::new(file);

        for (idx, line) in reader.lines().enumerate() {
            (|| -> Result<()> {
                let line = line?;
                if line.is_empty() {
                    return Ok(());
                }

                let split_idx = line
                    .find('\t')
                    .with_context(|| format!("invalid entry: {}", line))?;
                let (rank, path) = line.split_at(split_idx);

                let rank = rank
                    .parse()
                    .with_context(|| format!("invalid rank: {}", rank))?;

                let path = if self.resolve_symlinks {
                    util::canonicalize
                } else {
                    util::resolve_path
                }(&path)?;
                let path = util::path_to_str(&path)?;

                if store.dirs.iter_mut().find(|dir| dir.path == path).is_none() {
                    store.dirs.push(Dir {
                        path: path.into(),
                        rank,
                        last_accessed: self.now,
                    });
                    store.modified = true;
                }

                Ok(())
            })()
            .with_context(|| format!("line {}: error reading from z database", idx + 1))?;
        }

        Ok(())
    }
}
