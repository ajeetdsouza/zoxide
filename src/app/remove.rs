use std::io::{self, Write};

use anyhow::{bail, Context, Result};

use crate::app::{Remove, Run};
use crate::db::DatabaseFile;
use crate::fzf::Fzf;
use crate::{config, util};

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        match &self.interactive {
            Some(keywords) => {
                let now = util::current_time()?;
                let mut stream = db.stream(now).with_keywords(keywords);

                let mut fzf = Fzf::new(true)?;
                let selection = loop {
                    let dir = match stream.next() {
                        Some(dir) => dir,
                        None => break fzf.select()?,
                    };

                    match writeln!(fzf.stdin(), "{}", dir.display_score(now)) {
                        Ok(()) => (()),
                        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break fzf.select()?,
                        Err(e) => Err(e).context("could not write to fzf")?,
                    }
                };

                let paths = selection.lines().filter_map(|line| line.get(5..));
                for path in paths {
                    if !db.remove(path) {
                        bail!("path not found in database: {}", path);
                    }
                }
            }
            None => {
                for path in &self.paths {
                    if !db.remove(path) {
                        let path_abs = util::resolve_path(path)?;
                        let path_abs = util::path_to_str(&path_abs)?;
                        if path_abs != path && !db.remove(path_abs) {
                            bail!("path not found in database: {} ({})", path, path_abs)
                        }
                    }
                }
            }
        }

        db.save()
    }
}
