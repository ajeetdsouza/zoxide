use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::db::Dir;
use crate::import::{ImportError, Importer, z};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct ZLua {}

impl Importer for ZLua {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        let path = data_path()?;
        let err = match File::open(&path) {
            Ok(file) => return Ok(z::Iter::new(BufReader::new(file), path)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => e,
            Err(e) => return Err(e).with_context(|| format!("could not read {path:?}")),
        };

        let fish_path = data_path_fish()?;
        let file = match File::open(&fish_path) {
            Ok(file) => file,
            // Both paths missing - report the original path's error.
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(err).with_context(|| format!("could not read {path:?}"));
            }
            // Fish path failed for some other reason (permissions, etc.)
            Err(e) => return Err(e).with_context(|| format!("could not read {fish_path:?}")),
        };
        // z.lua uses the same `path|rank|last_accessed` line format as z.
        Ok(z::Iter::new(BufReader::new(file), fish_path))
    }
}

/// Mirrors z.lua's path logic:
///
/// ```lua
/// DATA_FILE = '~/.zlua'    -- default
///
/// -- in z_init():
/// local _zl_data = os.getenv('_ZL_DATA')
/// if _zl_data ~= nil and _zl_data ~= "" then
///     if windows then
///         DATA_FILE = _zl_data
///     else
///         -- avoid windows environments affect cygwin & msys
///         if not string.match(_zl_data, '^%a:[/\\]') then
///             DATA_FILE = _zl_data
///         end
///     end
/// end
/// ```
fn data_path() -> Result<PathBuf> {
    if let Some(path) = env::var_os("_ZL_DATA")
        // Skip empty paths.
        .filter(|path| !path.is_empty())
        // On non-Windows, skip values that look like a Windows path (`C:\...`)
        // — guards against Cygwin/MSYS environments leaking through.
        .filter(|path| cfg!(target_os = "windows") || !looks_like_windows_path(path))
    {
        return Ok(PathBuf::from(path));
    }

    let mut path = dirs::home_dir().context("could not find home directory")?;
    path.push(".zlua");

    Ok(path)
}

/// Mirrors z.lua's path logic on Fish:
///
/// ```fish
/// if test -z "$XDG_DATA_HOME"
///     set -U _ZL_DATA_DIR "$HOME/.local/share/zlua"
/// else
///     set -U _ZL_DATA_DIR "$XDG_DATA_HOME/zlua"
/// end
/// set -x _ZL_DATA "$_ZL_DATA_DIR/zlua.txt"
/// ```
fn data_path_fish() -> Result<PathBuf> {
    let mut path = match env::var_os("XDG_DATA_HOME") {
        Some(xdg) => PathBuf::from(xdg),
        None => {
            let mut path = dirs::home_dir().context("could not find home directory")?;
            path.push(".local");
            path.push("share");
            path
        }
    };

    path.push("zlua");
    path.push("zlua.txt");

    Ok(path)
}

/// Matches Lua's `^%a:[/\\]` — ASCII letter, colon, slash-or-backslash.
fn looks_like_windows_path(s: &OsStr) -> bool {
    let bytes = s.as_encoded_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}
