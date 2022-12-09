use std::io::{self, Write};
use std::process::Command;

use anyhow::Result;

use crate::cmd::{Edit, EditCommand, Run};
use crate::store::{Epoch, Store};
use crate::util;

impl Run for Edit {
    fn run(&self) -> Result<()> {
        let now = util::current_time()?;
        let db = &mut Store::open()?;

        match &self.cmd {
            Some(EditCommand::Decrement { path }) => {
                db.increment(path, -1.0, now);
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Delete { path }) => {
                db.remove(path);
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Increment { path }) => {
                db.increment(path, 1.0, now);
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Reload) => print_dirs(db, now),
            None => {
                db.sort_by_score(now);
                db.save()?;

                let mut fzf = Command::new("fzf");
                fzf.args([
                    // Search mode
                    "--delimiter=\\x00 ",
                    "--nth=2",
                    // Search result
                    "--no-sort",
                    // Interface
                    "--bind=\
                        ctrl-r:reload(zoxide edit reload),\
                        ctrl-w:reload(zoxide edit delete {2..}),\
                        ctrl-a:reload(zoxide edit increment {2..}),\
                        ctrl-d:reload(zoxide edit decrement {2..}),\
                        ctrl-z:ignore,\
                        double-click:ignore,\
                        enter:abort",
                    "--cycle",
                    "--keep-right",
                    // Layout
                    "--header=\
ctrl-r:reload     ctrl-w:delete
ctrl-a:increment  ctrl-d:decrement

SCORE PATH",
                    "--info=inline",
                    "--layout=reverse",
                ])
                .envs([("FZF_DEFAULT_COMMAND", "zoxide edit reload")]);

                let mut fzf = fzf.spawn().unwrap();
                fzf.wait().unwrap();
            }
        }

        Ok(())
    }
}

fn print_dirs(db: &Store, now: Epoch) {
    let stdout = &mut io::stdout().lock();
    for dir in db.dirs().iter().rev() {
        writeln!(stdout, "{:>5}\x00 {}", dir.score(now), &dir.path).unwrap();
    }
}
