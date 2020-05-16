#![forbid(unsafe_code)]

mod config;
mod db;
mod error;
mod fzf;
mod subcommand;
mod util;

use crate::error::SilentExit;

use anyhow::Result;
use structopt::StructOpt;

use std::process;

#[derive(Debug, StructOpt)]
#[structopt(about, version = env!("ZOXIDE_VERSION"))]
enum Zoxide {
    Add(subcommand::Add),
    Import(subcommand::Import),
    Init(subcommand::Init),
    Query(subcommand::Query),
    Remove(subcommand::Remove),
}

pub fn main() -> Result<()> {
    let opt = Zoxide::from_args();

    let res = match opt {
        Zoxide::Add(add) => add.run(),
        Zoxide::Import(import) => import.run(),
        Zoxide::Init(init) => init.run(),
        Zoxide::Query(query) => query.run(),
        Zoxide::Remove(remove) => remove.run(),
    };

    res.map_err(|e| match e.downcast::<SilentExit>() {
        Ok(SilentExit { code }) => process::exit(code),
        Err(e) => e,
    })
}
