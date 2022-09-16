use std::path::Path;

use anyhow::{bail, Result};

use crate::cmd::{Add, Run};
use crate::db::DatabaseFile;
use crate::{config, util};

impl Run for Add {
    fn run(&self) -> Result<()> {
        // These characters can't be printed cleanly to a single line, so they can cause confusion
        // when writing to fzf / stdout.
        const EXCLUDE_CHARS: &[char] = &['\n', '\r'];

        let data_dir = config::data_dir()?;
        let exclude_dirs = config::exclude_dirs()?;
        let max_age = config::maxage()?;
        let now = util::current_time()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        for path in &self.paths {
            let path = if config::resolve_symlinks() { util::canonicalize } else { util::resolve_path }(path)?;
            let path = util::path_to_str(&path)?;

            // Ignore path if it contains unsupported characters, or if it's in the exclude list.
            if path.contains(EXCLUDE_CHARS) || exclude_dirs.iter().any(|glob| glob.matches(path)) {
                continue;
            }
            if !Path::new(path).is_dir() {
                bail!("not a directory: {path}");
            }
            db.add(path, now);
        }

        if db.modified {
            db.age(max_age);
            db.save()?;
        }

        Ok(())
    }
}
