use crate::util;
use anyhow::Result;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Search for a directory")]
pub struct Query {
    keywords: Vec<String>,
    #[structopt(short, long, help = "Opens an interactive selection menu using fzf")]
    interactive: bool,
}

impl Query {
    pub fn run(mut self) -> Result<()> {
        let path_opt = if self.interactive {
            self.query_interactive()
        } else {
            self.query()
        }?;

        if let Some(path) = path_opt {
            println!("query: {}", path.trim());
        }

        Ok(())
    }

    fn query(&mut self) -> Result<Option<String>> {
        let now = util::get_current_time()?;
        let mut db = util::get_db()?;

        if let [path] = self.keywords.as_slice() {
            if Path::new(path).is_dir() {
                return Ok(Some(path.to_string()));
            }
        }

        for keyword in &mut self.keywords {
            keyword.make_ascii_lowercase();
        }

        if let Some(dir) = db.query(&self.keywords, now) {
            return Ok(Some(dir.path));
        }

        Ok(None)
    }

    fn query_interactive(&mut self) -> Result<Option<String>> {
        let now = util::get_current_time()?;

        for keyword in &mut self.keywords {
            keyword.make_ascii_lowercase();
        }

        let dirs = util::get_db()?.query_all(&self.keywords);
        util::fzf_helper(now, dirs)
    }
}
