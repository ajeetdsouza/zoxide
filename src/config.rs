use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use dirs;
use glob::Pattern;

use crate::db::Rank;

pub fn data_dir() -> Result<PathBuf> {
    let path = match env::var_os("_ZO_DATA_DIR") {
        Some(path) => PathBuf::from(path),
        None => match dirs::data_local_dir() {
            Some(mut path) => {
                path.push("zoxide");
                path
            }
            None => bail!("could not find data directory, please set _ZO_DATA_DIR manually"),
        },
    };

    Ok(path)
}

pub fn echo() -> bool {
    match env::var_os("_ZO_ECHO") {
        Some(var) => var == "1",
        None => false,
    }
}

pub fn exclude_dirs() -> Result<Vec<Pattern>> {
    env::var_os("_ZO_EXCLUDE_DIRS").map_or_else(
        || {
            let pattern = (|| {
                let home = dirs::home_dir()?;
                let home = home.to_str()?;
                let home = Pattern::escape(home);
                Pattern::new(&home).ok()
            })();
            Ok(pattern.into_iter().collect())
        },
        |paths| {
            env::split_paths(&paths)
                .map(|path| {
                    let pattern = path.to_str().context("invalid unicode in _ZO_EXCLUDE_DIRS")?;
                    Pattern::new(pattern).with_context(|| format!("invalid glob in _ZO_EXCLUDE_DIRS: {}", pattern))
                })
                .collect()
        },
    )
}

pub fn fzf_opts() -> Option<OsString> {
    env::var_os("_ZO_FZF_OPTS")
}

pub fn maxage() -> Result<Rank> {
    match env::var_os("_ZO_MAXAGE") {
        Some(maxage) => {
            let maxage = maxage.to_str().context("invalid unicode in _ZO_MAXAGE")?;
            let maxage =
                maxage.parse::<u64>().with_context(|| format!("unable to parse _ZO_MAXAGE as integer: {}", maxage))?;
            Ok(maxage as Rank)
        }
        None => Ok(10000.0),
    }
}

pub fn resolve_symlinks() -> bool {
    match env::var_os("_ZO_RESOLVE_SYMLINKS") {
        Some(var) => var == "1",
        None => false,
    }
}
