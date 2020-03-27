mod config;
mod db;
mod dir;
mod subcommand;
mod types;
mod util;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "A cd command that learns your habits")]
enum Zoxide {
    Add(subcommand::Add),
    Import(subcommand::Import),
    Init(subcommand::Init),
    Query(subcommand::Query),
    Remove(subcommand::Remove),
}

pub fn main() -> Result<()> {
    let opt = Zoxide::from_args();

    match opt {
        Zoxide::Add(add) => add.run()?,
        Zoxide::Import(import) => import.run()?,
        Zoxide::Init(init) => init.run()?,
        Zoxide::Query(query) => query.run()?,
        Zoxide::Remove(remove) => remove.run()?,
    };

    Ok(())
}
