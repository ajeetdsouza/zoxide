use crate::util;

use anyhow::{bail, Result};
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
            self.query()?
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

    fn query(&self) -> Result<Option<Vec<u8>>> {
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

        let path_opt = util::get_db()?.query(&keywords, now).map(|dir| {
            // `path_to_bytes` is guaranteed to succeed here since
            // the path has already been queried successfully
            let path_bytes = util::path_to_bytes(&dir.path).unwrap();
            path_bytes.to_vec()
        });

        Ok(path_opt)
    }

    fn query_interactive(&self) -> Result<Option<Vec<u8>>> {
        let now = util::get_current_time()?;

        let keywords = self
            .keywords
            .iter()
            .map(|keyword| keyword.to_lowercase())
            .collect::<Vec<_>>();

        let mut db = util::get_db()?;
        let dirs = db.query_all(&keywords);
        util::fzf_helper(now, dirs)
    }
}
