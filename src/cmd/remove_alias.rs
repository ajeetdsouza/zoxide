use anyhow::{Result, bail};

use crate::cmd::{RemoveAlias, Run};
use crate::db::Database;
use crate::util;

impl Run for RemoveAlias {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        if !db.remove_alias(&self.path, self.aliases.iter()) {
            let path_abs = util::resolve_path(&self.path)?;
            let path_abs = util::path_to_str(&path_abs)?;
            if path_abs == self.path || !db.remove_alias(path_abs, self.aliases.iter()) {
                bail!("path not found in database: {}", &self.path)
            }
        }

        db.save()
    }
}
