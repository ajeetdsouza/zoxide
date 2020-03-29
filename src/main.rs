mod db;
mod dir;
mod env;
mod subcommand;
mod types;
mod util;

use crate::env::Env;
use anyhow::{Context, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "A cd command that learns your habits")]
enum Zoxide {
    Add(subcommand::Add),
    Init(subcommand::Init),
    Migrate(subcommand::Migrate),
    Query(subcommand::Query),
    Remove(subcommand::Remove),
}

pub fn main() -> Result<()> {
    let opt = Zoxide::from_args();
    let env = envy::prefixed("_ZO_")
        .from_env::<Env>()
        .with_context(|| "could not parse environment variables")?;

    match opt {
        Zoxide::Add(add) => add.run(&env)?,
        Zoxide::Init(init) => init.run()?,
        Zoxide::Migrate(migrate) => migrate.run(&env)?,
        Zoxide::Query(query) => query.run(&env)?,
        Zoxide::Remove(remove) => remove.run(&env)?,
    };

    Ok(())
}
