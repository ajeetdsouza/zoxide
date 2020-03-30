use crate::db::DBVersion;
use crate::dir::Rank;

use anyhow::{bail, Context, Result};

use std::env;
use std::fs;
use std::path::PathBuf;

pub const DB_MAX_SIZE: u64 = 8 * 1024 * 1024; // 8 MiB
pub const DB_VERSION: DBVersion = 3;

pub fn zo_data_dir() -> Result<PathBuf> {
    let data_dir = match env::var_os("_ZO_DATA_DIR") {
        Some(data_osstr) => PathBuf::from(data_osstr),
        None => match dirs::data_local_dir() {
            Some(mut data_dir) => {
                data_dir.push("zoxide");
                data_dir
            }
            None => bail!("could not find database directory, please set _ZO_DATA_DIR manually"),
        },
    };

    // This will fail when `data_dir` points to a file or a broken symlink, but
    // will no-op on a valid symlink (to a directory), or an actual directory.
    fs::create_dir_all(&data_dir).context("could not create data directory")?;

    Ok(data_dir)
}

pub fn zo_exclude_dirs() -> Vec<PathBuf> {
    match env::var_os("_ZO_EXCLUDE_DIRS") {
        Some(dirs_osstr) => env::split_paths(&dirs_osstr).collect(),
        None => Vec::new(),
    }
}

pub fn zo_maxage() -> Result<Rank> {
    match env::var_os("_ZO_MAXAGE") {
        Some(maxage_osstr) => match maxage_osstr.to_str() {
            Some(maxage_str) => {
                let maxage = maxage_str
                    .parse::<i64>()
                    .context("unable to parse _ZO_MAXAGE as integer")?;

                Ok(maxage as Rank)
            }
            None => bail!("invalid Unicode in _ZO_MAXAGE"),
        },
        None => Ok(1000.0),
    }
}
