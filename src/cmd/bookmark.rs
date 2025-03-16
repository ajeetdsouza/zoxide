use super::{Bookmark, Run};
use anyhow::Result;

impl Run for Bookmark {
    fn run(&self) -> Result<()> {}
}

impl Bookmark {}
