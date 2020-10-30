use super::Import;

use crate::store::{Dir, Store};
use anyhow::{Context, Result};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Z {
    pub resolve_symlinks: bool,
}

impl Import for Z {
    fn import<P: AsRef<Path>>(&self, store: &mut Store, path: P) -> Result<()> {
        let file = File::open(path).context("could not open z database")?;
        let reader = BufReader::new(file);

        for (idx, line) in reader.lines().enumerate() {
            (|| -> Result<()> {
                let line = line?;
                if line.is_empty() {
                    return Ok(());
                }

                let (path, rank, last_accessed) = (|| {
                    let mut split = line.rsplitn(3, '|');
                    let last_accessed = split.next()?;
                    let rank = split.next()?;
                    let path = split.next()?;
                    Some((path, rank, last_accessed))
                })()
                .with_context(|| format!("invalid entry: {}", line))?;

                let rank = rank
                    .parse()
                    .with_context(|| format!("invalid rank: {}", rank))?;

                let last_accessed = last_accessed
                    .parse()
                    .with_context(|| format!("invalid epoch: {}", last_accessed))?;

                match store.dirs.iter_mut().find(|dir| dir.path == path) {
                    Some(dir) => {
                        dir.rank += rank;
                        dir.last_accessed = dir.last_accessed.max(last_accessed);
                    }
                    None => store.dirs.push(Dir {
                        path: path.to_string(),
                        rank,
                        last_accessed,
                    }),
                }
                store.modified = true;

                Ok(())
            })()
            .with_context(|| format!("line {}: error reading from z database", idx + 1))?;
        }

        Ok(())
    }
}
