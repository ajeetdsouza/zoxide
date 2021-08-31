use crate::app::{Remove, Run};
use crate::config;
use crate::db::DatabaseFile;
use crate::error::BrokenPipeHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Result};

use std::io::Write;

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        let selection;
        match &self.interactive {
            Some(keywords) => {
                let now = util::current_time()?;
                let mut stream = db.stream(now).with_keywords(keywords);

                let mut fzf = Fzf::new(true)?;
                while let Some(dir) = stream.next() {
                    writeln!(fzf.stdin(), "{}", dir.display_score(now)).pipe_exit("fzf")?;
                }

                selection = fzf.wait_select()?;
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
