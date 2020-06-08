use crate::config;
use crate::db::{Db, Epoch};

use anyhow::{bail, Context, Result};
use std::env;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

pub fn get_db() -> Result<Db> {
    let data_dir = config::zo_data_dir()?;
    Db::open(data_dir)
}

pub fn get_current_time() -> Result<Epoch> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock set to invalid time")?
        .as_secs();

    Ok(current_time as _)
}

pub fn canonicalize<P: AsRef<Path>>(path: &P) -> Result<PathBuf> {
    let path = path.as_ref();
    dunce::canonicalize(path).with_context(|| format!("could not resolve path: {}", path.display()))
}

/// Resolves the absolute version of a path.
///
/// If path is already absolute, the path is still processed to be cleaned, as it can contained ".." or "." (or other)
/// character.
/// If path is relative, use the current directory to build the absolute path.
#[cfg(any(unix, windows))]
pub fn resolve_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    let base_path;

    let mut components = path.components().peekable();
    let mut stack = Vec::new();

    // initialize root
    if cfg!(unix) {
        match components.peek() {
            Some(Component::RootDir) => {
                let root = components.next().unwrap();
                stack.push(root);
            }
            _ => {
                base_path = get_current_dir()?;
                stack.extend(base_path.components());
            }
        }
    } else if cfg!(windows) {
        use std::path::Prefix;

        fn get_drive_letter<P: AsRef<Path>>(path: P) -> Option<u8> {
            let path = path.as_ref();
            let mut components = path.components();

            match components.next() {
                Some(Component::Prefix(prefix)) => match prefix.kind() {
                    Prefix::Disk(drive_letter) | Prefix::VerbatimDisk(drive_letter) => {
                        Some(drive_letter)
                    }
                    _ => None,
                },
                _ => None,
            }
        }

        fn get_drive_path(drive_letter: u8) -> PathBuf {
            format!(r"{}:\", drive_letter as char).into()
        }

        fn get_drive_relative(drive_letter: u8) -> Result<PathBuf> {
            let path = get_current_dir()?;
            if Some(drive_letter) == get_drive_letter(&path) {
                return Ok(path);
            }

            if let Some(path) = env::var_os(format!("={}:", drive_letter as char)) {
                return Ok(path.into());
            }

            let path = get_drive_path(drive_letter);
            Ok(path)
        }

        match components.peek() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive_letter) => {
                    let disk = components.next().unwrap();
                    match components.peek() {
                        Some(Component::RootDir) => {
                            let root = components.next().unwrap();
                            stack.push(disk);
                            stack.push(root);
                        }
                        _ => {
                            base_path = get_drive_relative(drive_letter)?;
                            stack.extend(base_path.components());
                        }
                    }
                }
                Prefix::VerbatimDisk(drive_letter) => {
                    components.next();
                    if components.peek() == Some(&Component::RootDir) {
                        components.next();
                    }

                    base_path = get_drive_path(drive_letter);
                    stack.extend(base_path.components());
                }
                _ => bail!("invalid path: {}", path.display()),
            },
            Some(Component::RootDir) => {
                components.next();

                let current_dir = env::current_dir()?;
                let drive_letter = get_drive_letter(&current_dir).with_context(|| {
                    format!("could not get drive letter: {}", current_dir.display())
                })?;
                base_path = get_drive_path(drive_letter);
                stack.extend(base_path.components());
            }
            _ => {
                base_path = get_current_dir()?;
                stack.extend(base_path.components());
            }
        }
    }

    for component in components {
        match component {
            Component::Normal(_) => stack.push(component),
            Component::CurDir => (),
            Component::ParentDir => {
                if stack.last() != Some(&Component::RootDir) {
                    stack.pop();
                }
            }
            Component::Prefix(_) | Component::RootDir => unreachable!(),
        }
    }

    let result = stack.iter().collect::<PathBuf>();
    if !result.is_dir() {
        bail!("could not resolve path: {}", result.display());
    }
    Ok(result)
}

pub fn get_current_dir() -> Result<PathBuf> {
    env::current_dir().context("could not get current path")
}

pub fn path_to_str<P: AsRef<Path>>(path: &P) -> Result<&str> {
    let path = path.as_ref();
    path.to_str()
        .with_context(|| format!("invalid utf-8 sequence in path: {}", path.display()))
}
