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

pub trait Cmd {
    fn run(&self) -> Result<()>;
}

#[derive(Debug, Clap)]
#[clap(about, author, global_setting(AppSettings::GlobalVersion), global_setting(AppSettings::VersionlessSubcommands), version = env!("ZOXIDE_VERSION"))]
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
