use crate::config::zo_fzf_opts;
use crate::db::{Dir, Epoch};
use crate::error::SilentExit;

use anyhow::{bail, Context, Result};

use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::str;

pub struct Fzf {
    child: Child,
    lines: Vec<String>,
}

impl Fzf {
    pub fn new() -> Result<Self> {
        let mut command = Command::new("fzf");
        command
            .args(&["-n2..", "--no-sort"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        if let Some(fzf_opts) = zo_fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        }

        let child = command.spawn().context("could not launch fzf")?;

        Ok(Fzf {
            child,
            lines: Vec::new(),
        })
    }

    pub fn write_dir(&mut self, dir: &Dir, now: Epoch) {
        let frecency = dir.get_frecency(now);

        let frecency_scaled = if frecency > 9999.0 {
            9999
        } else if frecency > 0.0 {
            frecency as u32
        } else {
            0
        };

        self.lines
            .push(format!("{:>4}        {}", frecency_scaled, dir.path));
    }

    pub fn wait_selection(mut self) -> Result<Option<String>> {
        // unwrap() is safe here since we have captured `stdin`
        let stdin = self.child.stdin.as_mut().unwrap();

        self.lines.sort_unstable();

        for line in self.lines.iter() {
            writeln!(stdin, "{}", line).context("could not write into fzf stdin")?;
        }

        let output = self
            .child
            .wait_with_output()
            .context("wait failed on fzf")?;

        match output.status.code() {
            // normal exit
            Some(0) => {
                let path_bytes = output
                    .stdout
                    .get(12..output.stdout.len().saturating_sub(1))
                    .context("fzf returned invalid output")?;

                let path_str =
                    str::from_utf8(path_bytes).context("invalid utf-8 sequence in fzf output")?;

                Ok(Some(path_str.to_string()))
            }

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
