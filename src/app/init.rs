use super::Run;
use crate::app::{Init, InitShell};
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
            InitShell::Bash => shell::Bash(opts).render(),
            InitShell::Elvish => shell::Elvish(opts).render(),
            InitShell::Fish => shell::Fish(opts).render(),
            InitShell::Nushell => shell::Nushell(opts).render(),
            InitShell::Posix => shell::Posix(opts).render(),
            InitShell::Powershell => shell::Powershell(opts).render(),
            InitShell::Xonsh => shell::Xonsh(opts).render(),
            InitShell::Zsh => shell::Zsh(opts).render(),
        }
        .context("could not render template")?;
        writeln!(io::stdout(), "{}", source).pipe_exit("stdout")
    }
}
