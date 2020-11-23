use super::Cmd;
use crate::config;
use crate::fuzzy_finder::FuzzyFinder;
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

                let mut fuzzy_finder = FuzzyFinder::new()?;
                let fuzzy_finder_cmd = fuzzy_finder.cmd_name();
                let handle = fuzzy_finder.stdin();
                for dir in store.iter_matches(&query, now) {
                    writeln!(handle, "{}", dir.display_score(now))
                        .with_context(|| format!("could not write to {}", fuzzy_finder_cmd))?;
                }

                selection = fuzzy_finder.wait_select()?;
                selection
                    .get(5..selection.len().saturating_sub(1))
                    .with_context(|| format!("{} returned invalid output", fuzzy_finder_cmd))?
            }
            None => self.path.as_ref().unwrap(),
        };

        if !store.remove(path) {
            let path = util::resolve_path(&path)?;
            let path = util::path_to_str(&path)?;
            if !store.remove(path) {
                bail!("path not found in store: {}", &path)
            }
        }

        Ok(())
    }
}
