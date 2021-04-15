use super::Cmd;
use crate::config;
use crate::db::{DatabaseFile, Query};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Context, Result};
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
        let path = match &self.interactive {
            Some(keywords) => {
                let query = Query::new(keywords);
                let now = util::current_time()?;

                let mut fzf = Fzf::new()?;
                let handle = fzf.stdin();
                let resolve_symlinks = config::zo_resolve_symlinks();
                for dir in db.iter_matches(&query, now, resolve_symlinks) {
                    writeln!(handle, "{}", dir.display_score(now)).handle_err("fzf")?;
                }

                selection = fzf.wait_select()?;
                selection
                    .get(5..selection.len().saturating_sub(1))
                    .context("fzf returned invalid output")?
            }
            None => self.path.as_ref().unwrap(),
        };

        if !db.remove(path) {
            let path = util::resolve_path(&path)?;
            let path = util::path_to_str(&path)?;
            if !db.remove(path) {
                bail!("path not found in database: {}", &path)
            }
        }

        Ok(())
    }
}
