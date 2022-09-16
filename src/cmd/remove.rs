use std::io::{self, Write};

use anyhow::{bail, Context, Result};

use crate::cmd::{Remove, Run};
use crate::config;
use crate::db::DatabaseFile;
use crate::util::{self, Fzf};

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        if self.interactive {
            let keywords = &self.paths;
            let now = util::current_time()?;
            let mut stream = db.stream(now).with_keywords(keywords);

            let mut fzf = Fzf::new(true)?;
            let stdin = fzf.stdin();

            let selection = loop {
                let dir = match stream.next() {
                    Some(dir) => dir,
                    None => break fzf.select()?,
                };

                match writeln!(stdin, "{}", dir.display_score(now)) {
                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break fzf.select()?,
                    result => result.context("could not write to fzf")?,
                }
            };

            let paths = selection.lines().filter_map(|line| line.get(5..));
            for path in paths {
                if !db.remove(path) {
                    db.modified = false;
                    bail!("path not found in database: {path}");
                }
            }
        } else {
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
        }

        db.save()
    }
}
