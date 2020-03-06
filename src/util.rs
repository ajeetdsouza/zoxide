use crate::dir::Dir;
use crate::error::AppError;
use crate::types::Timestamp;
use failure::ResultExt;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::SystemTime;

pub fn get_current_time() -> Result<Timestamp, failure::Error> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .with_context(|_| AppError::SystemTimeError)?
        .as_secs();

    Ok(current_time as Timestamp)
}

pub fn fzf_helper(now: Timestamp, mut dirs: Vec<Dir>) -> Result<Option<String>, failure::Error> {
    let fzf = Command::new("fzf")
        .arg("-n2..")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|_| AppError::FzfLaunchError)?;

    let mut fzf_stdin = fzf.stdin.ok_or_else(|| AppError::FzfIoError)?;

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
            .with_context(|_| AppError::FzfIoError)?;
    }

    let mut fzf_stdout = fzf.stdout.ok_or_else(|| AppError::FzfIoError)?;

    let mut output = String::new();
    fzf_stdout
        .read_to_string(&mut output)
        .with_context(|_| AppError::FzfIoError)?;

    Ok(output.get(12..).map(str::to_owned))
}
