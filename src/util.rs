use crate::db::DB;
use crate::dir::Dir;
use crate::env::Env;
use crate::types::Epoch;
use anyhow::{anyhow, bail, Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::SystemTime;

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
        .with_context(|| "system clock set to invalid time")?
        .as_secs();

    Ok(current_time as Epoch)
}

pub fn fzf_helper(now: Epoch, mut dirs: Vec<Dir>) -> Result<Option<String>> {
    let mut fzf = Command::new("fzf")
        .arg("-n2..")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| anyhow!("could not launch fzf"))?;

    let fzf_stdin = fzf
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("could not connect to fzf stdin"))?;

    for dir in dirs.iter_mut() {
        dir.rank = dir.get_frecency(now);
    }

    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.rank as i64));

    for dir in dirs.iter() {
        // ensure that frecency fits in 4 characters
        let frecency = clamp(dir.rank, 0.0, 9999.0);
        writeln!(fzf_stdin, "{:>4.0}        {}", frecency, dir.path)
            .with_context(|| anyhow!("could not write into fzf stdin"))?;
    }

    let fzf_stdout = fzf
        .stdout
        .as_mut()
        .ok_or_else(|| anyhow!("could not connect to fzf stdout"))?;

    let mut output = String::new();
    fzf_stdout
        .read_to_string(&mut output)
        .with_context(|| anyhow!("could not read from fzf stdout"))?;

    let status = fzf.wait().with_context(|| "could not wait on fzf")?;

    match status.code() {
        // normal exit
        Some(0) => match output.get(12..) {
            Some(path) => Ok(Some(path.to_string())),
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
