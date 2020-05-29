use crate::util::{canonicalize, get_db, path_to_str};

use anyhow::{bail, Result};
use structopt::StructOpt;

/// Remove a directory
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Remove {
    path: String,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        remove(&self.path)
    }
}

fn remove(path: &str) -> Result<()> {
    let mut db = get_db()?;

    if let Some(idx) = db.dirs.iter().position(|dir| dir.path == path) {
        db.dirs.swap_remove(idx);
        db.modified = true;
        return Ok(());
    }

    let path = canonicalize(&path)?;
    let path = path_to_str(&path)?;

    if let Some(idx) = db.dirs.iter().position(|dir| dir.path == path) {
        db.dirs.swap_remove(idx);
        db.modified = true;
        return Ok(());
    }

    bail!("could not find path in database: {}", path)
}
