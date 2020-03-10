use crate::db::DB;
use crate::dir::Dir;
use crate::types::{Rank, Timestamp};
use anyhow::{anyhow, Context, Result};
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

pub fn get_zo_data() -> Result<PathBuf> {
    const ZO_DATA: &str = "_ZO_DATA";

    Ok(match env::var_os(ZO_DATA) {
        Some(path) => PathBuf::from(path),
        None => {
            let mut path =
                dirs::home_dir().ok_or_else(|| anyhow!("could not locate home directory"))?;
            path.push(".zo");
            path
        }
    })
}

pub fn get_zo_maxage() -> Result<Rank> {
    const ZO_MAXAGE: &str = "_ZO_MAXAGE";

    let maxage = match env::var_os(ZO_MAXAGE) {
        Some(maxage_var) => maxage_var
            .to_str()
            .ok_or_else(|| anyhow!("invalid Unicode in ${}", ZO_MAXAGE))?
            .parse::<i64>()
            .with_context(|| anyhow!("could not parse ${} as integer", ZO_MAXAGE))?
            as Rank,
        None => 1000.0,
    };
    Ok(maxage)
}

pub fn get_db() -> Result<DB> {
    let path = get_zo_data()?;
    DB::open(path)
}

pub fn get_current_time() -> Result<Timestamp> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .with_context(|| "system clock set to invalid time")?
        .as_secs();

    Ok(current_time as Timestamp)
}

pub fn fzf_helper(now: Timestamp, mut dirs: Vec<Dir>) -> Result<Option<String>> {
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

    Ok(output.get(12..).map(str::to_owned))
}
