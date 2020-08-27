mod bash;
mod fish;
mod posix;
mod powershell;
mod zsh;

use anyhow::{Context, Result};
use clap::arg_enum;
use structopt::StructOpt;

use std::io;

/// Generates shell configuration
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Init {
    #[structopt(possible_values = &Shell::variants(), case_insensitive = true)]
    shell: Shell,

    /// Renames the 'z' command and corresponding aliases
    #[structopt(long, alias = "z-cmd", default_value = "z")]
    cmd: String,

    /// Prevents zoxide from defining any commands
    #[structopt(long, alias = "no-define-aliases")]
    no_aliases: bool,

    /// Chooses event on which an entry is added to the database
    #[structopt(
        long,
        possible_values = &Hook::variants(),
        default_value = "pwd",
        case_insensitive = true
    )]
    hook: Hook,
}

impl Init {
    pub fn run(&self) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        match self.shell {
            Shell::bash => bash::run(&mut handle, self),
            Shell::fish => fish::run(&mut handle, self),
            Shell::posix => posix::run(&mut handle, self),
            Shell::powershell => powershell::run(&mut handle, self),
            Shell::zsh => zsh::run(&mut handle, self),
        }
        .context("could not initialize zoxide")
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Shell {
        bash,
        fish,
        posix,
        powershell,
        zsh,
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Hook {
        none,
        prompt,
        pwd,
    }
}
