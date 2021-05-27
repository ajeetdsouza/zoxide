use crate::app::{Add, Run};
use crate::config;
use crate::db::DatabaseFile;
use crate::util;

use anyhow::{bail, Result};

use std::path::Path;

impl Run for Add {
    fn run(&self) -> Result<()> {
        let path = if config::resolve_symlinks() {
            util::canonicalize(&self.path)
        } else {
            util::resolve_path(&self.path)
        }?;
        let path = util::path_to_str(&path)?;
        let now = util::current_time()?;

        // These characters can't be printed cleanly to a single line, so they
        // can cause confusion when writing to fzf / stdout.
        const EXCLUDE_CHARS: &[char] = &['\n', '\r'];
        let mut exclude_dirs = config::exclude_dirs()?.into_iter();
        if exclude_dirs.any(|pattern| pattern.matches(path)) || path.contains(EXCLUDE_CHARS) {
            return Ok(());
        }
        if !Path::new(path).is_dir() {
            bail!("not a directory: {}", path);
        }

        let data_dir = config::data_dir()?;
        let max_age = config::maxage()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        db.add(path, now);
        db.age(max_age);

        Ok(())
    }
}
