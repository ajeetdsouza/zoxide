use super::Cmd;
use crate::config;
use crate::db::{DatabaseFile, Query};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Result};
use clap::Clap;

use std::io::Write;

/// Remove a directory from the database
#[derive(Clap, Debug)]
pub struct Remove {
    // Use interactive selection
    #[clap(conflicts_with = "path", long, short, value_name = "keywords")]
    interactive: Option<Vec<String>>,
    #[clap(
        conflicts_with = "interactive",
        required_unless_present = "interactive"
    )]
    path: Option<String>,
}

impl Cmd for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        let selection;
        match &self.interactive {
            Some(keywords) => {
                let query = Query::new(keywords);
                let now = util::current_time()?;
                let resolve_symlinks = config::zo_resolve_symlinks();

                let mut fzf = Fzf::new(true)?;
                for dir in db.iter_matches(&query, now, resolve_symlinks) {
                    writeln!(fzf.stdin(), "{}", dir.display_score(now)).wrap_write("fzf")?;
                }

                selection = fzf.wait_select()?;
                let paths = selection.lines().filter_map(|line| line.get(5..));
                let mut not_found = Vec::new();
                for path in paths {
                    if !db.remove(&path) {
                        not_found.push(path);
                    }
                }

                if !not_found.is_empty() {
                    let mut err = "path not found in database:".to_string();
                    for path in not_found {
                        err.push_str("\n  ");
                        err.push_str(path.as_ref());
                    }
                    bail!(err);
                }
            }
            None => {
                // unwrap is safe here because path is required_unless_present = "interactive"
                let path = self.path.as_ref().unwrap();
                if !db.remove(path) {
                    let path_abs = util::resolve_path(&path)?;
                    let path_abs = util::path_to_str(&path_abs)?;
                    if path_abs != path && !db.remove(path) {
                        bail!("path not found in database:\n  {}", &path)
                    }
                }
            }
        }

        Ok(())
    }
}
