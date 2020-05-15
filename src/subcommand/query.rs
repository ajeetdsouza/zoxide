use crate::db::Db;
use crate::util;

use anyhow::{bail, Result};
use float_ord::FloatOrd;
use structopt::StructOpt;

use std::io::{self, Write};
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
            self.query_interactive()?
        } else {
            let mut db = util::get_db()?;
            self.query(&mut db)?
        };

        match path_opt {
            Some(path) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&path).unwrap();
                handle.write_all(b"\n").unwrap();
            }
            None => bail!("no match found"),
        };

        Ok(())
    }

    fn query(&self, db: &mut Db) -> Result<Option<Vec<u8>>> {
        // if the input is already a valid path, simply return it
        if let [path] = self.keywords.as_slice() {
            if Path::new(path).is_dir() {
                return Ok(Some(path.as_bytes().to_vec()));
            }
        }

        let now = util::get_current_time()?;

        let keywords = self
            .keywords
            .iter()
            .map(|keyword| keyword.to_lowercase())
            .collect::<Vec<_>>();

        db.dirs
            .sort_unstable_by_key(|dir| FloatOrd(dir.get_frecency(now)));

        // Iterating in reverse order ensures that the directory indices do not
        // change as we remove them.
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

            let path = util::path_to_bytes(&dir.path)?.to_vec();
            return Ok(Some(path));
        }

        Ok(None)
    }

    fn query_interactive(&self) -> Result<Option<Vec<u8>>> {
        let now = util::get_current_time()?;

        let keywords = self
            .keywords
            .iter()
            .map(|keyword| keyword.to_lowercase())
            .collect::<Vec<_>>();

        let mut db = util::get_db()?;
        let dirs = db.query_many(&keywords);
        util::fzf_helper(now, dirs)
    }
}
