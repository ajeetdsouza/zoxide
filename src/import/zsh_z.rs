use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::db::Dir;
use crate::import::{ImportError, Importer, z};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct ZshZ {}

impl Importer for ZshZ {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        let path = data_path()?;
        let file = File::open(&path).with_context(|| format!("could not read {path:?}"))?;
        let reader = BufReader::new(file);
        // zsh-z uses the same `path|rank|last_accessed` line format as z.
        Ok(z::Iter::new(reader, path))
    }
}

/// Mirrors zsh-z's path logic:
///
/// ```sh
/// # Allow the user to specify a custom datafile in $ZSHZ_DATA (or legacy $_Z_DATA)
/// local custom_datafile="${ZSHZ_DATA:-$_Z_DATA}"
/// # If the user specified a datafile, use that or default to ~/.z
/// local datafile=${${custom_datafile:-$HOME/.z}:A}
/// ```
fn data_path() -> Result<PathBuf> {
    match env::var_os("ZSHZ_DATA").or_else(|| env::var_os("_Z_DATA")) {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            let mut path = dirs::home_dir().context("could not find home directory")?;
            path.push(".z");
            Ok(path)
        }
    }
}
