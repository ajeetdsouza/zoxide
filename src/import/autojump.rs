use super::Import;

use crate::store::{Dir, Epoch, Store};
use anyhow::{Context, Result};

use std::fs;
use std::path::Path;

pub struct Autojump {
    pub resolve_symlinks: bool,
    pub now: Epoch,
}

impl Import for Autojump {
    fn import<P: AsRef<Path>>(&self, store: &mut Store, path: P) -> Result<()> {
        let path = path.as_ref();
        let buffer = fs::read_to_string(path)
            .with_context(|| format!("could not open autojump database: {}", path.display()))?;

        let mut entries = Vec::new();
        for (idx, line) in buffer.lines().enumerate() {
            (|| -> Result<()> {
                if line.is_empty() {
                    return Ok(());
                }

                let (rank, path) = (|| {
                    let mut split = line.splitn(2, '\t');
                    let rank = split.next()?;
                    let path = split.next()?;
                    Some((rank, path))
                })()
                .with_context(|| format!("invalid entry: {}", line))?;

                let rank = rank
                    .parse::<f64>()
                    .with_context(|| format!("invalid rank: {}", rank))?;

                entries.push((path, rank));
                Ok(())
            })()
            .with_context(|| format!("line {}: error reading from autojump database", idx + 1))?;
        }

        let rank_sum = entries.iter().map(|(_, rank)| rank).sum::<f64>();
        for (path, rank) in entries.iter() {
            if store
                .dirs
                .iter_mut()
                .find(|dir| &dir.path == path)
                .is_none()
            {
                store.dirs.push(Dir {
                    path: path.to_string(),
                    rank: rank / rank_sum,
                    last_accessed: self.now,
                });
                store.modified = true;
            }
        }

        Ok(())
    }
}
