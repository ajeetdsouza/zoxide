use crate::util;

use anyhow::Result;
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(about = "Remove a directory")]
pub struct Remove {
    #[structopt(required_unless("interactive"))]
    path: Option<PathBuf>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Remove {
    pub fn run(&self) -> Result<()> {
        if self.interactive {
            let mut db = util::get_db()?;
            let dirs = db.query_all();
            let now = util::get_current_time()?;

            if let Some(path_bytes) = util::fzf_helper(now, dirs)? {
                let path = util::bytes_to_path(&path_bytes)?;
                db.remove_exact(path)?;
            }

            Ok(())
        } else {
            // structopt guarantees that unwrap is safe here
            let path = self.path.as_ref().unwrap();
            util::get_db()?.remove(path)
        }
    }
}
