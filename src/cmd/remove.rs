use crate::cmd::Cmd;
use crate::config;
use crate::fzf::Fzf;
use crate::store::Query;
use crate::store::Store;
use crate::util;

use anyhow::{bail, Context, Result};
use clap::Clap;

use std::io::Write;

/// Removes a directory
#[derive(Clap, Debug)]
pub struct Remove {
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
        let mut store = Store::open(&data_dir)?;

        let selection;
        let path = match &self.interactive {
            Some(keywords) => {
                let query = Query::new(keywords);
                let now = util::current_time()?;

                let mut fzf = Fzf::new()?;
                let handle = fzf.stdin();
                for dir in store.iter_matches(&query, now) {
                    writeln!(handle, "{}", dir.display_score(now))
                        .context("could not write to fzf")?;
                }

                selection = fzf.wait_select()?;
                selection
                    .get(5..selection.len().saturating_sub(1))
                    .context("fzf returned invalid output")?
            }
            None => self.path.as_ref().unwrap(),
        };

        if !store.remove(path) {
            bail!("path not found in store: {}", &path)
        }

        Ok(())
    }
}
