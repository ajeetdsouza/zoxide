use crate::env::Env;
use crate::util;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    path: String,
}

impl Remove {
    pub fn run(&self, env: &Env) -> Result<()> {
        util::get_db(env)?.remove(&self.path)
    }
}
