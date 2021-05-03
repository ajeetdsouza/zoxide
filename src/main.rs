mod app;
mod cmd;
mod config;
mod db;
mod error;
mod fzf;
mod import;
mod shell;
mod util;

use crate::app::App;
use crate::cmd::Run;
use crate::error::SilentExit;

use clap::Clap;

use std::env;
use std::io::{self, Write};
use std::process;

pub fn main() {
    // Forcibly disable backtraces.
    env::remove_var("RUST_LIB_BACKTRACE");
    env::remove_var("RUST_BACKTRACE");

    if let Err(e) = App::parse().run() {
        match e.downcast::<SilentExit>() {
            Ok(SilentExit { code }) => process::exit(code),
            Err(e) => {
                let _ = writeln!(io::stderr(), "zoxide: {:?}", e);
                process::exit(1);
            }
        }
    }
}
