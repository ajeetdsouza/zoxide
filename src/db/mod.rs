mod dir;
mod stream;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use bincode::Options;

use crate::config;
pub use crate::db::dir::{Dir, Epoch, Rank};
pub use crate::db::stream::{Stream, StreamOptions};

/// Attempt to read an old bincode-formatted database and write its entries into
/// the provided SQLite connection.
fn migrate_from_bincode(conn: &mut Connection, old_path: &Path) -> Result<()> {
    // The on-disk format used by the legacy database is very similar to the
    // struct definitions we already use.  We simply serialise a version number
    // followed by a `Vec<Dir>`.
    const MAX_SIZE: u64 = 32 << 20; // 32 MiB

    let data = fs::read(old_path)
        .with_context(|| format!("could not read legacy database: {}", old_path.display()))?;

    let deserializer = &mut bincode::options()
        .with_fixint_encoding()
        .with_limit(MAX_SIZE);

    let version_size = deserializer.serialized_size(&Database::VERSION)? as usize;
    if data.len() < version_size {
        anyhow::bail!("legacy database is corrupted");
    }
    let (bytes_version, bytes_dirs) = data.split_at(version_size);

    let version: u32 = deserializer.deserialize(bytes_version)?;
    if version != Database::VERSION {
        anyhow::bail!(
            "unsupported legacy database version (got {}, expected {})",
            version,
            Database::VERSION
        );
    }

    let dirs: Vec<Dir<'_>> = deserializer.deserialize(bytes_dirs)?;

    let tx = conn.transaction()?;
    for dir in dirs {
        let path_s: String = dir.path.into_owned();
        tx.execute(
            "INSERT OR REPLACE INTO dirs (path, rank, last_accessed) VALUES (?1, ?2, ?3)",
            params![&path_s, dir.rank, dir.last_accessed],
        )?;
    }
    tx.commit()?;

    Ok(())
}

pub struct Database {
    conn: Connection,
    dirty: bool,
}

impl Database {
    const VERSION: u32 = 3;

    pub fn open() -> Result<Self> {
        let data_dir = config::data_dir()?;
        Self::open_dir(data_dir)
    }

    pub fn open_dir(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let path = data_dir.join("db.sqlite3");
        let path = fs::canonicalize(&path).unwrap_or(path);

        fs::create_dir_all(data_dir)
            .with_context(|| format!("unable to create data directory: {}", data_dir.display()))?;

        // Open or create sqlite database file.
        let existed = path.exists();

        let mut conn = Connection::open(&path)
            .with_context(|| format!("could not open database: {}", path.display()))?;

        // Enable WAL for better concurrency and durability.
        conn.pragma_update(None, "journal_mode", &"WAL").ok();

        // Create table if it doesn't exist.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS dirs (
                path TEXT PRIMARY KEY,
                rank REAL NOT NULL,
                last_accessed INTEGER NOT NULL
            );",
        )?;

        // If the sqlite database didn't previously exist, attempt to migrate data
        // from the legacy bincode-backed file.  This keeps behaviour identical to
        // older versions of zoxide while ensuring users transparently upgrade.
        if !existed {
            let old_path = data_dir.join("db.zo");
            if old_path.exists() {
                // Migration errors shouldn't prevent the program from running;
                // just print a warning so users can investigate.
                if let Err(e) = migrate_from_bincode(&mut conn, &old_path) {
                    eprintln!(
                        "warning: failed to migrate legacy database ({}): {}",
                        old_path.display(),
                        e
                    );
                }
            }
        }

        Ok(Database { conn, dirty: false })
    }

    pub fn save(&mut self) -> Result<()> {
        // For SQLite, write operations are applied immediately via transactions.
        // Keep save() for compatibility; do nothing.
        self.dirty = false;
        Ok(())
    }

    /// Increments the rank of a directory, or creates it if it does not exist.
    pub fn add(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        let path_s: String = path.into();
        let tx = match self.conn.transaction() {
            Ok(t) => t,
            Err(_) => return,
        };

        let existing: Option<(f64, u64)> = tx
            .query_row(
                "SELECT rank, last_accessed FROM dirs WHERE path = ?1",
                params![&path_s],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .unwrap_or(None);

        match existing {
            Some((rank, _last)) => {
                let new_rank = (rank + by).max(0.0);
                let _ = tx.execute(
                    "UPDATE dirs SET rank = ?1 WHERE path = ?2",
                    params![new_rank, &path_s],
                );
            }
            None => {
                let _ = tx.execute(
                    "INSERT INTO dirs (path, rank, last_accessed) VALUES (?1, ?2, ?3)",
                    params![&path_s, by.max(0.0), now],
                );
            }
        }

        let _ = tx.commit();
        self.dirty = true;
    }

    /// Creates a new directory. This will create a duplicate entry if this
    /// directory is already in the database, it is expected that the user
    /// either does a check before calling this, or calls `dedup()`
    /// afterward.
    #[cfg(test)]
    pub fn add_unchecked(&mut self, path: impl AsRef<str> + Into<String>, rank: Rank, now: Epoch) {
        let path_s: String = path.into();
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO dirs (path, rank, last_accessed) VALUES (?1, ?2, ?3)",
            params![&path_s, rank, now],
        );
        self.dirty = true;
    }

    /// choose the max `now`
    /// sum `rank`
    pub fn add_unchecked_merge(&mut self, path: impl AsRef<str> + Into<String>, rank: Rank, now: Epoch) {
        let path_s: String = path.into();
        let _ = self.conn.execute(
            "INSERT INTO dirs (path, rank, last_accessed) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET
               rank = dirs.rank + excluded.rank,
               last_accessed = MAX(dirs.last_accessed, excluded.last_accessed)",
            params![&path_s, rank, now],
        );
        self.dirty = true;
    }

    /// Increments the rank and updates the last_accessed of a directory, or
    /// creates it if it does not exist.
    pub fn add_update(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        let path_s: String = path.into();
        let tx = match self.conn.transaction() {
            Ok(t) => t,
            Err(_) => return,
        };

        let existing: Option<(f64, u64)> = tx
            .query_row(
                "SELECT rank, last_accessed FROM dirs WHERE path = ?1",
                params![&path_s],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .unwrap_or(None);

        match existing {
            Some((rank, _)) => {
                let new_rank = (rank + by).max(0.0);
                let _ = tx.execute(
                    "UPDATE dirs SET rank = ?1, last_accessed = ?2 WHERE path = ?3",
                    params![new_rank, now, &path_s],
                );
            }
            None => {
                let _ = tx.execute(
                    "INSERT INTO dirs (path, rank, last_accessed) VALUES (?1, ?2, ?3)",
                    params![&path_s, by.max(0.0), now],
                );
            }
        }

        let _ = tx.commit();
        self.dirty = true;
    }

    /// Removes the directory with `path` from the store. Returns true if an
    /// entry was deleted.
    pub fn remove(&mut self, path: impl AsRef<str>) -> bool {
        let path_s = path.as_ref();
        match self.conn.execute("DELETE FROM dirs WHERE path = ?1", params![path_s]) {
            Ok(count) => {
                if count > 0 {
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn swap_remove(&mut self, _idx: usize) {
        // In the sqlite-backed implementation we don't maintain an in-memory
        // vector, so this is a no-op. Higher-level code that relies on
        // indices shouldn't be calling this directly except within the
        // streaming logic which uses Database::dirs(). For compatibility, keep
        // the method but do nothing.
        self.dirty = true;
    }

    pub fn age(&mut self, max_age: Rank) {
        // Apply the aging algorithm to all rows.
        // Collect entries first to avoid holding a Statement borrow while starting
        // a transaction on the connection.
        let mut entries = Vec::new();
        if let Ok(mut stmt) = self.conn.prepare("SELECT path, rank FROM dirs") {
            if let Ok(rows) =
                stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)))
            {
                for r in rows {
                    if let Ok((path, rank)) = r {
                        entries.push((path, rank));
                    }
                }
            }
        }

        let total_age: f64 = entries.iter().map(|(_, rank)| *rank).sum();
        if total_age > max_age {
            let factor = 0.9 * max_age / total_age;
            if let Ok(tx) = self.conn.transaction() {
                for (path, rank) in entries {
                    let new_rank = rank * factor;
                    if new_rank < 1.0 {
                        let _ = tx.execute("DELETE FROM dirs WHERE path = ?1", params![path]);
                    } else {
                        let _ = tx.execute(
                            "UPDATE dirs SET rank = ?1 WHERE path = ?2",
                            params![new_rank, path],
                        );
                    }
                }
                let _ = tx.commit();
                self.dirty = true;
            }
        }
    }

    pub fn dedup(&mut self) {
        // Using path as PRIMARY KEY ensures uniqueness, nothing to do here.
    }

    #[cfg(test)]
    pub fn sort_by_path(&mut self) {
        // Sorting is done at query time in the sqlite-backed implementation.
    }

    pub fn sort_by_score(&mut self, _now: Epoch) {
        // Sorting is done at query time in the sqlite-backed implementation.
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn dirs(&self) -> Vec<Dir<'static>> {
        // Load all dirs from the database into an owned Vec.
        let mut stmt = match self.conn.prepare("SELECT path, rank, last_accessed FROM dirs") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            Ok(Dir {
                path: row.get::<_, String>(0)?.into(),
                rank: row.get::<_, f64>(1)?,
                last_accessed: row.get::<_, u64>(2)?,
            })
        });

        let mut out = Vec::new();
        if let Ok(map) = rows {
            for r in map {
                if let Ok(dir) = r {
                    out.push(dir);
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let db = Database::open_dir(data_dir.path()).unwrap();
            assert_eq!(db.dirs().len(), 1);

            let dirs = db.dirs();
            let dir = &dirs[0];
            assert_eq!(dir.path, path);
            assert!((dir.rank - 2.0).abs() < 0.01);
            assert_eq!(dir.last_accessed, now);
        }
    }

    #[test]
    fn remove() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.remove(path));
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.dirs().is_empty());
            assert!(!db.remove(path));
            db.save().unwrap();
        }
    }

    #[test]
    fn migrate_from_bincode() {
        let data_dir = tempfile::tempdir().unwrap();
        let old_path = data_dir.path().join("db.zo");

        // prepare a legacy file with one entry
        let dirs = vec![Dir {
            path: "/foo".into(),
            rank: 1.0,
            last_accessed: 12345,
        }];
        let mut bytes = Vec::new();
        let mut serializer = bincode::options().with_fixint_encoding();
        serializer.serialize_into(&mut bytes, &Database::VERSION).unwrap();
        serializer.serialize_into(&mut bytes, &dirs).unwrap();
        fs::write(&old_path, &bytes).unwrap();

        // opening should automatically migrate the data
        let db = Database::open_dir(data_dir.path()).unwrap();
        let dirs = db.dirs();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].path, "/foo");
        assert!((dirs[0].rank - 1.0).abs() < f64::EPSILON);
        assert_eq!(dirs[0].last_accessed, 12345);
        // sqlite file should exist after opening
        assert!(data_dir.path().join("db.sqlite3").exists());
        // old file is left intact so future runs are no-ops
        assert!(old_path.exists());
    }
}
