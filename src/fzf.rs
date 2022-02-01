use std::io::{self, Read};
use std::mem;
use std::process::{Child, ChildStdin, Stdio};

use anyhow::{bail, Context, Result};

use crate::error::{FzfNotFound, SilentExit};
use crate::{config, util};

pub struct Fzf {
    child: Child,
}

impl Fzf {
    pub fn new(multiple: bool) -> Result<Self> {
        let bin = if cfg!(windows) { "fzf.exe" } else { "fzf" };
        let mut command = util::get_command(bin).map_err(|_| FzfNotFound)?;
        if multiple {
            command.arg("-m");
        }
        command.arg("-n2..").stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(fzf_opts) = config::fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        } else {
            command.args(&[
                // Search result
                "--no-sort",
                // Interface
                "--keep-right",
                // Layout
                "--height=40%",
                "--info=inline",
                "--layout=reverse",
                // Scripting
                "--exit-0",
                "--select-1",
                // Key/Event bindings
                "--bind=ctrl-z:ignore",
            ]);
            if cfg!(unix) {
                command.arg("--preview=ls -p {2..}");
            }
        }

        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == io::ErrorKind::NotFound => bail!(FzfNotFound),
            Err(e) => Err(e).context("could not launch fzf")?,
        };

        Ok(Fzf { child })
    }

    pub fn stdin(&mut self) -> &mut ChildStdin {
        self.child.stdin.as_mut().unwrap()
    }

    pub fn select(mut self) -> Result<String> {
        // Drop stdin to prevent deadlock.
        mem::drop(self.child.stdin.take());

        let mut stdout = self.child.stdout.take().unwrap();
        let mut output = String::new();
        stdout.read_to_string(&mut output).context("failed to read from fzf")?;

        let status = self.child.wait().context("wait failed on fzf")?;
        match status.code() {
            Some(0) => Ok(output),
            Some(1) => bail!("no match found"),
            Some(2) => bail!("fzf returned an error"),
            Some(code @ 130) => bail!(SilentExit { code }),
            Some(128..=254) | None => bail!("fzf was terminated"),
            _ => bail!("fzf returned an unknown error"),
        }
    }
}
