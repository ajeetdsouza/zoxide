#![allow(clippy::module_inception)]

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum, ValueHint};

const HELP_TEMPLATE: &str = color_print::cstr!(
    "\
<bold><underline>{before-help}{name} {version}</underline></bold>
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}

<bold><underline>Environment variables:</underline></bold>
  <bold>_ZO_DATA_DIR</bold>          Path for zoxide data files
  <bold>_ZO_ECHO</bold>              Print the matched directory before navigating to it when set to 1
  <bold>_ZO_EXCLUDE_DIRS</bold>      List of directory globs to be excluded
  <bold>_ZO_FZF_OPTS</bold>          Custom flags to pass to fzf
  <bold>_ZO_MAXAGE</bold>            Maximum total age after which entries start getting deleted
  <bold>_ZO_RESOLVE_SYMLINKS</bold>  Resolve symlinks when storing paths"
);

#[derive(Debug, Parser)]
#[clap(
    about,
    author,
    help_template = HELP_TEMPLATE,
    disable_help_subcommand = true,
    propagate_version = true,
    version = option_env!("ZOXIDE_VERSION").unwrap_or_default()
)]
pub enum Cmd {
    Add(Add),
    Edit(Edit),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

/// Add a new directory or increment its rank
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
pub struct Add {
    #[clap(num_args = 1.., required = true, value_hint = ValueHint::DirPath)]
    pub paths: Vec<PathBuf>,
}

/// Edit the database
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
pub struct Edit {
    #[clap(subcommand)]
    pub cmd: Option<EditCommand>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum EditCommand {
    #[clap(hide = true)]
    Decrement { path: String },
    #[clap(hide = true)]
    Delete { path: String },
    #[clap(hide = true)]
    Increment { path: String },
    #[clap(hide = true)]
    Reload,
}

/// Import entries from another application
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
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
    #[clap(alias = "fasd")]
    Z,
}

/// Generate shell configuration
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
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
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
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

    /// Exclude the current directory
    #[clap(long, value_hint = ValueHint::DirPath, value_name = "path")]
    pub exclude: Option<String>,
}

/// Remove a directory from the database
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HELP_TEMPLATE,
)]
pub struct Remove {
    #[clap(value_hint = ValueHint::DirPath)]
    pub paths: Vec<String>,
}
