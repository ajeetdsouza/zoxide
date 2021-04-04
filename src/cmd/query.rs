use super::Cmd;
use crate::config;
use crate::db::{self, DatabaseFile};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

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

    /// Excludes a path from results
    #[clap(long, hidden = true)]
    exclude: Option<String>,
}

impl Cmd for Query {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        let query = db::Query::new(&self.keywords);
        let now = util::current_time()?;

        let resolve_symlinks = config::zo_resolve_symlinks();
        let mut matches = db
            .iter_matches(&query, now, resolve_symlinks)
            .filter(|dir| Some(dir.path.as_ref()) != self.exclude.as_deref());

        if self.interactive {
            let mut fzf = Fzf::new()?;
            let handle = fzf.stdin();
            for dir in matches {
                writeln!(handle, "{}", dir.display_score(now)).handle_err("fzf")?;
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
                .handle_err("stdout")?;
            }
        } else {
            let dir = matches.next().context("no match found")?;
            if self.score {
                writeln!(io::stdout(), "{}", dir.display_score(now))
            } else {
                writeln!(io::stdout(), "{}", dir.display())
            }
            .handle_err("stdout")?;
        }

        Ok(())
    }
}
