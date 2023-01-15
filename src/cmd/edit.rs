use std::io::{self, Write};

use anyhow::Result;

use crate::cmd::{Edit, EditCommand, Run};
use crate::db::Database;
use crate::error::BrokenPipeHandler;
use crate::util::{self, Fzf, FzfChild};

impl Run for Edit {
    fn run(&self) -> Result<()> {
        let now = util::current_time()?;
        let db = &mut Database::open()?;

        match &self.cmd {
            Some(cmd) => {
                match cmd {
                    EditCommand::Decrement { path } => db.add(path, -1.0, now),
                    EditCommand::Delete { path } => {
                        db.remove(path);
                    }
                    EditCommand::Increment { path } => db.add(path, 1.0, now),
                    EditCommand::Reload => {}
                }
                db.save()?;

                let stdout = &mut io::stdout().lock();
                for dir in db.dirs().iter().rev() {
                    write!(stdout, "{}\0", dir.display().with_score(now).with_separator('\t'))
                        .pipe_exit("fzf")?;
                }
                Ok(())
            }
            None => {
                db.sort_by_score(now);
                db.save()?;
                Self::get_fzf()?.wait()?;
                Ok(())
            }
        }
    }
}

impl Edit {
    fn get_fzf() -> Result<FzfChild> {
        Fzf::new()?
            .args([
                // Search mode
                "--exact",
                // Search result
                "--no-sort",
                // Interface
                "--bind=\
btab:up,\
ctrl-r:reload(zoxide edit reload),\
ctrl-d:reload(zoxide edit delete {2..}),\
ctrl-w:reload(zoxide edit increment {2..}),\
ctrl-s:reload(zoxide edit decrement {2..}),\
ctrl-z:ignore,\
double-click:ignore,\
enter:abort,\
start:reload(zoxide edit reload),\
tab:down",
                "--cycle",
                "--keep-right",
                // Layout
                "--border=sharp",
                "--border-label=  zoxide-edit  ",
                "--header=\
ctrl-r:reload   \tctrl-d:delete
ctrl-w:increment\tctrl-s:decrement

 SCORE\tPATH",
                "--info=inline",
                "--layout=reverse",
                "--padding=1,0,0,0",
                // Display
                "--color=label:bold",
                "--tabstop=1",
            ])
            .enable_preview()
            .spawn()
    }
}
