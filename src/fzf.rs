use crate::config;
use crate::error::SilentExit;

use anyhow::{bail, Context, Result};

use std::io;
use std::process::{Child, ChildStdin, Command, Stdio};

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
        if let Some(fzf_opts) = config::zo_fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
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
        // unwrap is safe here because command.stdin() has been piped
        self.child.stdin.as_mut().unwrap()
    }

    pub fn wait_select(self) -> Result<String> {
        let output = self.child.wait_with_output().context("wait failed on fzf")?;

        match output.status.code() {
            // normal exit
            Some(0) => String::from_utf8(output.stdout).context("invalid unicode in fzf output"),

            // no match
            Some(1) => bail!("no match found"),

            // error
            Some(2) => bail!("fzf returned an error"),

            // terminated by a signal
            Some(code @ 130) => bail!(SilentExit { code }),
            Some(128..=254) | None => bail!("fzf was terminated"),

            // unknown
            _ => bail!("fzf returned an unknown error"),
        }
    }
}
