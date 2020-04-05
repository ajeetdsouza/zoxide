use crate::config;
use crate::db::DB;
use crate::dir::{Dir, Epoch};
use crate::error::SilentExit;

use anyhow::{anyhow, bail, Context, Result};

use std::cmp::Reverse;
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

pub fn get_db() -> Result<DB> {
    let mut db_path = config::zo_data_dir()?;
    db_path.push("db.zo");

    // FIXME: fallback to old database location; remove in next breaking version
    if !db_path.is_file() {
        if let Some(mut old_db_path) = dirs::home_dir() {
            old_db_path.push(".zo");

            if old_db_path.is_file() {
                return DB::open_and_migrate(old_db_path, db_path);
            }
        }
    }

    DB::open(db_path)
}

pub fn get_current_time() -> Result<Epoch> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock set to invalid time")?
        .as_secs();

    Ok(current_time as Epoch)
}

pub fn fzf_helper<'a, I>(now: Epoch, dirs: I) -> Result<Option<Vec<u8>>>
where
    I: IntoIterator<Item = &'a Dir>,
{
    let mut fzf = Command::new("fzf")
        .args(&["-n2..", "--no-sort"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("could not launch fzf")?;

    let fzf_stdin = fzf
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("could not connect to fzf stdin"))?;

    let mut dir_frecencies = dirs
        .into_iter()
        .map(|dir| (dir, clamp(dir.get_frecency(now), 0.0, 9999.0) as i32))
        .collect::<Vec<_>>();

    dir_frecencies.sort_unstable_by_key(|&(dir, frecency)| Reverse((frecency, &dir.path)));

    for &(dir, frecency) in dir_frecencies.iter() {
        // ensure that frecency fits in 4 characters
        if let Some(path_bytes) = path_to_bytes(&dir.path) {
            (|| {
                write!(fzf_stdin, "{:>4}        ", frecency)?;
                fzf_stdin.write_all(path_bytes)?;
                writeln!(fzf_stdin)
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
        Some(code @ 130) => bail!(SilentExit { code }),
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

    if val > max {
        max
    } else if val > min {
        val
    } else {
        min
    }
}
