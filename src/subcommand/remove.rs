use crate::util;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    path: String,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        util::get_db()?.remove(&self.path)
    }
}
