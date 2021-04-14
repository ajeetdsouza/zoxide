use super::Cmd;
use crate::config;
use crate::import::{Autojump, Import as _, Z};
use crate::util;

use crate::db::DatabaseFile;
use anyhow::{bail, Result};
use clap::{ArgEnum, Clap};

use std::path::PathBuf;

/// Import entries from another application
#[derive(Clap, Debug)]
pub struct Import {
    path: PathBuf,

    /// Application to import from
    #[clap(arg_enum, long)]
    from: From,

    /// Merge into existing database
    #[clap(long)]
    merge: bool,
}

impl Cmd for Import {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;
        if !self.merge && !db.dirs.is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway")
        }

        let resolve_symlinks = config::zo_resolve_symlinks();
        match self.from {
            From::Autojump => Autojump {
                resolve_symlinks,
                now: util::current_time()?,
            }
            .import(&mut db, &self.path),
            From::Z => Z { resolve_symlinks }.import(&mut db, &self.path),
        }
    }
}

#[derive(ArgEnum, Debug)]
enum From {
    Autojump,
    Z,
}
