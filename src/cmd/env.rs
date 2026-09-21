use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::cmd::{Env, EnvCommand, Run};
use crate::config;

impl Run for Env {
    fn run(&self) -> Result<()> {
        let mut stdout = io::stdout().lock();
        match self.cmd {
            Some(cmd) => {
                let value = cmd.value()?;
                if let Some(value) = value {
                    write_value(&mut stdout, &value)?;
                }
                writeln!(stdout)?;
            }
            None => {
                for cmd in EnvCommand::ALL {
                    let name = cmd.name();
                    let value = cmd.value()?;
                    match value {
                        Some(value) => {
                            write!(stdout, "{name}=")?;
                            write_value(&mut stdout, &value)?;
                            writeln!(stdout)?;
                        }
                        None => writeln!(stdout, "{name}=")?,
                    }
                }
            }
        }
        Ok(())
    }
}

impl EnvCommand {
    pub const ALL: &'static [EnvCommand] = &[
        EnvCommand::DataDir,
        EnvCommand::Echo,
        EnvCommand::ExcludeDirs,
        EnvCommand::FzfOpts,
        EnvCommand::Maxage,
        EnvCommand::ResolveSymlinks,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            EnvCommand::DataDir => "_ZO_DATA_DIR",
            EnvCommand::Echo => "_ZO_ECHO",
            EnvCommand::ExcludeDirs => "_ZO_EXCLUDE_DIRS",
            EnvCommand::FzfOpts => "_ZO_FZF_OPTS",
            EnvCommand::Maxage => "_ZO_MAXAGE",
            EnvCommand::ResolveSymlinks => "_ZO_RESOLVE_SYMLINKS",
        }
    }

    /// Returns the resolved value, or `None` if the variable is unset and has
    /// no implicit default.
    pub fn value(&self) -> Result<Option<Box<OsStr>>> {
        match self {
            EnvCommand::DataDir => {
                Ok(Some(config::data_dir()?.into_os_string().into_boxed_os_str()))
            }
            EnvCommand::Echo => Ok(config::echo().then(|| Box::from(OsStr::new("1")))),
            EnvCommand::ExcludeDirs => {
                let value = match env::var_os("_ZO_EXCLUDE_DIRS") {
                    Some(paths) => paths,
                    None => {
                        let patterns = config::exclude_dirs()?;
                        let pattern = patterns
                            .first()
                            .context("could not resolve default value of _ZO_EXCLUDE_DIRS")?;
                        pattern.as_str().into()
                    }
                };
                Ok(Some(value.into_boxed_os_str()))
            }
            EnvCommand::FzfOpts => Ok(config::fzf_opts().map(|opts| opts.into_boxed_os_str())),
            EnvCommand::Maxage => {
                let maxage = config::maxage()?;
                Ok(Some(Box::from(OsStr::new(&maxage.to_string()))))
            }
            EnvCommand::ResolveSymlinks => {
                Ok(config::resolve_symlinks().then(|| Box::from(OsStr::new("1"))))
            }
        }
    }
}

fn write_value(stdout: &mut impl Write, value: &OsStr) -> Result<()> {
    stdout.write_all(value.as_encoded_bytes()).context("could not write to stdout")
}
