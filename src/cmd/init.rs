use super::Cmd;
use crate::config;
use crate::error::WriteErrorHandler;
use crate::shell::{self, Hook, Opts};

use anyhow::{Context, Result};
use askama::Template;
use clap::{ArgEnum, Clap};
use once_cell::sync::OnceCell;

use std::io::{self, Write};

/// Generates shell configuration
#[derive(Clap, Debug)]
#[clap(after_help(env_help()))]
pub struct Init {
    #[clap(arg_enum)]
    shell: Shell,

    /// Prevents zoxide from defining any commands
    #[clap(long)]
    no_aliases: bool,

    /// Renames the 'z' command and corresponding aliases
    #[clap(long, default_value = "z")]
    cmd: String,

    /// Chooses event upon which an entry is added to the database
    #[clap(arg_enum, long, default_value = "pwd")]
    hook: Hook,
}

impl Cmd for Init {
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
            Shell::Fish => shell::Fish(opts).render(),
            Shell::Posix => shell::Posix(opts).render(),
            Shell::Powershell => shell::PowerShell(opts).render(),
            Shell::Xonsh => shell::Xonsh(opts).render(),
            Shell::Zsh => shell::Zsh(opts).render(),
        }
        .context("could not render template")?;
        writeln!(io::stdout(), "{}", source).handle_err("stdout")
    }
}

#[derive(ArgEnum, Debug)]
enum Shell {
    Bash,
    Fish,
    Posix,
    Powershell,
    Xonsh,
    Zsh,
}

fn env_help() -> &'static str {
    static ENV_HELP: OnceCell<String> = OnceCell::new();
    ENV_HELP.get_or_init(|| {
        #[cfg(unix)]
        const PATH_SPLIT_SEPARATOR: u8 = b':';
        #[cfg(windows)]
        const PATH_SPLIT_SEPARATOR: u8 = b';';

        format!(
            "\
ENVIRONMENT VARIABLES:
    _ZO_DATA_DIR            Path for zoxide data files
                            [current: {data_dir}]
    _ZO_ECHO                Prints the matched directory before navigating to it when set to 1
    _ZO_EXCLUDE_DIRS        List of directory globs to be excluded, separated by '{path_split_separator}'
    _ZO_FZF_OPTS            Custom flags to pass to fzf
    _ZO_MAXAGE              Maximum total age after which entries start getting deleted
    _ZO_RESOLVE_SYMLINKS    Resolve symlinks when storing paths",
            data_dir=config::zo_data_dir().unwrap_or_else(|_| "none".into()).display(),
            path_split_separator=PATH_SPLIT_SEPARATOR as char)
    })
}
