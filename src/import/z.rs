use super::Import;

use crate::db::{Database, Dir};
use anyhow::{Context, Result};

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Z {
    pub resolve_symlinks: bool,
}

impl Import for Z {
    fn import<P: AsRef<Path>>(&self, db: &mut Database, path: P) -> Result<()> {
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

                match db.dirs.iter_mut().find(|dir| dir.path == path) {
                    Some(dir) => {
                        dir.rank += rank;
                        dir.last_accessed = dir.last_accessed.max(last_accessed);
                    }
                    None => db.dirs.push(Dir {
                        path: Cow::Owned(path.into()),
                        rank,
                        last_accessed,
                    }),
                }
                db.modified = true;

                Ok(())
            })()
            .with_context(|| format!("line {}: error reading from z database", idx + 1))?;
        }

        Ok(())
    }
}
