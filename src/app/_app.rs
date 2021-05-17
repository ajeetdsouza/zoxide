use clap::{AppSettings, ArgEnum, Clap};

use std::path::PathBuf;

const ENV_HELP: &str = "ENVIRONMENT VARIABLES:
    _ZO_DATA_DIR            Path for zoxide data files
    _ZO_ECHO                Prints the matched directory before navigating to it when set to 1
    _ZO_EXCLUDE_DIRS        List of directory globs to be excluded
    _ZO_FZF_OPTS            Custom flags to pass to fzf
    _ZO_MAXAGE              Maximum total age after which entries start getting deleted
    _ZO_RESOLVE_SYMLINKS    Resolve symlinks when storing paths";

#[derive(Debug, Clap)]
#[clap(
    bin_name = env!("CARGO_PKG_NAME"),
    about,
    author,
    after_help = ENV_HELP,
    global_setting(AppSettings::ColoredHelp),
    global_setting(AppSettings::DisableHelpSubcommand),
    global_setting(AppSettings::GlobalVersion),
    global_setting(AppSettings::VersionlessSubcommands),
    version = option_env!("ZOXIDE_VERSION").unwrap_or_default()
)]
pub enum App {
    Add(Add),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

/// Add a new directory or increment its rank
#[derive(Clap, Debug)]
pub struct Add {
    pub path: PathBuf,
}

/// Import entries from another application
#[derive(Clap, Debug)]
pub struct Import {
    pub path: PathBuf,

    /// Application to import from
    #[clap(arg_enum, long)]
    pub from: ImportFrom,

    /// Merge into existing database
    #[clap(long)]
    pub merge: bool,
}

#[derive(ArgEnum, Debug)]
pub enum ImportFrom {
    Autojump,
    Z,
}

/// Generate shell configuration
#[derive(Clap, Debug)]
pub struct Init {
    #[clap(arg_enum)]
    pub shell: InitShell,

    /// Prevents zoxide from defining any commands
    #[clap(long)]
    pub no_aliases: bool,

    /// Renames the 'z' command and corresponding aliases
    #[clap(long, default_value = "z")]
    pub cmd: String,

    /// Chooses event upon which an entry is added to the database
    #[clap(arg_enum, long, default_value = "pwd")]
    pub hook: InitHook,
}

#[derive(ArgEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitHook {
    None,
    Prompt,
    Pwd,
}

#[derive(ArgEnum, Debug)]
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
#[derive(Clap, Debug)]
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
    #[clap(long, value_name = "path")]
    pub exclude: Option<String>,
}

/// Remove a directory from the database
#[derive(Clap, Debug)]
pub struct Remove {
    // Use interactive selection
    #[clap(conflicts_with = "path", long, short, value_name = "keywords")]
    pub interactive: Option<Vec<String>>,
    #[clap(conflicts_with = "interactive", required_unless_present = "interactive")]
    pub path: Option<String>,
}
