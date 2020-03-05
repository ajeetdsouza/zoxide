use crate::config::DB_PATH;
use crate::dir::Dir;
use crate::error::AppError;
use crate::types::Timestamp;
use failure::ResultExt;
use std::env;
use std::ffi::OsString;
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

pub fn get_db_path() -> Result<OsString, failure::Error> {
    let path = match env::var_os(DB_PATH) {
        Some(path) => path,
        None => {
            let mut path = dirs::home_dir().ok_or_else(|| AppError::GetHomeDirError)?;
            path.push(".zo");
            path.into_os_string()
        }
    };
    Ok(path)
}

pub fn process_query<'a, I: Iterator<Item = &'a str>>(keywords: I) -> Vec<String> {
    keywords.map(|keyword| keyword.to_ascii_lowercase()).collect()
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

    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.rank));

    for dir in dirs.iter() {
        // ensure that frecency fits in 4 characters
        let mut frecency = dir.rank;
        if frecency < 0 {
            frecency = 0;
        } else if frecency > 9999 {
            frecency = 9999;
        }

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
