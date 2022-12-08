use std::io::{self, Write};
use std::process::Command;

use anyhow::Result;

use crate::cmd::{Edit, EditCommand, Run};
use crate::db::{Database, DatabaseFile, Epoch};
use crate::{config, util};

impl Run for Edit {
    fn run(&self) -> Result<()> {
        let now = util::current_time()?;

        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let db = &mut db.open()?;

        match &self.cmd {
            Some(EditCommand::Decrement { path }) => {
                if let Some(dir) = db.dirs.iter_mut().find(|dir| &dir.path == path) {
                    dir.rank = (dir.rank - 1.0).max(0.0);
                }
                db.modified = true;
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Delete { path }) => {
                if let Some(idx) = db.dirs.iter().position(|dir| &dir.path == path) {
                    db.dirs.remove(idx);
                }
                db.modified = true;
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Increment { path }) => {
                if let Some(dir) = db.dirs.iter_mut().find(|dir| &dir.path == path) {
                    dir.rank += 1.0;
                }
                db.modified = true;
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Reload) => print_dirs(db, now),
            None => {
                db.dirs.sort_unstable_by(|dir1, dir2| dir2.score(now).total_cmp(&dir1.score(now)));
                db.modified = true;
                db.save()?;

                let mut fzf = Command::new("fzf");
                fzf.args([
                    "--bind=\
ctrl-r:reload(zoxide edit reload),\
ctrl-w:reload(zoxide edit delete {2..}),\
ctrl-a:reload(zoxide edit increment {2..}),\
ctrl-d:reload(zoxide edit decrement {2..}),\
double-click:ignore,\
enter:abort",
                    "--header=\
ctrl-r:reload     ctrl-w:delete
ctrl-a:increment  ctrl-d:decrement

SCORE PATH",
                    //
                    // Search mode
                    "--delimiter=\\x00 ",
                    "--nth=2",
                    // Search result
                    "--no-sort",
                    // Interface
                    "--cycle",
                    "--keep-right",
                    // Layout
                    "--info=inline",
                    "--layout=reverse",
                    // Key/Event bindings
                    "--bind=ctrl-z:ignore",
                ])
                .envs([("FZF_DEFAULT_COMMAND", "zoxide edit reload")]);

                let mut fzf = fzf.spawn().unwrap();
                fzf.wait().unwrap();
            }
        }

        Ok(())
    }
}

fn print_dirs(db: &Database, now: Epoch) {
    let stdout = &mut io::stdout().lock();
    for dir in db.dirs.iter() {
        writeln!(stdout, "{:>5}\x00 {}", dir.score(now), &dir.path).unwrap();
    }
}
