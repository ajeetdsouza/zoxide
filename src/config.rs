use crate::db::Rank;

use anyhow::{bail, Context, Result};

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

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
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("could not create data directory: {}", data_dir.display()))?;

    Ok(data_dir)
}

pub fn zo_exclude_dirs() -> Result<Vec<glob::Pattern>> {
    match env::var_os("_ZO_EXCLUDE_DIRS") {
        Some(dirs_osstr) => env::split_paths(&dirs_osstr)
            .map(|path| {
                let pattern = path
                    .to_str()
                    .context("invalid utf-8 sequence in _ZO_EXCLUDE_DIRS")?;
                glob::Pattern::new(&pattern)
                    .with_context(|| format!("invalid glob in _ZO_EXCLUDE_DIRS: {}", pattern))
            })
            .collect(),
        None => Ok(Vec::new()),
    }
}

pub fn zo_fzf_opts() -> Option<OsString> {
    env::var_os("_ZO_FZF_OPTS")
}

pub fn zo_maxage() -> Result<Rank> {
    match env::var_os("_ZO_MAXAGE") {
        Some(maxage_osstr) => {
            let maxage_str = maxage_osstr
                .to_str()
                .context("invalid utf-8 sequence in _ZO_MAXAGE")?;
            let maxage = maxage_str.parse::<u64>().with_context(|| {
                format!("unable to parse _ZO_MAXAGE as integer: {}", maxage_str)
            })?;
            Ok(maxage as Rank)
        }
        None => Ok(10000.0),
    }
}

pub fn zo_echo() -> bool {
    match env::var_os("_ZO_ECHO") {
        Some(var) => var == "1",
        None => false,
    }
}

pub fn zo_resolve_symlinks() -> bool {
    match env::var_os("_ZO_RESOLVE_SYMLINKS") {
        Some(var) => var == "1",
        None => false,
    }
}
