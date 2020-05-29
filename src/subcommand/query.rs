use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Result};
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
}

impl Query {
    pub fn run(&self) -> Result<()> {
        if self.list {
            return query_list(&self.keywords);
        }

        if self.interactive {
            return query_interactive(&self.keywords);
        }

        // if the input is already a valid path, simply return it
        if let [path] = self.keywords.as_slice() {
            if Path::new(path).is_dir() {
                println!("{}", path);
                return Ok(());
            }
        }

        query(&self.keywords)
    }
}

fn query(keywords: &[String]) -> Result<()> {
    let mut db = util::get_db()?;
    let now = util::get_current_time()?;

    let mut matches = db.matches(now, keywords);

    match matches.next() {
        Some(dir) => println!("{}", dir.path),
        None => bail!("no match found"),
    }

    Ok(())
}

fn query_interactive(keywords: &[String]) -> Result<()> {
    let mut db = util::get_db()?;
    let now = util::get_current_time()?;

    let mut fzf = Fzf::new()?;

    let mut matches = db.matches(now, keywords);

    while let Some(dir) = matches.next() {
        fzf.write_dir(dir, now);
    }

    match fzf.wait_selection()? {
        Some(path) => println!("{}", path),
        None => bail!("no match found"),
    };

    Ok(())
}

fn query_list(keywords: &[String]) -> Result<()> {
    let mut db = util::get_db()?;
    let now = util::get_current_time()?;

    let mut matches = db.matches(now, keywords);

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    while let Some(dir) = matches.next() {
        let path = &dir.path;
        writeln!(handle, "{}", path).unwrap();
    }

    Ok(())
}
