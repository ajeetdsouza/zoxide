use std::io::{self, Write};

use anyhow::{Context, Result};
use rinja::Template;

use crate::cmd::{Init, InitShell, Run};
use crate::config;
use crate::error::BrokenPipeHandler;
use crate::shell::{Bash, Elvish, Fish, Ksh, Nushell, Opts, Posix, Powershell, Xonsh, Zsh};

impl Run for Init {
    fn run(&self) -> Result<()> {
        let cmd = if self.no_cmd { None } else { Some(self.cmd.as_str()) };
        let echo = config::echo();
        let resolve_symlinks = config::resolve_symlinks();
        let opts = &Opts { cmd, hook: self.hook, echo, resolve_symlinks };

        let source = match self.shell {
            InitShell::Bash => Bash(opts).render(),
            InitShell::Elvish => Elvish(opts).render(),
            InitShell::Fish => Fish(opts).render(),
            InitShell::Ksh => Ksh(opts).render(),
            InitShell::Nushell => Nushell(opts).render(),
            InitShell::Posix => Posix(opts).render(),
            InitShell::Powershell => Powershell(opts).render(),
            InitShell::Xonsh => Xonsh(opts).render(),
            InitShell::Zsh => Zsh(opts).render(),
        }
        .context("could not render template")?;
        writeln!(io::stdout(), "{source}").pipe_exit("stdout")
    }
}
