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
                db.add(path, -1.0, now);
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Delete { path }) => {
                db.remove(path);
                db.save()?;
                print_dirs(db, now);
            }
            Some(EditCommand::Increment { path }) => {
                db.add(path, 1.0, now);
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
                    "--delimiter=\t",
                    "--nth=2",
                    "--scheme=path",
                    // Search result
                    "--tiebreak=end,chunk,index",
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
                    "--border=rounded",
                    "--border-label= zoxide-edit ",
                    "--header=\
ctrl-r:reload   \tctrl-w:delete
ctrl-a:increment\tctrl-d:decrement

SCORE\tPATH",
                    "--info=inline",
                    "--layout=reverse",
                    "--padding=1",
                    // Display
                    "--color=label:bold",
                    "--tabstop=2",
                    // Scripting
                    "--read0",
                ])
                .envs([
                    ("CLICOLOR", "1"),
                    ("CLICOLOR_FORCE", "1"),
                    ("FZF_DEFAULT_COMMAND", "zoxide edit reload"),
                ]);

                if cfg!(unix) {
                    // Non-POSIX args are only available on certain operating systems.
                    const PREVIEW_ARG: &str = if cfg!(target_os = "linux") {
                        r"--preview=\command -p ls -Cp --color=always --group-directories-first {2..}"
                    } else {
                        r"--preview=\command -p ls -Cp {2..}"
                    };
                    fzf.args([PREVIEW_ARG, "--preview-window=down,30%"]).env("SHELL", "sh");
                }

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
        write!(stdout, "{:>5}\t{}\x00", dir.score(now), &dir.path).unwrap();
    }
}
