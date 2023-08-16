use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf, Prefix};
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;
use std::{env, mem};

#[cfg(windows)]
use anyhow::anyhow;
use anyhow::{bail, Context, Result};

use crate::db::{Dir, Epoch};
use crate::error::SilentExit;

pub const SECOND: Epoch = 1;
pub const MINUTE: Epoch = 60 * SECOND;
pub const HOUR: Epoch = 60 * MINUTE;
pub const DAY: Epoch = 24 * HOUR;
pub const WEEK: Epoch = 7 * DAY;
pub const MONTH: Epoch = 30 * DAY;

pub struct Fzf(Command);

impl Fzf {
    const ERR_FZF_NOT_FOUND: &'static str = "could not find fzf, is it installed?";

    pub fn new() -> Result<Self> {
        // On Windows, CreateProcess implicitly searches the current working
        // directory for the executable, which is a potential security issue.
        // Instead, we resolve the path to the executable and then pass it to
        // CreateProcess.
        #[cfg(windows)]
        let program = which::which("fzf.exe").map_err(|_| anyhow!(Self::ERR_FZF_NOT_FOUND))?;
        #[cfg(not(windows))]
        let program = "fzf";

        // TODO: check version of fzf here.

        let mut cmd = Command::new(program);
        cmd.args([
            // Search mode
            "--delimiter=\t",
            "--nth=2",
            // Scripting
            "--read0",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

        Ok(Fzf(cmd))
    }

    pub fn enable_preview(&mut self) -> &mut Self {
        // Previews are only supported on UNIX.
        if !cfg!(unix) {
            return self;
        }

        self.args([
            // Non-POSIX args are only available on certain operating systems.
            if cfg!(target_os = "linux") {
                r"--preview=\command -p ls -Cp --color=always --group-directories-first {2..}"
            } else {
                r"--preview=\command -p ls -Cp {2..}"
            },
            // Rounded edges don't display correctly on some terminals.
            "--preview-window=down,30%,sharp",
        ])
        .envs([
            // Enables colorized `ls` output on macOS / FreeBSD.
            ("CLICOLOR", "1"),
            // Forces colorized `ls` output when the output is not a
            // TTY (like in fzf's preview window) on macOS /
            // FreeBSD.
            ("CLICOLOR_FORCE", "1"),
            // Ensures that the preview command is run in a
            // POSIX-compliant shell, regardless of what shell the
            // user has selected.
            ("SHELL", "sh"),
        ])
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.0.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.0.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.0.envs(vars);
        self
    }

    pub fn spawn(&mut self) -> Result<FzfChild> {
        match self.0.spawn() {
            Ok(child) => Ok(FzfChild(child)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => bail!(Self::ERR_FZF_NOT_FOUND),
            Err(e) => Err(e).context("could not launch fzf"),
        }
    }
}

pub struct FzfChild(Child);

impl FzfChild {
    pub fn write(&mut self, dir: &Dir, now: Epoch) -> Result<Option<String>> {
        let handle = self.0.stdin.as_mut().unwrap();
        match write!(handle, "{}\0", dir.display().with_score(now).with_separator('\t')) {
            Ok(()) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => self.wait().map(Some),
            Err(e) => Err(e).context("could not write to fzf"),
        }
    }

    pub fn wait(&mut self) -> Result<String> {
        // Drop stdin to prevent deadlock.
        mem::drop(self.0.stdin.take());

        let mut stdout = self.0.stdout.take().unwrap();
        let mut output = String::default();
        stdout.read_to_string(&mut output).context("failed to read from fzf")?;

        let status = self.0.wait().context("wait failed on fzf")?;
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
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();
    let dir = path.parent().unwrap();

    // Create a tmpfile.
    let (mut tmp_file, tmp_path) = tmpfile(dir)?;
    let result = (|| {
        // Write to the tmpfile.
        _ = tmp_file.set_len(contents.len() as u64);
        tmp_file
            .write_all(contents)
            .with_context(|| format!("could not write to file: {}", tmp_path.display()))?;

        // Set the owner of the tmpfile (UNIX only).
        #[cfg(unix)]
        if let Ok(metadata) = path.metadata() {
            use std::os::unix::fs::MetadataExt;
            use std::os::unix::io::AsRawFd;

            use nix::unistd::{self, Gid, Uid};

            let uid = Uid::from_raw(metadata.uid());
            let gid = Gid::from_raw(metadata.gid());
            _ = unistd::fchown(tmp_file.as_raw_fd(), Some(uid), Some(gid));
        }

        // Close and rename the tmpfile.
        mem::drop(tmp_file);
        rename(&tmp_path, path)
    })();
    // In case of an error, delete the tmpfile.
    if result.is_err() {
        _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Atomically create a tmpfile in the given directory.
fn tmpfile(dir: impl AsRef<Path>) -> Result<(File, PathBuf)> {
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
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && attempts < MAX_ATTEMPTS => {}
            Err(e) => {
                break Err(e).with_context(|| format!("could not create file: {}", path.display()));
            }
        }
    }
}

/// Similar to [`fs::rename`], but with retries on Windows.
fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    const MAX_ATTEMPTS: usize = if cfg!(windows) { 5 } else { 1 };
    let mut attempts = 0;

    loop {
        match fs::rename(from, to) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempts < MAX_ATTEMPTS => {
                attempts += 1
            }
            result => {
                break result.with_context(|| {
                    format!("could not rename file: {} -> {}", from.display(), to.display())
                });
            }
        }
    }
}

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    dunce::canonicalize(&path)
        .with_context(|| format!("could not resolve path: {}", path.as_ref().display()))
}

pub fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("could not get current directory")
}

pub fn current_time() -> Result<Epoch> {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock set to invalid time")?
        .as_secs();

    Ok(current_time)
}

pub fn path_to_str(path: &impl AsRef<Path>) -> Result<&str> {
    let path = path.as_ref();
    path.to_str().with_context(|| format!("invalid unicode in path: {}", path.display()))
}

pub fn patch_path(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        fn patch_drive(drive_letter: u8) -> char {
            drive_letter.to_ascii_uppercase() as char
        }

        let mut components = path.components();
        match components.next() {
            Some(Component::Prefix(prefix)) => {
                let prefix = match prefix.kind() {
                    Prefix::Disk(drive_letter) => {
                        format!(r"{}:", patch_drive(drive_letter))
                    }
                    Prefix::VerbatimDisk(drive_letter) => {
                        format!(r"\\?\{}:", patch_drive(drive_letter))
                    }
                    _ => return path,
                };

                let mut path = PathBuf::default();
                path.push(prefix);
                path.extend(components);
                path
            }
            _ => path,
        }
    } else {
        path
    }
}

/// Returns the absolute version of a path. Like
/// [`std::path::Path::canonicalize`], but doesn't resolve symlinks.
pub fn resolve_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let base_path;

    let mut components = path.components().peekable();
    let mut stack = Vec::new();

    // initialize root
    if cfg!(windows) {
        fn get_drive_letter(path: impl AsRef<Path>) -> Option<u8> {
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
            format!(r"{}:\", drive_letter.to_ascii_uppercase() as char).into()
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
                    components.next();
                    if components.peek() == Some(&Component::RootDir) {
                        components.next();
                        base_path = get_drive_path(drive_letter);
                    } else {
                        base_path = get_drive_relative(drive_letter)?;
                    }

                    stack.extend(base_path.components());
                }
                Prefix::VerbatimDisk(drive_letter) => {
                    components.next();
                    if components.peek() == Some(&Component::RootDir) {
                        components.next();
                        base_path = get_drive_path(drive_letter);
                    } else {
                        bail!("illegal path: {}", path.display());
                    }

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
            Component::CurDir => {}
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
pub fn to_lowercase(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    if s.is_ascii() { s.to_ascii_lowercase() } else { s.to_lowercase() }
}
