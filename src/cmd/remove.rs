use anyhow::{bail, Result};

use crate::cmd::{Remove, Run};
use crate::db::DatabaseFile;
use crate::{config, util};

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        for path in &self.paths {
            if !db.remove(path) {
                let path_abs = util::resolve_path(path)?;
                let path_abs = util::path_to_str(&path_abs)?;
                if path_abs == path || !db.remove(path_abs) {
                    db.modified = false;
                    bail!("path not found in database: {path}")
                }
            }
        }

        db.save()
    }
}
