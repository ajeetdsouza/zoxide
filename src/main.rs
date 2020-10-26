mod cmd;
mod config;
mod error;
mod fzf;
mod import;
mod shell;
mod store;
mod util;

use crate::cmd::{Add, Cmd, Import, Init, Query, Remove};
use crate::error::SilentExit;

use anyhow::Result;
use clap::{AppSettings, Clap};

use std::process;

#[derive(Debug, Clap)]
#[clap(
    about,
    author,
    global_setting(AppSettings::GlobalVersion),
    global_setting(AppSettings::VersionlessSubcommands),
    version = env!("ZOXIDE_VERSION"))]
enum Opts {
    Add(Add),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

pub fn main() -> Result<()> {
    let opts = Opts::parse();

    let result: Result<()> = match opts {
        Opts::Add(cmd) => cmd.run(),
        Opts::Import(cmd) => cmd.run(),
        Opts::Init(cmd) => cmd.run(),
        Opts::Query(cmd) => cmd.run(),
        Opts::Remove(cmd) => cmd.run(),
    };

    result.map_err(|e| match e.downcast::<SilentExit>() {
        Ok(SilentExit { code }) => process::exit(code),
        // TODO: change the error prefix to `zoxide:`
        Err(e) => e,
    })
}
