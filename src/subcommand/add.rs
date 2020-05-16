use crate::config;
use crate::db::{Dir, Rank};
use crate::util::{get_current_time, get_db, path_to_str};

use anyhow::{Context, Result};
use structopt::StructOpt;

use std::env;

#[derive(Debug, StructOpt)]
#[structopt(about = "Add a new directory or increment its rank")]
pub struct Add {
    path: Option<String>,
}

impl Add {
    pub fn run(&self) -> Result<()> {
        let current_dir;
        let path = match &self.path {
            Some(path) => path,
            None => {
                current_dir = env::current_dir().context("unable to fetch current directory")?;
                path_to_str(&current_dir)?
            }
        };

        add(path)
    }
}

fn add(path: &str) -> Result<()> {
    let path_abs = dunce::canonicalize(path)
        .with_context(|| format!("could not resolve directory: {}", path))?;

    let exclude_dirs = config::zo_exclude_dirs();
    if exclude_dirs
        .iter()
        .any(|excluded_path| excluded_path == &path_abs)
    {
        return Ok(());
    }

    let path_abs_str = path_to_str(&path_abs)?;

    let mut db = get_db()?;
    let now = get_current_time()?;

    let maxage = config::zo_maxage()?;

    match db.dirs.iter_mut().find(|dir| dir.path == path_abs_str) {
        None => db.dirs.push(Dir {
            path: path_abs_str.to_string(),
            last_accessed: now,
            rank: 1.0,
        }),
        Some(dir) => {
            dir.last_accessed = now;
            dir.rank += 1.0;
        }
    };

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

    db.modified = true;

    Ok(())
}
