use std::io::{self, Write};

use anyhow::Result;

use crate::cmd::{Edit, EditCommand, Run};
use crate::db::{Database, Epoch};
use crate::error::BrokenPipeHandler;
use crate::util::{self, Fz};

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
                print_dirs(db, now)
            }
            None => {
                db.sort_by_score(now);
                db.save()?;

                let mut fzf = Fz::new()?;
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
                    "--border-label=  zoxide-edit  ",
                    "--header=\
ctrl-r:reload   \tctrl-w:delete
ctrl-a:increment\tctrl-d:decrement

 SCORE\tPATH",
                    "--info=inline",
                    "--layout=reverse",
                    "--padding=1,0,0,0",
                    // Display
                    "--color=label:bold",
                    "--tabstop=1",
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

                let mut fzf = fzf.spawn()?;
                fzf.wait()
            }
        }
    }
}

fn print_dirs(db: &Database, now: Epoch) -> Result<()> {
    let stdout = &mut io::stdout().lock();
    for dir in db.dirs().iter().rev() {
        let score = dir.score(now).clamp(0.0, 9999.0);
        write!(stdout, "{:>6.1}\t{}\x00", score, &dir.path).pipe_exit("fzf")?;
    }
    Ok(())
}
