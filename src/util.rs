use crate::db::DB;
use crate::dir::Dir;
use crate::env::Env;
use crate::types::Epoch;

use anyhow::{anyhow, bail, Context, Result};
use std::cmp::{Ordering, PartialOrd};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;

#[cfg(unix)]
pub fn path_to_bytes<P: AsRef<Path>>(path: &P) -> Option<&[u8]> {
    use std::os::unix::ffi::OsStrExt;
    Some(path.as_ref().as_os_str().as_bytes())
}

#[cfg(not(unix))]
pub fn path_to_bytes<P: AsRef<Path>>(path: &P) -> Option<&[u8]> {
    Some(path.as_ref().to_str()?.as_bytes())
}

pub fn get_db(env: &Env) -> Result<DB> {
    let path = env
        .data
        .as_ref()
        .ok_or_else(|| anyhow!("could not locate database file"))?;
    DB::open(path)
}

pub fn get_current_time() -> Result<Epoch> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock set to invalid time")?
        .as_secs();

    Ok(current_time as Epoch)
}

pub fn fzf_helper(now: Epoch, mut dirs: Vec<Dir>) -> Result<Option<Vec<u8>>> {
    let mut fzf = Command::new("fzf")
        .arg("-n2..")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("could not launch fzf")?;

    let fzf_stdin = fzf
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("could not connect to fzf stdin"))?;

    for dir in dirs.iter_mut() {
        dir.rank = dir.get_frecency(now);
    }

    dirs.sort_unstable_by(|dir1, dir2| {
        dir1.rank
            .partial_cmp(&dir2.rank)
            .unwrap_or(Ordering::Equal)
            .reverse()
    });

    for dir in dirs.iter() {
        // ensure that frecency fits in 4 characters
        let frecency = clamp(dir.rank, 0.0, 9999.0);

        if let Some(path_bytes) = path_to_bytes(&dir.path) {
            (|| {
                write!(fzf_stdin, "{:>4.0}        ", frecency)?;
                fzf_stdin.write_all(path_bytes)?;
                fzf_stdin.write_all(b"\n")
            })()
            .context("could not write into fzf stdin")?;
        }
    }

    let fzf_stdout = fzf
        .stdout
        .as_mut()
        .ok_or_else(|| anyhow!("could not connect to fzf stdout"))?;

    let mut buffer = Vec::new();
    fzf_stdout
        .read_to_end(&mut buffer)
        .context("could not read from fzf stdout")?;

    let status = fzf.wait().context("wait failed on fzf")?;
    match status.code() {
        // normal exit
        Some(0) => match buffer.get(12..buffer.len() - 1) {
            Some(path) => Ok(Some(path.to_vec())),
            None => bail!("fzf returned invalid output"),
        },

        // no match
        Some(1) => Ok(None),

        // error
        Some(2) => bail!("fzf returned an error"),

        // terminated by a signal
        Some(128..=254) | None => bail!("fzf was terminated"),

        // unknown
        _ => bail!("fzf returned an unknown error"),
    }
}

// FIXME: replace with f64::clamp once it is stable <https://github.com/rust-lang/rust/issues/44095>
#[must_use = "method returns a new number and does not mutate the original value"]
#[inline]
pub fn clamp(val: f64, min: f64, max: f64) -> f64 {
    assert!(min <= max);
    let mut x = val;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}
