use super::Run;
use crate::app::{Init, Shell};
use crate::config;
use crate::error::WriteErrorHandler;
use crate::shell::{self, Opts};

use anyhow::{Context, Result};
use askama::Template;

use std::io::{self, Write};

impl Run for Init {
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
