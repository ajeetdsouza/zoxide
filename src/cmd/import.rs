use anyhow::{Result, bail};

use crate::cmd::{Import, ImportFrom, Run};
use crate::db::Database;
use crate::import;

impl Run for Import {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;
        if !self.merge && !db.dirs().is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        match self.from {
            ImportFrom::Atuin => import::run(&import::Atuin {}, &mut db)?,
            ImportFrom::Autojump => import::run(&import::Autojump {}, &mut db)?,
            ImportFrom::Fasd => import::run(&import::Fasd {}, &mut db)?,
            ImportFrom::Z => import::run(&import::Z {}, &mut db)?,
            ImportFrom::ZLua => import::run(&import::ZLua {}, &mut db)?,
            ImportFrom::ZshZ => import::run(&import::ZshZ {}, &mut db)?,
        }

        db.save()
    }
}
