use crate::db::Database;

use super::{Bookmark, Run};
use anyhow::Result;

impl Run for Bookmark {
    fn run(&self) -> Result<()> {
        let mut db = crate::db::Database::open()?;
        self.add_bookmark(&mut db).and(db.save())
    }
}

impl Bookmark {
    fn add_bookmark(&self, db: &mut Database) -> Result<()> {
        Ok(db.add_bookmark(self.bookmark_id.clone(), self.path.clone()))
    }
}
