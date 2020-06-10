use crate::config::zo_fzf_opts;
use crate::error::SilentExit;

use anyhow::{bail, Context, Result};

use std::io::Write;
use std::process::{Child, Command, Stdio};

pub struct Fzf {
    child: Child,
}

impl Fzf {
    pub fn new() -> Result<Self> {
        let mut command = Command::new("fzf");
        command
            .arg("-n2..")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        if let Some(fzf_opts) = zo_fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        }

        let child = command.spawn().context("could not launch fzf")?;

        Ok(Fzf { child })
    }

    pub fn write(&mut self, line: String) -> Result<()> {
        // unwrap() is safe here since we have captured `stdin`
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", line).context("could not write into fzf stdin")
    }

    pub fn wait_select(self) -> Result<Option<String>> {
        let output = self
            .child
            .wait_with_output()
            .context("wait failed on fzf")?;

        match output.status.code() {
            // normal exit
            Some(0) => String::from_utf8(output.stdout)
                .context("invalid utf-8 sequence in fzf output")
                .map(Some),

            // no match
            Some(1) => Ok(None),

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
