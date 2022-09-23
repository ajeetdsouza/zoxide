use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::SystemTime;
use std::{env, mem};

#[cfg(windows)]
use anyhow::anyhow;
use anyhow::{bail, Context, Result};

use crate::config;
use crate::db::Epoch;
use crate::error::SilentExit;

pub struct Fzf {
    child: Child,
}

impl Fzf {
    pub fn new(multiple: bool) -> Result<Self> {
        const ERR_FZF_NOT_FOUND: &str = "could not find fzf, is it installed?";

        // On Windows, CreateProcess implicitly searches the current working
        // directory for the executable, which is a potential security issue.
        // Instead, we resolve the path to the executable and then pass it to
        // CreateProcess.
        #[cfg(windows)]
        let mut command = Command::new(which::which("fzf.exe").map_err(|_| anyhow!(ERR_FZF_NOT_FOUND))?);
        #[cfg(not(windows))]
        let mut command = Command::new("fzf");
        if multiple {
            command.arg("-m");
        }
        command.arg("--nth=2..").stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(fzf_opts) = config::fzf_opts() {
            command.env("FZF_DEFAULT_OPTS", fzf_opts);
        } else {
            command.args(&[
                // Search result
                "--no-sort",
                // Interface
                "--keep-right",
                // Layout
                "--height=50%",
                "--info=inline",
                "--layout=reverse",
                // Scripting
                "--exit-0",
                "--select-1",
                // Key/Event bindings
                "--bind=ctrl-z:ignore",
            ]);
            if cfg!(unix) {
                // Non-POSIX args are only available on certain operating systems.
                const PREVIEW_CMD: &str = if cfg!(target_os = "linux") {
                    r"\command -p ls -Cp --color=always --group-directories-first {2..}"
                } else {
                    r"\command -p ls -Cp {2..}"
                };
                command.args(&["--preview", PREVIEW_CMD, "--preview-window=down,30%"]).env("SHELL", "sh");
            }
        }

        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == io::ErrorKind::NotFound => bail!(ERR_FZF_NOT_FOUND),
            Err(e) => Err(e).context("could not launch fzf")?,
        };

        Ok(Fzf { child })
    }

    pub fn stdin(&mut self) -> &mut ChildStdin {
        self.child.stdin.as_mut().unwrap()
    }

    pub fn select(mut self) -> Result<String> {
        // Drop stdin to prevent deadlock.
        mem::drop(self.child.stdin.take());

        let mut stdout = self.child.stdout.take().unwrap();
        let mut output = String::new();
        stdout.read_to_string(&mut output).context("failed to read from fzf")?;

        let status = self.child.wait().context("wait failed on fzf")?;
        match status.code() {
            Some(0) => Ok(output),
            Some(1) => bail!("no match found"),
            Some(2) => bail!("fzf returned an error"),
            Some(130) => bail!(SilentExit { code: 130 }),
            Some(128..=254) | None => bail!("fzf was terminated"),
            _ => bail!("fzf returned an unknown error"),
        }
    }
}

/// Similar to [`fs::write`], but atomic (best effort on Windows).
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();
    let dir = path.parent().unwrap();

    // Create a tmpfile.
    let (mut tmp_file, tmp_path) = tmpfile(dir)?;
    let result = (|| {
        // Write to the tmpfile.
        let _ = tmp_file.set_len(contents.len() as u64);
        tmp_file.write_all(contents).with_context(|| format!("could not write to file: {}", tmp_path.display()))?;

        // Set the owner of the tmpfile (UNIX only).
        #[cfg(unix)]
        if let Ok(metadata) = path.metadata() {
            use std::os::unix::fs::MetadataExt;
            use std::os::unix::io::AsRawFd;

            use nix::unistd::{self, Gid, Uid};

            let uid = Uid::from_raw(metadata.uid());
            let gid = Gid::from_raw(metadata.gid());
            let _ = unistd::fchown(tmp_file.as_raw_fd(), Some(uid), Some(gid));
        }

        // Close and rename the tmpfile.
        mem::drop(tmp_file);
        rename(&tmp_path, path)
    })();
    // In case of an error, delete the tmpfile.
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Atomically create a tmpfile in the given directory.
fn tmpfile<P: AsRef<Path>>(dir: P) -> Result<(File, PathBuf)> {
    const MAX_ATTEMPTS: usize = 5;
    const TMP_NAME_LEN: usize = 16;
    let dir = dir.as_ref();

    let mut attempts = 0;
    loop {
        attempts += 1;

        // Generate a random name for the tmpfile.
        let mut name = String::with_capacity(TMP_NAME_LEN);
        name.push_str("tmp_");
        while name.len() < TMP_NAME_LEN {
            name.push(fastrand::alphanumeric());
        }
        let path = dir.join(name);

        // Atomically create the tmpfile.
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break Ok((file, path)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && attempts < MAX_ATTEMPTS => (),
            Err(e) => break Err(e).with_context(|| format!("could not create file: {}", path.display())),
        }
    }
}

/// Similar to [`fs::rename`], but retries on Windows.
fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    const MAX_ATTEMPTS: usize = 5;
    let from = from.as_ref();
    let to = to.as_ref();

    if cfg!(windows) {
        let mut attempts = 0;
        loop {
            attempts += 1;
            match fs::rename(from, to) {
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempts < MAX_ATTEMPTS => (),
                result => break result,
            }
        }
    } else {
        fs::rename(from, to)
    }
    .with_context(|| format!("could not rename file: {} -> {}", from.display(), to.display()))
}

pub fn canonicalize<P: AsRef<Path>>(path: &P) -> Result<PathBuf> {
    dunce::canonicalize(path).with_context(|| format!("could not resolve path: {}", path.as_ref().display()))
}

pub fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("could not get current directory")
}

pub fn current_time() -> Result<Epoch> {
    let current_time =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).context("system clock set to invalid time")?.as_secs();

    Ok(current_time)
}

pub fn path_to_str<P: AsRef<Path>>(path: &P) -> Result<&str> {
    let path = path.as_ref();
    path.to_str().with_context(|| format!("invalid unicode in path: {}", path.display()))
}

/// Returns the absolute version of a path. Like [`std::path::Path::canonicalize`], but doesn't
/// resolve symlinks.
pub fn resolve_path<P: AsRef<Path>>(path: &P) -> Result<PathBuf> {
    let path = path.as_ref();
    let base_path;

    let mut components = path.components().peekable();
    let mut stack = Vec::new();

    // initialize root
    if cfg!(windows) {
        use std::path::Prefix;

        fn get_drive_letter<P: AsRef<Path>>(path: P) -> Option<u8> {
            let path = path.as_ref();
            let mut components = path.components();

            match components.next() {
                Some(Component::Prefix(prefix)) => match prefix.kind() {
                    Prefix::Disk(drive_letter) | Prefix::VerbatimDisk(drive_letter) => Some(drive_letter),
                    _ => None,
                },
                _ => None,
            }
        }

        fn get_drive_path(drive_letter: u8) -> PathBuf {
            format!(r"{}:\", drive_letter as char).into()
        }

        fn get_drive_relative(drive_letter: u8) -> Result<PathBuf> {
            let path = current_dir()?;
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
                    if components.peek() == Some(&Component::RootDir) {
                        let root = components.next().unwrap();
                        stack.push(disk);
                        stack.push(root);
                    } else {
                        base_path = get_drive_relative(drive_letter)?;
                        stack.extend(base_path.components());
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
                let drive_letter = get_drive_letter(&current_dir)
                    .with_context(|| format!("could not get drive letter: {}", current_dir.display()))?;
                base_path = get_drive_path(drive_letter);
                stack.extend(base_path.components());
            }
            _ => {
                base_path = current_dir()?;
                stack.extend(base_path.components());
            }
        }
    } else if components.peek() == Some(&Component::RootDir) {
        let root = components.next().unwrap();
        stack.push(root);
    } else {
        base_path = current_dir()?;
        stack.extend(base_path.components());
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

    Ok(stack.iter().collect())
}

/// Convert a string to lowercase, with a fast path for ASCII strings.
pub fn to_lowercase<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    if s.is_ascii() { s.to_ascii_lowercase() } else { s.to_lowercase() }
}
