use crate::util;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    query: Vec<String>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        if self.interactive {
            let mut db = util::get_db()?;
            let dirs = db.query_many(&self.query);
            let now = util::get_current_time()?;

            if let Some(path_bytes) = util::fzf_helper(now, dirs)? {
                let path = util::bytes_to_path(&path_bytes)?;
                db.remove_exact(path)?;
            }

            Ok(())
        } else {
            match self.query.as_slice() {
                [path] => util::get_db()?.remove(path),
                _ => clap::Error::with_description(
                    &format!(
                        "remove requires 1 value in non-interactive mode, but {} were provided",
                        self.query.len()
                    ),
                    clap::ErrorKind::WrongNumberOfValues,
                )
                .exit(),
            }
        }
    }
}
