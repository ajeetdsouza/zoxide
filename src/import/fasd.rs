use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::db::Dir;
use crate::import::{ImportError, Importer, z};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct Fasd {}

impl Importer for Fasd {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        let path = data_path()?;
        let file = File::open(&path).with_context(|| format!("could not read {path:?}"))?;
        let reader = BufReader::new(file);
        // fasd uses the same `path|rank|last_accessed` line format as z, so reuse z's iterator.
        Ok(z::Iter::new(reader, path))
    }
}

/// Mirrors fasd's path logic:
///
/// ```sh
/// [ -z "$_FASD_DATA" ] && _FASD_DATA="$HOME/.fasd"
/// ```
fn data_path() -> Result<PathBuf> {
    match env::var_os("_FASD_DATA") {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            let mut path = dirs::home_dir().context("could not find home directory")?;
            path.push(".fasd");
            Ok(path)
        }
    }
}
