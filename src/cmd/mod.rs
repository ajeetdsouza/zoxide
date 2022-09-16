mod add;
mod cmd;
mod import;
mod init;
mod query;
mod remove;

use anyhow::Result;

pub use crate::cmd::cmd::*;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for Cmd {
    fn run(&self) -> Result<()> {
        match self {
            Cmd::Add(cmd) => cmd.run(),
            Cmd::Import(cmd) => cmd.run(),
            Cmd::Init(cmd) => cmd.run(),
            Cmd::Query(cmd) => cmd.run(),
            Cmd::Remove(cmd) => cmd.run(),
        }
    }
}
