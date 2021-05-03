mod add;
mod import;
mod init;
mod query;
mod remove;

use crate::app::Cli;

use anyhow::Result;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for Cli {
    fn run(&self) -> Result<()> {
        match self {
            Cli::Add(cmd) => cmd.run(),
            Cli::Import(cmd) => cmd.run(),
            Cli::Init(cmd) => cmd.run(),
            Cli::Query(cmd) => cmd.run(),
            Cli::Remove(cmd) => cmd.run(),
        }
    }
}
