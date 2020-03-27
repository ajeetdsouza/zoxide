use crate::env::Env;
use crate::util;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Import from z database")]
pub struct Import {
    path: String,

    #[structopt(long, help = "Merge entries into existing database")]
    merge: bool,
}

impl Import {
    pub fn run(&self, env: &Env) -> Result<()> {
        util::get_db(env)?.import(&self.path, self.merge)
    }
}
