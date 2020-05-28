use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Result};
use float_ord::FloatOrd;
use structopt::StructOpt;

use std::path::Path;

#[derive(Debug, StructOpt)]
#[structopt(about = "Search for a directory")]
pub struct Query {
    keywords: Vec<String>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Query {
    pub fn run(&self) -> Result<()> {
        let path_opt = if self.interactive {
            query_interactive(&self.keywords)?
        } else {
            query(&self.keywords)?
        };

        match path_opt {
            Some(path) => println!("{}", path),
            None => bail!("no match found"),
        };

        Ok(())
    }
}

fn query(keywords: &[String]) -> Result<Option<String>> {
    // if the input is already a valid path, simply return it
    if let [path] = keywords {
        if Path::new(path).is_dir() {
            return Ok(Some(path.to_string()));
        }
    }

    let mut db = util::get_db()?;
    let now = util::get_current_time()?;

    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<_>>();

    db.dirs
        .sort_unstable_by_key(|dir| FloatOrd(dir.get_frecency(now)));

    for idx in (0..db.dirs.len()).rev() {
        let dir = &db.dirs[idx];
        if !dir.is_match(&keywords) {
            continue;
        }

        if !dir.is_valid() {
            db.dirs.swap_remove(idx);
            db.modified = true;
            continue;
        }

        let path = &dir.path;
        return Ok(Some(path.to_string()));
    }

    Ok(None)
}

fn query_interactive(keywords: &[String]) -> Result<Option<String>> {
    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<_>>();

    let mut db = util::get_db()?;
    let now = util::get_current_time()?;

    let mut fzf = Fzf::new()?;

    for idx in (0..db.dirs.len()).rev() {
        let dir = &db.dirs[idx];

        if !dir.is_match(&keywords) {
            continue;
        }

        if !dir.is_valid() {
            db.dirs.swap_remove(idx);
            db.modified = true;
            continue;
        }

        fzf.write_dir(dir, now);
    }

    fzf.wait_selection()
}
