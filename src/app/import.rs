use crate::app::{Import, ImportFrom, Run};
use crate::config;
use crate::db::{Database, DatabaseFile, Dir, DirList};

use anyhow::{bail, Context, Result};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

impl Run for Import {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;

        let mut db = DatabaseFile::new(data_dir);
        let db = &mut db.open()?;
        if !self.merge && !db.dirs.is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        match self.from {
            ImportFrom::Autojump => from_autojump(db, &self.path),
            ImportFrom::Z => from_z(db, &self.path),
        }
        .context("import error")
    }
}

fn from_autojump<P: AsRef<Path>>(db: &mut Database, path: P) -> Result<()> {
    let path = path.as_ref();
    let buffer = fs::read_to_string(path)
        .with_context(|| format!("could not open autojump database: {}", path.display()))?;

    let mut dirs = db
        .dirs
        .iter()
        .map(|dir| (dir.path.as_ref(), dir.clone()))
        .collect::<HashMap<_, _>>();

    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.splitn(2, '\t');

        let rank = split
            .next()
            .with_context(|| format!("invalid entry: {}", line))?;
        let mut rank = rank
            .parse::<f64>()
            .with_context(|| format!("invalid rank: {}", rank))?;
        // Normalize the rank using a sigmoid function. Don't import actual
        // ranks from autojump, since its scoring algorithm is very different,
        // and might take a while to get normalized.
        rank = 1.0 / (1.0 + (-rank).exp());

        let path = split
            .next()
            .with_context(|| format!("invalid entry: {}", line))?;

        dirs.entry(path)
            .and_modify(|dir| dir.rank += rank)
            .or_insert_with(|| Dir {
                path: path.to_string().into(),
                rank,
                last_accessed: 0,
            });
    }

    db.dirs = DirList(dirs.into_iter().map(|(_, dir)| dir).collect());
    db.modified = true;

    Ok(())
}

fn from_z<P: AsRef<Path>>(db: &mut Database, path: P) -> Result<()> {
    let path = path.as_ref();
    let buffer = fs::read_to_string(path)
        .with_context(|| format!("could not open z database: {}", path.display()))?;

    let mut dirs = db
        .dirs
        .iter()
        .map(|dir| (dir.path.as_ref(), dir.clone()))
        .collect::<HashMap<_, _>>();

    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.rsplitn(3, '|');

        let last_accessed = split
            .next()
            .with_context(|| format!("invalid entry: {}", line))?;
        let last_accessed = last_accessed
            .parse()
            .with_context(|| format!("invalid epoch: {}", last_accessed))?;

        let rank = split
            .next()
            .with_context(|| format!("invalid entry: {}", line))?;
        let rank = rank
            .parse()
            .with_context(|| format!("invalid rank: {}", rank))?;

        let path = split
            .next()
            .with_context(|| format!("invalid entry: {}", line))?;

        dirs.entry(path)
            .and_modify(|dir| {
                dir.rank += rank;
                if last_accessed > dir.last_accessed {
                    dir.last_accessed = last_accessed;
                }
            })
            .or_insert(Dir {
                path: path.to_string().into(),
                rank,
                last_accessed,
            });
    }

    db.dirs = DirList(dirs.into_iter().map(|(_, dir)| dir).collect());
    db.modified = true;

    Ok(())
}
