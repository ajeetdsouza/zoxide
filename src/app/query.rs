use super::Run;
use crate::app::Query;
use crate::config;
use crate::db::{self, DatabaseFile};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{Context, Result};

use std::io::{self, BufWriter, Write};

impl Run for Query {
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
            let mut fzf = Fzf::new(false)?;
            for dir in matches {
                writeln!(fzf.stdin(), "{}", dir.display_score(now)).pipe_exit("fzf")?;
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
                .pipe_exit("stdout")?;
            }
            handle.flush().pipe_exit("stdout")?;
        } else {
            let dir = matches.next().context("no match found")?;
            if self.score {
                writeln!(io::stdout(), "{}", dir.display_score(now))
            } else {
                writeln!(io::stdout(), "{}", dir.display())
            }
            .pipe_exit("stdout")?;
        }

        Ok(())
    }
}
