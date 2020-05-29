use crate::config;
use crate::db::{Db, Dir, Rank};
use crate::util::{canonicalize, get_current_time, get_db, path_to_str};

use anyhow::{Context, Result};
use structopt::StructOpt;

use std::env;
use std::path::{Path, PathBuf};

/// Add a new directory or increment its rank
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Add {
    path: Option<PathBuf>,
}

impl Add {
    pub fn run(&self) -> Result<()> {
        let current_dir;
        let path = match &self.path {
            Some(path) => path,
            None => {
                current_dir = env::current_dir().context("unable to fetch current directory")?;
                &current_dir
            }
        };

        add(&path)
    }
}

fn add<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let path = canonicalize(&path)?;

    if config::zo_exclude_dirs().contains(&path) {
        return Ok(());
    }

    let mut db = get_db()?;
    let now = get_current_time()?;
    let path = path_to_str(&path)?;
    let maxage = config::zo_maxage()?;

    match db.dirs.iter_mut().find(|dir| dir.path == path) {
        None => db.dirs.push(Dir {
            path: path.to_string(),
            last_accessed: now,
            rank: 1.0,
        }),
        Some(dir) => {
            dir.last_accessed = now;
            dir.rank += 1.0;
        }
    };

    age(&mut db, maxage);
    db.modified = true;

    Ok(())
}

fn age(db: &mut Db, maxage: Rank) {
    let sum_age = db.dirs.iter().map(|dir| dir.rank).sum::<Rank>();

    if sum_age > maxage {
        let factor = 0.9 * maxage / sum_age;
        for dir in &mut db.dirs {
            dir.rank *= factor;
        }

        for idx in (0..db.dirs.len()).rev() {
            let dir = &db.dirs[idx];
            if dir.rank < 1.0 {
                db.dirs.swap_remove(idx);
            }
        }
    }
}
