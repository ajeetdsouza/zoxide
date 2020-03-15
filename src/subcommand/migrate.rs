use crate::env::Env;
use crate::util;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Migrate from z database")]
pub struct Migrate {
    path: String,

    #[structopt(long, help = "Merge entries into existing database")]
    merge: bool,
}

impl Migrate {
    pub fn run(&self, env: &Env) -> Result<()> {
        util::get_db(env)?.migrate(&self.path, self.merge)
    }
}
