#![allow(clippy::module_inception)]

use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};

const ENV_HELP: &str = "ENVIRONMENT VARIABLES:
    _ZO_DATA_DIR            Path for zoxide data files
    _ZO_ECHO                Print the matched directory before navigating to it when set to 1
    _ZO_EXCLUDE_DIRS        List of directory globs to be excluded
    _ZO_FZF_OPTS            Custom flags to pass to fzf
    _ZO_MAXAGE              Maximum total age after which entries start getting deleted
    _ZO_RESOLVE_SYMLINKS    Resolve symlinks when storing paths";

#[derive(Debug, Parser)]
#[clap(
    bin_name = env!("CARGO_PKG_NAME"),
    about,
    author,
    after_help = ENV_HELP,
    disable_help_subcommand = true,
    propagate_version = true,
    version = option_env!("ZOXIDE_VERSION").unwrap_or_default()
)]
pub enum Cmd {
    Add(Add),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

/// Add a new directory or increment its rank
#[derive(Debug, Parser)]
pub struct Add {
    #[clap(num_args = 1.., required = true, value_hint = ValueHint::DirPath)]
    pub paths: Vec<PathBuf>,
}

/// Import entries from another application
#[derive(Debug, Parser)]
pub struct Import {
    #[clap(value_hint = ValueHint::FilePath)]
    pub path: PathBuf,

    /// Application to import from
    #[clap(value_enum, long)]
    pub from: ImportFrom,

    /// Merge into existing database
    #[clap(long)]
    pub merge: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ImportFrom {
    Autojump,
    Z,
}

/// Generate shell configuration
#[derive(Debug, Parser)]
pub struct Init {
    #[clap(value_enum)]
    pub shell: InitShell,

    /// Prevents zoxide from defining the `z` and `zi` commands
    #[clap(long, alias = "no-aliases")]
    pub no_cmd: bool,

    /// Changes the prefix of the `z` and `zi` commands
    #[clap(long, default_value = "z")]
    pub cmd: String,

    /// Changes how often zoxide increments a directory's score
    #[clap(value_enum, long, default_value = "pwd")]
    pub hook: InitHook,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitHook {
    None,
    Prompt,
    Pwd,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum InitShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Posix,
    Powershell,
    Xonsh,
    Zsh,
}

/// Search for a directory in the database
#[derive(Debug, Parser)]
pub struct Query {
    pub keywords: Vec<String>,

    /// Show deleted directories
    #[clap(long)]
    pub all: bool,

    /// Use interactive selection
    #[clap(long, short, conflicts_with = "list")]
    pub interactive: bool,

    /// List all matching directories
    #[clap(long, short, conflicts_with = "interactive")]
    pub list: bool,

    /// Print score with results
    #[clap(long, short)]
    pub score: bool,

    /// Exclude a path from results
    #[clap(long, value_hint = ValueHint::DirPath, value_name = "path")]
    pub exclude: Option<String>,
}

/// Remove a directory from the database
#[derive(Debug, Parser)]
pub struct Remove {
    /// Use interactive selection
    #[clap(long, short)]
    pub interactive: bool,
    #[clap(value_hint = ValueHint::DirPath)]
    pub paths: Vec<String>,
}
