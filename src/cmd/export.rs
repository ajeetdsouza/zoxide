use std::io::{self, Write};

use serde_json::to_writer_pretty;

use crate::cmd::{Export, ExportCommand, Run};
use crate::db::Database;
use crate::error::BrokenPipeHandler;


impl Run for Export {
    fn run(&self) -> anyhow::Result<()> {
        let db = Database::open()?;
        let dirs = db.dirs();
        let out = &mut io::stdout().lock();

        match &self.cmd {
            ExportCommand::Csv => {
                writeln!(out, "path,rank,last_accessed").pipe_exit("stdout")?;
                for dir in dirs {
                    writeln!(out, "{},{},{}", dir.path, dir.rank, dir.last_accessed)
                        .pipe_exit("stdout")?;
                }
            }
            ExportCommand::Json => {
                to_writer_pretty(&mut *out, dirs)?;
                writeln!(out).pipe_exit("stdout")?;
            }
            ExportCommand::Text => {
                for dir in dirs {
                    writeln!(out, "{}|{}|{}", dir.path, dir.rank, dir.last_accessed)
                        .pipe_exit("stdout")?;
                }
            }
        }

        Ok(())
    }
}
