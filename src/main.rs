mod db;
mod dir;
mod subcommand;
mod types;
mod util;

use anyhow::Result;
use structopt::StructOpt;

// TODO: use structopt to parse env variables: <https://github.com/TeXitoi/structopt/blob/master/examples/env.rs>

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
    match opt {
        Zoxide::Add(add) => add.run()?,
        Zoxide::Init(init) => init.run()?,
        Zoxide::Migrate(migrate) => migrate.run()?,
        Zoxide::Query(query) => query.run()?,
        Zoxide::Remove(remove) => remove.run()?,
    };

    Ok(())
}
