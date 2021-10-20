mod app;
mod config;
mod db;
mod error;
mod fzf;
mod shell;
mod util;

use std::io::{self, Write};
use std::{env, process};

use clap::Parser;

use crate::app::{App, Run};
use crate::error::SilentExit;

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
