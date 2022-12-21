use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};
use glob::Pattern;

use crate::db::Rank;

pub fn data_dir() -> Result<PathBuf> {
    let path = match env::var_os("_ZO_DATA_DIR") {
        Some(path) => PathBuf::from(path),
        None => dirs::data_local_dir()
            .context("could not find data directory, please set _ZO_DATA_DIR manually")?
            .join("zoxide"),
    };
    Ok(path)
}

pub fn echo() -> bool {
    env::var_os("_ZO_ECHO").map_or(false, |var| var == "1")
}

pub fn exclude_dirs() -> Result<Vec<Pattern>> {
    match env::var_os("_ZO_EXCLUDE_DIRS") {
        Some(paths) => env::split_paths(&paths)
            .map(|path| {
                let pattern = path.to_str().context("invalid unicode in _ZO_EXCLUDE_DIRS")?;
                Pattern::new(pattern).with_context(|| format!("invalid glob in _ZO_EXCLUDE_DIRS: {pattern}"))
            })
            .collect(),
        None => {
            let pattern = (|| {
                let home = dirs::home_dir()?;
                let home = Pattern::escape(home.to_str()?);
                Pattern::new(&home).ok()
            })();
            Ok(pattern.into_iter().collect())
        }
    }
}

pub fn fzf_opts() -> Option<OsString> {
    env::var_os("_ZO_FZF_OPTS")
}

pub fn maxage() -> Result<Rank> {
    env::var_os("_ZO_MAXAGE").map_or(Ok(10_000.0), |maxage| {
        let maxage = maxage.to_str().context("invalid unicode in _ZO_MAXAGE")?;
        let maxage =
            maxage.parse::<u32>().with_context(|| format!("unable to parse _ZO_MAXAGE as integer: {maxage}"))?;
        Ok(maxage as Rank)
    })
}

pub fn resolve_symlinks() -> bool {
    env::var_os("_ZO_RESOLVE_SYMLINKS").map_or(false, |var| var == "1")
}
