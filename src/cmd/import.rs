use super::Run;
use crate::app::{Import, ImportFrom};
use crate::config;
use crate::import::{Autojump, Import as _, Z};
use crate::util;

use crate::db::DatabaseFile;
use anyhow::{bail, Result};

impl Run for Import {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;
        if !self.merge && !db.dirs.is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        let resolve_symlinks = config::zo_resolve_symlinks();
        match self.from {
            ImportFrom::Autojump => Autojump {
                resolve_symlinks,
                now: util::current_time()?,
            }
            .import(&mut db, &self.path),
            ImportFrom::Z => Z { resolve_symlinks }.import(&mut db, &self.path),
        }
    }
}
