use std::io::{self, Write};

use anyhow::{bail, Context, Result};

use crate::cmd::{Query, Run};
use crate::config;
use crate::db::{Database, Epoch, Stream};
use crate::error::BrokenPipeHandler;
use crate::util::{self, Fzf, FzfChild};

impl Run for Query {
    fn run(&self) -> Result<()> {
        let mut db = crate::db::Database::open()?;
        self.query(&mut db).and(db.save())
    }
}

impl Query {
    fn query(&self, db: &mut Database) -> Result<()> {
        let now = util::current_time()?;
        let mut stream = self.get_stream(db, now);

        if self.interactive {
            let mut fzf = Self::get_fzf()?;
            let selection = loop {
                match stream.next() {
                    Some(dir) => {
                        if let Some(selection) = fzf.write(dir, now)? {
                            break selection;
                        }
                    }
                    None => break fzf.wait()?,
                }
            };

            if self.score {
                print!("{selection}");
            } else {
                let path = selection.get(7..).context("could not read selection from fzf")?;
                print!("{path}");
            }
        } else if self.list {
            let handle = &mut io::stdout().lock();
            while let Some(dir) = stream.next() {
                let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
                writeln!(handle, "{dir}").pipe_exit("stdout")?;
            }
        } else {
            let handle = &mut io::stdout();
            let Some(dir) = stream.next() else {
                bail!(if stream.did_exclude() {
                    "you are already in the only match"
                } else {
                    "no match found"
                });
            };
            let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
            writeln!(handle, "{dir}").pipe_exit("stdout")?;
        }

        Ok(())
    }

    fn get_stream<'a>(&self, db: &'a mut Database, now: Epoch) -> Stream<'a> {
        let mut stream = db.stream(now).with_keywords(&self.keywords);
        if !self.all {
            let resolve_symlinks = config::resolve_symlinks();
            stream = stream.with_exists(resolve_symlinks);
        }
        if let Some(path) = &self.exclude {
            stream = stream.with_exclude(path);
        }
        stream
    }

    fn get_fzf() -> Result<FzfChild> {
        let mut fzf = Fzf::new()?;
        if let Some(fzf_opts) = config::fzf_opts() {
            fzf.env("FZF_DEFAULT_OPTS", fzf_opts)
        } else {
            fzf.args([
                // Search mode
                "--scheme=path",
                // Search result
                "--tiebreak=end,chunk,index",
                // Interface
                "--bind=ctrl-z:ignore,btab:up,tab:down",
                "--cycle",
                "--keep-right",
                // Layout
                "--border=sharp", // rounded edges don't display correctly on some terminals
                "--height=45%",
                "--info=inline",
                "--layout=reverse",
                // Display
                "--tabstop=1",
                // Scripting
                "--exit-0",
                "--select-1",
            ])
            .enable_preview()
        }
        .spawn()
    }
}
