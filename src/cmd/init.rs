use super::Cmd;
use crate::config;
use crate::error::WriteErrorHandler;
use crate::shell::{self, Hook, Opts};

use anyhow::{Context, Result};
use askama::Template;
use clap::{ArgEnum, Clap};

use std::io::{self, Write};

/// Generate shell configuration
#[derive(Clap, Debug)]
pub struct Init {
    #[clap(arg_enum)]
    shell: Shell,

    /// Prevents zoxide from defining any commands
    #[clap(long)]
    no_aliases: bool,

    /// Renames the 'z' command and corresponding aliases
    #[clap(long, default_value = "z")]
    cmd: String,

    /// Chooses event upon which an entry is added to the database
    #[clap(arg_enum, long, default_value = "pwd")]
    hook: Hook,
}

impl Cmd for Init {
    fn run(&self) -> Result<()> {
        let cmd = if self.no_aliases {
            None
        } else {
            Some(self.cmd.as_str())
        };

        let echo = config::zo_echo();
        let resolve_symlinks = config::zo_resolve_symlinks();

        let opts = &Opts {
            cmd,
            hook: self.hook,
            echo,
            resolve_symlinks,
        };

        let source = match self.shell {
            Shell::Bash => shell::Bash(opts).render(),
            Shell::Elvish => shell::Elvish(opts).render(),
            Shell::Fish => shell::Fish(opts).render(),
            Shell::Nushell => shell::Nushell(opts).render(),
            Shell::Posix => shell::Posix(opts).render(),
            Shell::Powershell => shell::Powershell(opts).render(),
            Shell::Xonsh => shell::Xonsh(opts).render(),
            Shell::Zsh => shell::Zsh(opts).render(),
        }
        .context("could not render template")?;
        writeln!(io::stdout(), "{}", source).wrap_write("stdout")
    }
}

#[derive(ArgEnum, Debug)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Posix,
    Powershell,
    Xonsh,
    Zsh,
}
