#[path = "src/cmd/cmd.rs"]
mod cmd;

use std::{env, io};

use clap::CommandFactory;
use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_complete_fig::Fig;
use cmd::Cmd;

fn main() -> io::Result<()> {
    // Since we are generating completions in the package directory, we need to
    // set this so that Cargo doesn't rebuild every time.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=tests/");
    generate_completions()
}

fn generate_completions() -> io::Result<()> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");
    const OUT_DIR: &str = "contrib/completions";
    let cmd = &mut Cmd::command();

    clap_complete::generate_to(Bash, cmd, BIN_NAME, OUT_DIR)?;
    clap_complete::generate_to(Elvish, cmd, BIN_NAME, OUT_DIR)?;
    clap_complete::generate_to(Fig, cmd, BIN_NAME, OUT_DIR)?;
    clap_complete::generate_to(Fish, cmd, BIN_NAME, OUT_DIR)?;
    clap_complete::generate_to(PowerShell, cmd, BIN_NAME, OUT_DIR)?;
    clap_complete::generate_to(Zsh, cmd, BIN_NAME, OUT_DIR)?;

    Ok(())
}
