mod add;
mod import;
mod init;
mod query;
mod remove;

use anyhow::Result;
use clap::{AppSettings, Clap};

pub use add::Add;
pub use import::Import;
pub use init::Init;
pub use query::Query;
pub use remove::Remove;

const ENV_HELP: &str = "ENVIRONMENT VARIABLES:
    _ZO_DATA_DIR            Path for zoxide data files
    _ZO_ECHO                Prints the matched directory before navigating to it when set to 1
    _ZO_EXCLUDE_DIRS        List of directory globs to be excluded
    _ZO_FZF_OPTS            Custom flags to pass to fzf
    _ZO_MAXAGE              Maximum total age after which entries start getting deleted
    _ZO_RESOLVE_SYMLINKS    Resolve symlinks when storing paths";

pub trait Cmd {
    fn run(&self) -> Result<()>;
}

#[derive(Debug, Clap)]
#[clap(
    about,
    author,
    after_help = ENV_HELP,
    global_setting(AppSettings::ColoredHelp),
    global_setting(AppSettings::DisableHelpSubcommand),
    global_setting(AppSettings::GlobalVersion),
    global_setting(AppSettings::VersionlessSubcommands),
    version = env!("ZOXIDE_VERSION"))]
pub enum App {
    Add(Add),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

impl Cmd for App {
    fn run(&self) -> Result<()> {
        match self {
            App::Add(cmd) => cmd.run(),
            App::Import(cmd) => cmd.run(),
            App::Init(cmd) => cmd.run(),
            App::Query(cmd) => cmd.run(),
            App::Remove(cmd) => cmd.run(),
        }
    }
}
