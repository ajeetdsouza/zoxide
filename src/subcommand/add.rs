use crate::config;
use crate::util;

use anyhow::{Context, Result};
use structopt::StructOpt;

use std::env;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(about = "Add a new directory or increment its rank")]
pub struct Add {
    path: Option<PathBuf>,
}

impl Add {
    pub fn run(&self) -> Result<()> {
        let mut db = util::get_db()?;
        let now = util::get_current_time()?;
        let maxage = config::zo_maxage()?;
        let excluded_dirs = config::zo_exclude_dirs();

        let path = match &self.path {
            Some(path) => path.clone(),
            None => env::current_dir().context("unable to fetch current directory")?,
        };

        if excluded_dirs.contains(&path) {
            return Ok(());
        }

        db.add(path, maxage, now)
    }
}
