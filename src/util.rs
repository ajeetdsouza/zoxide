use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;
use std::{env, mem};

#[cfg(windows)]
use anyhow::anyhow;
use anyhow::{Context, Result, bail};

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
        let mut output = String::new();
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

        // Set the owner and permissions of the tmpfile (UNIX only).
        #[cfg(unix)]
        if let Ok(metadata) = path.metadata() {
            use std::os::unix::fs::{MetadataExt, fchown};

            _ = fchown(&tmp_file, Some(metadata.uid()), Some(metadata.gid()));
            _ = tmp_file.set_permissions(metadata.permissions());
        }

        // Close and rename the tmpfile.
        // In some cases, errors from the last write() are reported only on close().
        // Rust ignores errors from close(), since it occurs inside `Drop`. To
        // catch these errors, we manually call `File::sync_all()` first.
        tmp_file
            .sync_all()
            .with_context(|| format!("could not sync writes to file: {}", tmp_path.display()))?;
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
    let path = dunce::canonicalize(&path)
        .with_context(|| format!("could not resolve path: {}", path.as_ref().display()))?;

    // On Windows, canonicalization leaves UNC paths in their verbatim form
    // (`\\?\UNC\server\share`), which shells such as PowerShell refuse to
    // navigate to. Normalize them to the equivalent plain form
    // (`\\server\share`). This is a no-op on other platforms, where
    // canonicalized paths are always rooted at `/`.
    if let Some(path) = path.to_str()
        && is_unc(path)
    {
        return Ok(PathBuf::from(simplify_unc(path).into_owned()));
    }

    Ok(path)
}

/// Returns true if the path is a verbatim UNC path, i.e. it starts with
/// `\\?\UNC\` (or the forward-slash form `//?/UNC/`).
pub fn is_verbatim_unc(path: &str) -> bool {
    let Some(rest) = path.strip_prefix(r#"\\?\"#).or_else(|| path.strip_prefix("//?/")) else {
        return false;
    };
    let rest = rest.as_bytes();
    // The `UNC` part is matched case-insensitively.
    rest.len() >= 4 && rest[..3].eq_ignore_ascii_case(b"unc") && matches!(rest[3], b'\\' | b'/')
}

/// Returns true if the path has a plain UNC prefix: `\\server\share` or
/// `//server/share`, with a non-empty server and share.
fn has_plain_unc_prefix(path: &str) -> bool {
    // Exclude verbatim (`\\?\...`) and device (`\\.\...`) paths.
    if path.starts_with(r#"\\?\"#)
        || path.starts_with(r#"\\.\"#)
        || path.starts_with("//?/")
        || path.starts_with("//./")
    {
        return false;
    }

    let Some(rest) = path.strip_prefix(r"\\").or_else(|| path.strip_prefix("//")) else {
        return false;
    };
    let mut parts = rest.split(['\\', '/']);
    let (Some(server), Some(share)) = (parts.next(), parts.next()) else {
        return false;
    };
    !server.is_empty() && !share.is_empty()
}

/// Returns true if the path is a UNC path: `\\server\share`,
/// `//server/share`, or a verbatim UNC path (`\\?\UNC\server\share`).
/// UNC paths are always absolute on Windows.
pub fn is_unc(path: &str) -> bool {
    is_verbatim_unc(path) || has_plain_unc_prefix(path)
}

/// Normalizes a verbatim UNC path (`\\?\UNC\server\share\...`) to the plain
/// UNC form (`\\server\share\...`). The two forms are equivalent on Windows,
/// but only the plain form can be passed to shells. Other paths are returned
/// unchanged.
pub fn simplify_unc(path: &str) -> Cow<'_, str> {
    if !is_verbatim_unc(path) {
        return Cow::Borrowed(path);
    }

    // Strip the 8-byte `\\?\UNC\` (or `//?/UNC/`) prefix and replace it with
    // `\\` (or `//`), preserving the separator style of the input.
    let (separator, rest) =
        if path.starts_with("//") { ("//", &path[8..]) } else { (r"\\", &path[8..]) };
    Cow::Owned(format!("{separator}{rest}"))
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

/// Returns the absolute version of a path. Like
/// [`std::path::Path::canonicalize`], but doesn't resolve symlinks.
pub fn resolve_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let base_path;

    let mut components = path.components().peekable();
    let mut stack = Vec::new();

    // initialize root
    if cfg!(windows) {
        use std::path::Prefix;

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

        /// Returns the `\\server\share` root of a UNC path.
        fn unc_root(server: &OsStr, share: &OsStr) -> PathBuf {
            format!(r"\\{}\{}", server.to_string_lossy(), share.to_string_lossy()).into()
        }

        /// Returns the root of an absolute path: `\\server\share` for UNC paths
        /// (a verbatim `\\?\UNC\server\share` prefix is normalized to the plain
        /// form, which is what shells can navigate to), or the drive root
        /// (`C:\`) for disk paths.
        fn get_root(path: &Path) -> Result<PathBuf> {
            if let Some(Component::Prefix(prefix)) = path.components().next() {
                match prefix.kind() {
                    Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                        return Ok(unc_root(server, share));
                    }
                    Prefix::Disk(drive_letter) | Prefix::VerbatimDisk(drive_letter) => {
                        return Ok(get_drive_path(drive_letter));
                    }
                    _ => {}
                }
            }

            let drive_letter = get_drive_letter(path)
                .with_context(|| format!("could not get drive letter: {}", path.display()))?;
            Ok(get_drive_path(drive_letter))
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
                // UNC paths (`\\server\share`) are always absolute. A verbatim
                // UNC prefix (`\\?\UNC\server\share`) is normalized to the
                // plain form, which is what shells can navigate to.
                Prefix::UNC(..) | Prefix::VerbatimUNC(..) => {
                    components.next();
                    if components.peek() == Some(&Component::RootDir) {
                        components.next();
                    }

                    base_path = get_root(path)?;
                    stack.extend(base_path.components());
                }
                _ => bail!("invalid path: {}", path.display()),
            },
            Some(Component::RootDir) => {
                components.next();

                // If the current directory is itself on a UNC path
                // (`\\server\share\...`), reuse its root instead of looking
                // up a drive letter.
                base_path = get_root(&env::current_dir()?)?;
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

// These tests are pure string functions, so they run on all platforms
// (including the Linux CI runners).
#[cfg(test)]
mod unc_tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // Plain UNC paths
    #[case(r"\\server\share", true)]
    #[case(r"\\server\share\dir\sub", true)]
    #[case(r"\\wsl$\fedoraremix", true)]
    #[case(r"\\NAS\Share", true)]
    // Forward-slash form (e.g. MSYS2), which Windows parses as UNC
    #[case("//server/share", true)]
    #[case("//server/share/dir", true)]
    // Verbatim UNC paths (produced by canonicalization on Windows)
    #[case(r"\\?\UNC\server\share", true)]
    #[case(r"\\?\UNC\server\share\dir", true)]
    #[case(r"\\?\UNC\wsl$\fedoraremix", true)]
    #[case("//?/UNC/server/share", true)]
    #[case(r"\\?\unc\server\share", true)] // case-insensitive
    // Not UNC paths
    #[case(r"C:\Users\foo", false)]
    #[case("C:/Users/foo", false)]
    #[case(r"\\?\C:\Users\foo", false)] // verbatim disk
    #[case(r"\\.\COM1", false)] // device path
    #[case(r"\foo\bar", false)]
    #[case("/foo/bar", false)]
    #[case(r"relative\path", false)]
    #[case(r"\\server", false)] // no share
    #[case(r"\\server\", false)] // empty share
    #[case(r"\\", false)]
    #[case("//", false)]
    #[case(r"\\?\UNC", false)] // incomplete verbatim prefix
    #[case("", false)]
    fn detects_unc(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(is_unc(path), expected, "is_unc({path:?})");
    }

    #[rstest]
    #[case(r"\\?\UNC\server\share", true)]
    #[case(r"\\?\UNC\server\share\dir", true)]
    #[case("//?/UNC/server/share", true)]
    #[case(r"\\?\unc\server\share", true)] // case-insensitive
    #[case(r"\\server\share", false)] // already plain
    #[case(r"\\?\C:\foo", false)] // verbatim disk, not UNC
    #[case(r"\\.\COM1", false)]
    #[case(r"C:\foo", false)]
    #[case("/foo/bar", false)]
    #[case("", false)]
    fn detects_verbatim_unc(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(is_verbatim_unc(path), expected, "is_verbatim_unc({path:?})");
    }

    #[rstest]
    // Verbatim UNC is normalized to plain UNC
    #[case(r"\\?\UNC\server\share", r"\\server\share")]
    #[case(r"\\?\UNC\server\share\dir\sub", r"\\server\share\dir\sub")]
    #[case(r"\\?\UNC\wsl$\fedoraremix", r"\\wsl$\fedoraremix")]
    #[case("//?/UNC/server/share/dir", "//server/share/dir")] // keeps separator style
    #[case(r"\\?\unc\server\share", r"\\server\share")] // case-insensitive prefix
    // Everything else is left unchanged
    #[case(r"\\server\share", r"\\server\share")]
    #[case(r"\\server\share\dir", r"\\server\share\dir")]
    #[case("//server/share", "//server/share")]
    #[case(r"C:\Users\foo", r"C:\Users\foo")]
    #[case(r"\\?\C:\Users\foo", r"\\?\C:\Users\foo")] // verbatim disk: dunce handles this
    #[case(r"\\.\COM1", r"\\.\COM1")]
    #[case("/foo/bar", "/foo/bar")]
    #[case("relative", "relative")]
    #[case("", "")]
    fn normalizes_verbatim_unc(#[case] path: &str, #[case] expected: &str) {
        assert_eq!(simplify_unc(path), expected, "simplify_unc({path:?})");
    }

    /// Regression test for https://github.com/ajeetdsouza/zoxide/issues/162:
    /// UNC paths must be recognized as valid absolute paths, and canonicalized
    /// UNC paths must not keep the `\\?\UNC\` prefix that shells reject.
    #[test]
    fn issue_162_unc_paths() {
        // `zoxide add \\wsl$\fedoraremix` used to fail with `invalid path:
        // UNC(...)`.
        assert!(is_unc(r"\\wsl$\fedoraremix"));
        // `dunce::canonicalize` leaves UNC paths as `\\?\UNC\wsl$\fedoraremix`,
        // which PowerShell refuses to cd into.
        assert_eq!(simplify_unc(r"\\?\UNC\wsl$\fedoraremix"), r"\\wsl$\fedoraremix");
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn resolve_path_unc() {
        assert_eq!(resolve_path(r"\\server\share").unwrap(), PathBuf::from(r"\\server\share"));
        assert_eq!(
            resolve_path(r"\\server\share\dir\sub").unwrap(),
            PathBuf::from(r"\\server\share\dir\sub")
        );
        assert_eq!(
            resolve_path(r"\\wsl$\fedoraremix").unwrap(),
            PathBuf::from(r"\\wsl$\fedoraremix")
        );
    }

    #[test]
    fn resolve_path_verbatim_unc_is_normalized() {
        assert_eq!(
            resolve_path(r"\\?\UNC\server\share\dir").unwrap(),
            PathBuf::from(r"\\server\share\dir")
        );
    }

    #[test]
    fn resolve_path_unc_with_dots() {
        assert_eq!(
            resolve_path(r"\\server\share\dir\..\other").unwrap(),
            PathBuf::from(r"\\server\share\other")
        );
        // Cannot traverse above the share root.
        assert_eq!(
            resolve_path(r"\\server\share\..\..").unwrap(),
            PathBuf::from(r"\\server\share")
        );
    }

    #[test]
    fn resolve_path_disk_unchanged() {
        assert_eq!(resolve_path(r"C:\Users\test").unwrap(), PathBuf::from(r"C:\Users\test"));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn write_preserves_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db.txt");
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

        write(&path, "new").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
