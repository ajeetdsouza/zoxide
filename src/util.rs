use crate::config;
use crate::db::{Db, Epoch};

use anyhow::{Context, Result};

use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn get_db() -> Result<Db> {
    let data_dir = config::zo_data_dir()?;
    Db::open(data_dir)
}

pub fn get_current_time() -> Result<Epoch> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock set to invalid time")?
        .as_secs();

    Ok(current_time as _)
}

pub fn canonicalize<P: AsRef<Path>>(path: &P) -> Result<PathBuf> {
    let path = path.as_ref();
    dunce::canonicalize(path).with_context(|| format!("could not resolve path: {}", path.display()))
}

pub fn path_to_str<P: AsRef<Path>>(path: &P) -> Result<&str> {
    let path = path.as_ref();
    path.to_str()
        .with_context(|| format!("invalid utf-8 sequence in path: {}", path.display()))
}
