use crate::env::Env;
use crate::types::Rank;
use crate::util;

use anyhow::{Context, Result};
use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Add a new directory or increment its rank")]
pub struct Add {
    path: Option<String>,
}

impl Add {
    pub fn run(&self, env: &Env) -> Result<()> {
        let mut db = util::get_db(env)?;
        let now = util::get_current_time()?;
        let maxage = env.maxage as Rank;

        match &self.path {
            Some(path) => db.add(path, maxage, now),
            None => {
                let current_dir =
                    env::current_dir().context("unable to fetch current directory")?;
                db.add(current_dir, maxage, now)
            }
        }
    }
}
