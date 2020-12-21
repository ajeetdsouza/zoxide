use super::Cmd;
use crate::config;

use crate::store::Store;
use anyhow::{Context, Result};
use clap::Clap;

use std::io::{self, Write};

/// Export entries from database
#[derive(Clap, Debug)]
pub struct Export {}

impl Cmd for Export {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let store = Store::open(&data_dir)?;

        let mut handle = io::stdout();
        for dir in store.dirs.iter() {
            writeln!(handle, "{}|{}|{}", dir.path, dir.rank, dir.last_accessed)
                .context("could not write to stdout")?;
        }

        Ok(())
    }
}
