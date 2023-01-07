#![allow(clippy::single_component_path_imports)]

// rstest_reuse must be imported at the top of the crate.
#[cfg(all(test, feature = "nix-dev"))]
use rstest_reuse;

mod cmd;
mod config;
mod db;
mod error;
mod shell;
mod util;

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

use crate::cmd::{Cmd, Run};
use crate::error::SilentExit;

pub fn main() -> ExitCode {
    // Forcibly disable backtraces.
    env::remove_var("RUST_LIB_BACKTRACE");
    env::remove_var("RUST_BACKTRACE");

    match Cmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => code.into(),
            Err(e) => {
                let _ = writeln!(io::stderr(), "zoxide: {e:?}");
                ExitCode::FAILURE
            }
        },
    }
}
