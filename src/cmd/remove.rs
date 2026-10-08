use anyhow::{Result, bail};

use crate::cmd::{Remove, Run};
use crate::db::Database;
use crate::{config, util};

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        for path in &self.paths {
            if db.remove(path) {
                continue;
            }

            let path_abs = util::resolve_path(path)?;
            let path_abs = util::path_to_str(&path_abs)?;
            if path_abs != path && db.remove(path_abs) {
                continue;
            }

            if config::resolve_symlinks()
                && let Ok(path_canonical) = util::canonicalize(path)
                && let Ok(path_canonical) = util::path_to_str(&path_canonical)
                && db.remove(path_canonical)
            {
                continue;
            }

            bail!("path not found in database: {path}")
        }

        db.save()
    }
}
