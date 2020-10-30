mod cmd;
mod config;
mod error;
mod fzf;
mod import;
mod shell;
mod store;
mod util;

use crate::cmd::{App, Cmd};
use crate::error::SilentExit;

use anyhow::Result;
use clap::Clap;

use std::process;

pub fn main() -> Result<()> {
    App::parse()
        .run()
        .map_err(|e| match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => process::exit(code),
            // TODO: change the error prefix to `zoxide:`
            Err(e) => e,
        })
}
