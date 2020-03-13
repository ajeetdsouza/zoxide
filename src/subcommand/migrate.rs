use crate::util;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Migrate from z database")]
pub struct Migrate {
    path: String,
}

impl Migrate {
    pub fn run(&self) -> Result<()> {
        util::get_db()?.migrate(&self.path)
    }
}
