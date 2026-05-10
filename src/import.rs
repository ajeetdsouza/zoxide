mod atuin;
mod autojump;
mod fasd;
mod z;
mod z_lua;
mod zsh_z;

pub(crate) use atuin::Atuin;
pub(crate) use autojump::Autojump;
pub(crate) use fasd::Fasd;
pub(crate) use z::Z;
pub(crate) use z_lua::ZLua;
pub(crate) use zsh_z::ZshZ;

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;

use crate::config;
use crate::db::{Database, Dir};

pub(crate) trait Importer {
    /// Yields directory entries to be imported.
    ///
    /// The outer `Result` reports failure to fetch the input (e.g. missing
    /// file, subprocess errored). The per-item `Result` reports a malformed
    /// row, which doesn't necessarily abort the whole import.
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>>;
}

/// A single record that failed to import.
#[derive(Debug)]
pub(crate) struct ImportError {
    /// Path of the source file containing the offending record. `None` if the
    /// importer is not file-based (e.g. atuin streams from a subprocess).
    pub path: Option<PathBuf>,

    /// 1-indexed line number of the offending input.
    pub line_num: usize,

    /// Underlying reason the record could not be imported.
    pub source: anyhow::Error,
}

/// Drives a single importer end-to-end: writes each `Ok` dir into the
/// database and prints each `Err` to stderr in `<path>:<line>: <reason>`
/// format. Doesn't abort on per-record errors — bad rows are skipped, the
/// rest of the import continues. After the iteration completes successfully,
/// the database is deduplicated and aged.
pub(crate) fn run<I: Importer>(importer: &I, db: &mut Database) -> Result<()> {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    for entry in importer.dirs()? {
        match entry {
            Ok(dir) => db.add_unchecked(dir.path, dir.rank, dir.last_accessed),
            Err(e) => {
                let location = match &e.path {
                    Some(path) => format!("{}:{}", path.display(), e.line_num),
                    None => format!("line {}", e.line_num),
                };
                let _ = writeln!(stderr, "{location}: {:#}", e.source);
            }
        }
    }

    if db.dirty() {
        db.dedup();
        let max_age = config::maxage()?;
        db.age(max_age);
    }

    Ok(())
}
