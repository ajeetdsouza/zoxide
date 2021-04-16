use super::Cmd;
use crate::config;
use crate::db::{self, DatabaseFile};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{Context, Result};
use clap::Clap;

use std::io::{self, BufWriter, Write};

/// Search for a directory in the database
#[derive(Clap, Debug)]
pub struct Query {
    keywords: Vec<String>,

    /// Use interactive selection
    #[clap(long, short, conflicts_with = "list")]
    interactive: bool,

    /// List all matching directories
    #[clap(long, short, conflicts_with = "interactive")]
    list: bool,

    /// Print score with results
    #[clap(long, short)]
    score: bool,

    /// Exclude a path from results
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
            // Rust does line-buffering by default, i.e. it flushes stdout
            // after every newline. This is not ideal when printing a large
            // number of lines, so we put stdout in a BufWriter.
            let stdout = io::stdout();
            let stdout = stdout.lock();
            let mut handle = BufWriter::new(stdout);

            for dir in matches {
                if self.score {
                    writeln!(handle, "{}", dir.display_score(now))
                } else {
                    writeln!(handle, "{}", dir.display())
                }
                .handle_err("stdout")?;
            }
            handle.flush().handle_err("stdout")?;
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
