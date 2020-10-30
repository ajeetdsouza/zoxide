use super::Cmd;
use crate::config;
use crate::util;

use crate::store::Store;
use anyhow::Result;
use clap::Clap;

use std::path::PathBuf;

/// Add a new directory or increment its rank
#[derive(Clap, Debug)]
pub struct Add {
    path: Option<PathBuf>,
}

impl Cmd for Add {
    fn run(&self) -> Result<()> {
        let path = match &self.path {
            Some(path) => {
                if config::zo_resolve_symlinks() {
                    util::canonicalize(&path)
                } else {
                    util::resolve_path(&path)
                }
            }
            None => util::current_dir(),
        }?;

        if config::zo_exclude_dirs()?
            .iter()
            .any(|pattern| pattern.matches_path(&path))
        {
            return Ok(());
        }

        let path = util::path_to_str(&path)?;
        let now = util::current_time()?;

        let data_dir = config::zo_data_dir()?;
        let max_age = config::zo_maxage()?;

        let mut store = Store::open(&data_dir)?;
        store.add(path, now);
        store.age(max_age);

        Ok(())
    }
}
