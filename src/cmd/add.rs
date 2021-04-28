use super::Cmd;
use crate::config;
use crate::db::DatabaseFile;
use crate::util;

use anyhow::{bail, Result};
use clap::Clap;

use std::path::PathBuf;

/// Add a new directory or increment its rank
#[derive(Clap, Debug)]
pub struct Add {
    path: PathBuf,
}

impl Cmd for Add {
    fn run(&self) -> Result<()> {
        let path = if config::zo_resolve_symlinks() {
            util::canonicalize(&self.path)
        } else {
            util::resolve_path(&self.path)
        }?;

        if config::zo_exclude_dirs()?
            .iter()
            .any(|pattern| pattern.matches_path(&path))
        {
            return Ok(());
        }

        if !path.is_dir() {
            bail!("not a directory: {}", path.display());
        }
        let path = util::path_to_str(&path)?;
        let now = util::current_time()?;

        let data_dir = config::zo_data_dir()?;
        let max_age = config::zo_maxage()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;
        db.add(path, now);
        db.age(max_age);

        Ok(())
    }
}
