use crate::fzf::Fzf;
use crate::util::{get_current_time, get_db, path_to_str};

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    query: Vec<String>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        if self.interactive {
            remove_interactive(&self.query)
        } else {
            if let &[path] = &self.query.as_slice() {
                remove(&path)
            } else {
                clap::Error::with_description(
                    &format!(
                        "remove requires 1 value in non-interactive mode, but {} were provided",
                        self.query.len()
                    ),
                    clap::ErrorKind::WrongNumberOfValues,
                )
                .exit();
            }
        }
    }
}

fn remove(path_str: &str) -> Result<()> {
    let mut db = get_db()?;

    if let Some(idx) = db.dirs.iter().position(|dir| &dir.path == path_str) {
        db.dirs.swap_remove(idx);
        db.modified = true;
        return Ok(());
    }

    let path_abs = dunce::canonicalize(path_str)
        .with_context(|| format!("could not resolve path: {}", path_str))?;
    let path_abs_str = path_to_str(&path_abs)?;

    if let Some(idx) = db.dirs.iter().position(|dir| dir.path == path_abs_str) {
        db.dirs.swap_remove(idx);
        db.modified = true;
        return Ok(());
    }

    bail!("could not find path in database: {}", path_str)
}

fn remove_interactive(keywords: &[String]) -> Result<()> {
    let mut db = get_db()?;
    let now = get_current_time()?;

    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<_>>();

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

        fzf.write_dir(&dir, now);
    }

    if let Some(path) = fzf.wait_selection()? {
        if let Some(idx) = db.dirs.iter().position(|dir| dir.path == path) {
            db.dirs.swap_remove(idx);
            db.modified = true;
            return Ok(());
        }
    }

    bail!("no match found");
}
