use std::io::{self, Read};
use std::mem;
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::config;
use crate::error::SilentExit;

pub struct Fzf {
    child: Child,
}

impl Fzf {
    pub fn new(multiple: bool) -> Result<Self> {
        let mut command = Command::new("fzf");
        if multiple {
            command.arg("-m");
        }
        command.arg("-n2..").stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(fzf_opts) = config::fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        } else {
            command.args(&[
                "--bind=ctrl-z:ignore",
                "--exit-0",
                "--height=40%",
                "--inline-info",
                "--no-sort",
                "--reverse",
                "--select-1",
            ]);
            if cfg!(unix) {
                command.arg("--preview=ls -p {2}");
            }
        }

        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                bail!("could not find fzf, is it installed?")
            }
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
