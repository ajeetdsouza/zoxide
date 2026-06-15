use std::io::{self, Read, Write};

use anyhow::{Result, bail};

use crate::cmd::{Rename, Run};
use crate::db::{Database, Epoch, Rank};
use crate::error::BrokenPipeHandler;

struct Data {
    pub old_path: String,
    pub new_path: String,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Run for Rename {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        let mut to_edit = Vec::<Data>::new();

        for path in db.dirs() {
            let old = path.display().to_string();
            let new = path.display().to_string().replace(&self.old_name, &self.new_name);

            if new != old {
                writeln!(io::stdout(), "{old} -> {new}")
                    .pipe_exit("stdout")
                    .expect("cannot write to stdout.");

                to_edit.push(Data {
                    old_path: old,
                    new_path: new,
                    rank: path.rank,
                    last_accessed: path.last_accessed,
                });
            }
        }

        for item in to_edit.iter() {
            db.remove(&item.old_path);
            db.add(&item.new_path, item.rank, item.last_accessed);
        }

        if to_edit.is_empty() {
            bail!("No entries to rename.");
        }

        if self.force {
            return db.save();
        }

        writeln!(io::stdout(), "Rename {} entries? (y/n)", to_edit.len())
            .pipe_exit("stdout")
            .expect("cannot write to stdout.");

        let mut buf = [0];
        std::io::stdin().read_exact(&mut buf).expect("input expected");
        match buf[0] as char {
            'y' | 'Y' => db.save(),
            _ => bail!("Rename aborted."),
        }
    }
}
