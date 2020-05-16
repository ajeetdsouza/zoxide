use crate::config;
use crate::db::{Db, Epoch};

use anyhow::{Context, Result};

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

    Ok(current_time as Epoch)
}
