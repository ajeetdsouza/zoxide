mod _cmd;
mod add;
mod import;
mod init;
mod query;
mod remove;

pub use crate::cmd::_cmd::*;

use anyhow::Result;

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
