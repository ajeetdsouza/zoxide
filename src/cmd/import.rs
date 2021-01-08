use super::Cmd;
use crate::config;
use crate::import::{Autojump, Import as _, Z};
use crate::util;

use crate::store::StoreBuilder;
use anyhow::{bail, Result};
use clap::{ArgEnum, Clap};

use std::path::PathBuf;

/// Import entries from another database
#[derive(Clap, Debug)]
pub struct Import {
    path: PathBuf,

    /// Application to import from
    #[clap(arg_enum, long, default_value = "z")]
    from: From,

    /// Merge into existing database
    #[clap(long)]
    merge: bool,
}

impl Cmd for Import {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;

        let mut store = StoreBuilder::new(data_dir);
        let mut store = store.build()?;
        if !self.merge && !store.dirs.is_empty() {
            bail!("zoxide database is not empty, specify --merge to continue anyway")
        }

        let resolve_symlinks = config::zo_resolve_symlinks();
        match self.from {
            From::Autojump => Autojump {
                resolve_symlinks,
                now: util::current_time()?,
            }
            .import(&mut store, &self.path),
            From::Z => Z { resolve_symlinks }.import(&mut store, &self.path),
        }
    }
}

#[derive(ArgEnum, Debug)]
enum From {
    Autojump,
    Z,
}
