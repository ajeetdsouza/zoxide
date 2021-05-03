mod add;
mod import;
mod init;
mod query;
mod remove;

use crate::app::App;

use anyhow::Result;

pub trait Run {
    fn run(&self) -> Result<()>;
}

impl Run for App {
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
