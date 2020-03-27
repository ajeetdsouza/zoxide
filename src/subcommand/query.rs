use crate::util;

use anyhow::{bail, Result};
use std::io::{self, Write};
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Search for a directory")]
pub struct Query {
    keywords: Vec<String>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Query {
    pub fn run(mut self) -> Result<()> {
        let path_opt = if self.interactive {
            self.query_interactive()?
        } else {
            self.query()?
        };

        match path_opt {
            Some(path) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(b"query: ").unwrap();
                handle.write_all(&path).unwrap();
                handle.write_all(b"\n").unwrap();
            }
            None => bail!("no match found"),
        };

        Ok(())
    }

    fn query(&mut self) -> Result<Option<Vec<u8>>> {
        if let [path] = self.keywords.as_slice() {
            if Path::new(path).is_dir() {
                return Ok(Some(path.as_bytes().to_vec()));
            }
        }

        let now = util::get_current_time()?;

        for keyword in &mut self.keywords {
            *keyword = keyword.to_lowercase();
        }

        if let Some(dir) = util::get_db()?.query(&self.keywords, now) {
            // `path_to_bytes` is guaranteed to succeed here since
            // the path has already been queried successfully
            let path_bytes = util::path_to_bytes(&dir.path).unwrap();
            Ok(Some(path_bytes.to_vec()))
        } else {
            Ok(None)
        }
    }

    fn query_interactive(&mut self) -> Result<Option<Vec<u8>>> {
        let now = util::get_current_time()?;

        for keyword in &mut self.keywords {
            *keyword = keyword.to_lowercase();
        }

        let dirs = util::get_db()?.query_all(&self.keywords);
        util::fzf_helper(now, dirs)
    }
}
