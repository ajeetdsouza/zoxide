use crate::db::Dir;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use std::io::{self, Write};
use std::path::Path;

/// Search for a directory
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Query {
    keywords: Vec<String>,

    /// Opens an interactive selection menu using fzf
    #[structopt(short, long, conflicts_with = "list")]
    interactive: bool,

    /// List all matching directories
    #[structopt(short, long, conflicts_with = "interactive")]
    list: bool,

    /// Display score along with result
    #[structopt(short, long)]
    score: bool,
}

impl Query {
    pub fn run(&self) -> Result<()> {
        if self.list {
            return self.query_list();
        }

        if self.interactive {
            return self.query_interactive();
        }

        // if the input is already a valid path, simply print it as-is
        if let [path] = self.keywords.as_slice() {
            if Path::new(path).is_dir() {
                let dir = Dir {
                    path: path.to_string(),
                    rank: 0.0,
                    last_accessed: 0,
                };

                if self.score {
                    println!("{}", dir.display_score(0))
                } else {
                    println!("{}", dir.display());
                }

                return Ok(());
            }
        }

        self.query()
    }

    fn query(&self) -> Result<()> {
        let mut db = util::get_db()?;
        let now = util::get_current_time()?;

        let mut matches = db.matches(now, &self.keywords);

        match matches.next() {
            Some(dir) => {
                if self.score {
                    println!("{}", dir.display_score(now))
                } else {
                    println!("{}", dir.display());
                }
            }
            None => bail!("no match found"),
        }

        Ok(())
    }

    fn query_interactive(&self) -> Result<()> {
        let mut db = util::get_db()?;
        let now = util::get_current_time()?;

        let mut fzf = Fzf::new()?;
        let mut matches = db.matches(now, &self.keywords);

        while let Some(dir) = matches.next() {
            fzf.write(format!("{}", dir.display_score(now)))?;
        }

        match fzf.wait_select()? {
            Some(selection) => {
                if self.score {
                    print!("{}", selection)
                } else {
                    let selection = selection
                        .get(5..)
                        .with_context(|| format!("fzf returned invalid output: {}", selection))?;
                    print!("{}", selection)
                }
            }
            None => bail!("no match found"),
        };

        Ok(())
    }

    fn query_list(&self) -> Result<()> {
        let mut db = util::get_db()?;
        let now = util::get_current_time()?;

        let mut matches = db.matches(now, &self.keywords);

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        while let Some(dir) = matches.next() {
            if self.score {
                writeln!(handle, "{}", dir.display_score(now))
            } else {
                writeln!(handle, "{}", dir.display())
            }
            .unwrap();
        }

        Ok(())
    }
}
