use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::cmd::{Query, Run};
use crate::config;
use crate::db::{Database, Epoch, Stream, StreamOptions};
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
        let mut stream = self.get_stream(db, now)?;

        if self.interactive {
            self.query_interactive(&mut stream, now)
        } else if self.list {
            self.query_list(&mut stream, now)
        } else {
            self.query_first(&mut stream, now)
        }
    }

    fn query_interactive(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        // Paths are abbreviated with `~` only while being displayed in fzf, the
        // output always contains the full path.
        let home = if config::tilde() { util::home_dir() } else { None };
        let home = home.as_deref();

        let mut fzf = Self::get_fzf(home)?;
        let selection = loop {
            match stream.next() {
                Some(dir) if Some(dir.path.as_ref()) == self.exclude.as_deref() => continue,
                Some(dir) => {
                    if let Some(selection) = fzf.write(dir, now, home)? {
                        break selection;
                    }
                }
                None => break fzf.wait()?,
            }
        };

        let (score, path) =
            selection.split_at_checked(7).context("could not read selection from fzf")?;
        let path = match home {
            Some(home) => util::expand_tilde(path, home),
            None => path.into(),
        };

        let stdout = &mut io::stdout();
        if self.score {
            write!(stdout, "{score}{path}").pipe_exit("stdout")?;
        } else {
            write!(stdout, "{path}").pipe_exit("stdout")?;
        }
        stdout.flush().pipe_exit("stdout")
    }

    fn query_list(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        let handle = &mut io::stdout().lock();
        while let Some(dir) = stream.next() {
            if Some(dir.path.as_ref()) == self.exclude.as_deref() {
                continue;
            }
            let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
            writeln!(handle, "{dir}").pipe_exit("stdout")?;
        }
        Ok(())
    }

    fn query_first(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        let handle = &mut io::stdout();

        let mut dir = stream.next().context("no match found")?;
        while Some(dir.path.as_ref()) == self.exclude.as_deref() {
            dir = stream.next().context("you are already in the only match")?;
        }

        let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
        writeln!(handle, "{dir}").pipe_exit("stdout")
    }

    fn get_stream<'a>(&self, db: &'a mut Database, now: Epoch) -> Result<Stream<'a>> {
        let mut options = StreamOptions::new(now)
            .with_keywords(self.keywords.iter().map(|s| s.as_str()))
            .with_exclude(config::exclude_dirs()?)
            .with_base_dir(self.base_dir.clone());
        if !self.all {
            let resolve_symlinks = config::resolve_symlinks();
            options = options.with_exists(true).with_resolve_symlinks(resolve_symlinks);
        }

        let stream = Stream::new(db, options);
        Ok(stream)
    }

    fn get_fzf(home: Option<&str>) -> Result<FzfChild> {
        let mut fzf = Fzf::new()?;
        if let Some(fzf_opts) = config::fzf_opts() {
            fzf.env("FZF_DEFAULT_OPTS", fzf_opts)
        } else {
            fzf.args([
                // Search mode
                "--exact",
                // Search result
                "--no-sort",
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
            ])
            .enable_preview(home)
        }
        .spawn()
    }
}
