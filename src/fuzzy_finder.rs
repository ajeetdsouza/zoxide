use crate::config;
use crate::error::SilentExit;

use anyhow::{bail, Context, Result};

use std::process::{Child, ChildStdin, Command, Stdio};

pub struct FuzzyFinder {
    child: Child,
    fuzzy_finder: &'static str
}

impl FuzzyFinder {
    pub fn new() -> Result<Self> {
        let fuzzy_finder = config::zo_fuzzy_finder_cmd();
        let mut command = Command::new(fuzzy_finder);
        command
            .arg("-n2..")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        // `skim` does not read default options through an environnement
        // variable, so this simply will have no effects.
        if let Some(fzf_opts) = config::zo_fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        }

        Ok(Self {
            child: command.spawn().with_context(|| format!("could not launch {}", fuzzy_finder))?,
            fuzzy_finder,
        })
    }

    pub fn cmd_name(&self) -> &'static str {
        self.fuzzy_finder
    }

    pub fn stdin(&mut self) -> &mut ChildStdin {
        self.child.stdin.as_mut().unwrap()
    }

    pub fn wait_select(self) -> Result<String> {
        let fuzzy_finder = self.fuzzy_finder;
        let output = self
            .child
            .wait_with_output()
            .with_context(|| format!("wait failed on {}", fuzzy_finder))?;

        match output.status.code() {
            // normal exit
            Some(0) => String::from_utf8(output.stdout).with_context(|| format!("invalid unicode in {} output", fuzzy_finder)),

            // no match
            Some(1) => bail!("no match found"),

            // error
            Some(2) => bail!("{} returned an error", fuzzy_finder),

            // terminated by a signal
            Some(code @ 130) => bail!(SilentExit { code }),
            Some(128..=254) | None => bail!("{} was terminated", fuzzy_finder),

            // unknown
            _ => bail!("{} returned an unknown error", fuzzy_finder),
        }
    }
}
