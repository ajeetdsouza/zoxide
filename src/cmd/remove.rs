use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::cmd::{Remove, Run};
use crate::db::Database;
use crate::util;

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        // Build a unified iterator of PathBufs, whether recursive or not.
        let paths: Box<dyn Iterator<Item = PathBuf>> = if self.recursive {
            Box::new(self.paths.iter().flat_map(|p| util::walk_dir(p)))
        } else {
            Box::new(self.paths.iter().cloned())
        };

        for path in paths {
            let path_abs = util::resolve_path(path)?;
            let path_abs = util::path_to_str(&path_abs)?;
            if !db.remove(path_abs) {
                bail!("path not found in database: {}", path_abs)
            }
        }

        db.save()
    }
}
