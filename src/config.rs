use crate::types::Rank;

use anyhow::{bail, Context, Result};

use std::env;
use std::fs;
use std::path::PathBuf;

pub fn zo_data() -> Result<PathBuf> {
    let path = match env::var_os("_ZO_DATA") {
        Some(data_osstr) => PathBuf::from(data_osstr),
        None => {
            if let Some(mut cache_dir) = dirs::cache_dir() {
                cache_dir.push("zoxide");
                cache_dir
            } else if let Some(mut home_dir) = dirs::home_dir() {
                home_dir.push(".zoxide");
                home_dir
            } else {
                bail!("could not generate default directory, please set _ZO_DATA manually");
            }
        }
    };

    fs::create_dir_all(&path).context("could not create _ZO_DATA directory")?;
    Ok(path)
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
