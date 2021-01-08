use super::Cmd;
use crate::config;
use crate::fzf::Fzf;
use crate::util;

use crate::store::{self, StoreBuilder};
use anyhow::{Context, Result};
use clap::Clap;

use std::io::{self, Write};

/// Searches for a directory
#[derive(Clap, Debug)]
pub struct Query {
    keywords: Vec<String>,

    /// Lists all matching directories
    #[clap(long, short, conflicts_with = "list")]
    interactive: bool,

    /// Lists all matching directories
    #[clap(long, short, conflicts_with = "interactive")]
    list: bool,

    /// Prints score with results
    #[clap(long, short)]
    score: bool,
}

impl Cmd for Query {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let mut store = StoreBuilder::new(data_dir);
        let mut store = store.build()?;

        let query = store::Query::new(&self.keywords);
        let now = util::current_time()?;

        let mut matches = store.iter_matches(&query, now);

        if self.interactive {
            let mut fzf = Fzf::new()?;
            let handle = fzf.stdin();
            for dir in matches {
                writeln!(handle, "{}", dir.display_score(now)).context("could not write to fzf")?;
            }
            let selection = fzf.wait_select()?;
            if self.score {
                print!("{}", selection);
            } else {
                let path = selection
                    .get(5..)
                    .context("could not read selection from fzf")?;
                print!("{}", path)
            }
        } else if self.list {
            let stdout = io::stdout();
            let handle = &mut stdout.lock();
            for dir in matches {
                if self.score {
                    writeln!(handle, "{}", dir.display_score(now))
                } else {
                    writeln!(handle, "{}", dir.display())
                }
                .unwrap()
            }
        } else {
            let dir = matches.next().context("no match found")?;
            if self.score {
                println!("{}", dir.display_score(now))
            } else {
                println!("{}", dir.display())
            }
        }

        Ok(())
    }
}
