use crate::db::DB;
use crate::dir::Dir;
use crate::env::Env;
use crate::types::Epoch;
use anyhow::{anyhow, Context, Result};
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
    let fzf = Command::new("fzf")
        .arg("-n2..")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| anyhow!("could not launch fzf"))?;

    let mut fzf_stdin = fzf
        .stdin
        .ok_or_else(|| anyhow!("could not connect to fzf stdin"))?;

    for dir in dirs.iter_mut() {
        dir.rank = dir.get_frecency(now);
    }

    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.rank as i64));

    for dir in dirs.iter() {
        // ensure that frecency fits in 4 characters
        let frecency = if dir.rank > 9999.0 {
            9999
        } else if dir.rank > 0.0 {
            dir.rank as i32
        } else {
            0
        };

        writeln!(fzf_stdin, "{:>4}        {}", frecency, dir.path)
            .with_context(|| anyhow!("could not write into fzf stdin"))?;
    }

    let mut fzf_stdout = fzf
        .stdout
        .ok_or_else(|| anyhow!("could not connect to fzf stdout"))?;

    let mut output = String::new();
    fzf_stdout
        .read_to_string(&mut output)
        .with_context(|| anyhow!("could not read from fzf stdout"))?;

    Ok(output.get(12..).map(str::to_string))
}
