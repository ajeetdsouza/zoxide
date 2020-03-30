use crate::util;

use anyhow::Result;
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    path: PathBuf,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        util::get_db()?.remove(&self.path)
    }
}
