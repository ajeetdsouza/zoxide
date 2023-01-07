use anyhow::{bail, Result};

use crate::cmd::{Remove, Run};
use crate::db::Database;
use crate::util;

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        for path in &self.paths {
            if !db.remove(path) {
                let path_abs = util::resolve_path(path)?;
                let path_abs = util::path_to_str(&path_abs)?;
                if path_abs == path || !db.remove(path_abs) {
                    bail!("path not found in database: {path}")
                }
            }
        }

        db.save()
    }
}
