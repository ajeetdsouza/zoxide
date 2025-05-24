use anyhow::Result;

use crate::cmd::{List, Run};
use crate::db::Database;

impl Run for List {
    fn run(&self) -> Result<()> {
        let db = Database::open()?;

        if db.dirs().len() == 0 {
            println!("No directory yet!")
        }

        for dir in db.dirs() {
            println!("{}", format!("{} {}", dir.rank, dir.path))
        }

        Ok(())
    }
}
