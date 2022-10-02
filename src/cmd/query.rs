use anyhow::{Context, Result};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

use crate::cmd::{Query, Run};
use crate::config;
use crate::db::{Database, DatabaseFile};
use crate::error::BrokenPipeHandler;
use crate::util::{self, Fzf};

impl Run for Query {
    fn run(&self) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;
        self.query(&mut db).and(db.save())
    }
}

impl Query {
    fn query(&self, db: &mut Database) -> Result<()> {
        let now = util::current_time()?;

        let mut stream = db.stream(now).with_keywords(&self.keywords);
        if !self.all {
            let resolve_symlinks = config::resolve_symlinks();
            stream = stream.with_exists(resolve_symlinks);
        }
        if let Some(path) = &self.exclude {
            stream = stream.with_exclude(path);
        }

        if self.interactive {
            let mut fzf = Fzf::new(false)?;
            let stdin = fzf.stdin();

            let selection = loop {
                let dir = match stream.next() {
                    Some(dir) => dir,
                    None => break fzf.select()?,
                };

                match writeln!(stdin, "{}", dir.display_score(now)) {
                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break fzf.select()?,
                    result => result.context("could not write to fzf")?,
                }
            };

            if self.score {
                print!("{selection}");
            } else {
                let path = selection.get(5..).context("could not read selection from fzf")?;
                print!("{path}");
            }
        } else if self.list {
            let handle = &mut io::stdout().lock();
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
            let excluded_dir = self.exclude.as_ref().map_or(PathBuf::new(), |path| PathBuf::from_str(path).unwrap());

            let try_dir = stream.next();
            let dir = match try_dir {
                Some(dir) => dir,
                None => {
                    // Current Directory is passed as excluded in __zoxide_z
                    if excluded_dir == env::current_dir()? {
                        try_dir.context("already in the matched directory")?
                    } else {
                        try_dir.context("no match found")?
                    }
                }
            };

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
