mod autojump;
mod z;

use crate::store::Store;
use anyhow::Result;

use std::path::Path;

pub use autojump::Autojump;
pub use z::Z;

pub trait Import {
    fn import<P: AsRef<Path>>(&self, store: &mut Store, path: P) -> Result<()>;
}
