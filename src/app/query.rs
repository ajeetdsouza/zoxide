use crate::app::{Query, Run};
use crate::config;
use crate::db::DatabaseFile;
use crate::error::BrokenPipeHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{Context, Result};

use std::io::{self, BufWriter, Write};

impl Run for Query {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;
        let now = util::current_time()?;

        let mut stream = db.stream(now).with_keywords(&self.keywords);
        if !self.all {
            let resolve_symlinks = config::zo_resolve_symlinks();
            stream = stream.with_exists(resolve_symlinks);
        }
        if let Some(path) = &self.exclude {
            stream = stream.with_exclude(path);
        }

        if self.interactive {
            let mut fzf = Fzf::new(false)?;
            while let Some(dir) = stream.next() {
                writeln!(fzf.stdin(), "{}", dir.display_score(now)).pipe_exit("fzf")?;
            }

            let selection = fzf.wait_select()?;
            if self.score {
                print!("{}", selection);
            } else {
                let path = selection.get(5..).context("could not read selection from fzf")?;
                print!("{}", path)
            }
        } else if self.list {
            // Rust does line-buffering by default, i.e. it flushes stdout
            // after every newline. This is not ideal when printing a large
            // number of lines, so we put stdout in a BufWriter.
            let stdout = io::stdout();
            let stdout = stdout.lock();
            let mut handle = BufWriter::new(stdout);

            while let Some(dir) = stream.next() {
                if self.score {
                    writeln!(handle, "{}", dir.display_score(now))
                } else {
                    writeln!(handle, "{}", dir.display())
                }
                .pipe_exit("stdout")?;
            }
            handle.flush().pipe_exit("stdout")?;
        } else {
            let dir = stream.next().context("no match found")?;
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
