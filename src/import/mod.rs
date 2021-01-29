mod autojump;
mod z;

use crate::db::Database;
use anyhow::Result;

use std::path::Path;

pub use autojump::Autojump;
pub use z::Z;

pub trait Import {
    fn import<P: AsRef<Path>>(&self, db: &mut Database, path: P) -> Result<()>;
}
