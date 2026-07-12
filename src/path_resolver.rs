use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::util;

pub fn resolve(path: impl AsRef<Path>, resolve_symlinks: bool) -> Result<PathBuf> {
    if resolve_symlinks {
        return util::canonicalize(path);
    }

    #[cfg(target_os = "macos")]
    {
        return resolve_macos(path);
    }

    #[cfg(not(target_os = "macos"))]
    {
        util::resolve_path(path)
    }
}

#[cfg(target_os = "macos")]
fn resolve_macos(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = util::resolve_path(path)?;

    if path.is_file() {
        if let Some(resolved) = resolve_finder_alias(&path)? {
            return Ok(resolved);
        }
    }

    Ok(path)
}

#[cfg(target_os = "macos")]
fn resolve_finder_alias(path: &Path) -> Result<Option<PathBuf>> {
    let script = r#"
on run argv
    tell application "Finder"
        set alias_path to POSIX file (item 1 of argv) as alias
        return POSIX path of (original item of alias_path as alias)
    end tell
end run
"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .arg(path)
        .output()
        .context("could not run osascript to resolve Finder alias")?;

    if !output.status.success() {
        return Ok(None);
    }

    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if resolved.is_empty() {
        return Ok(None);
    }

    Ok(Some(util::resolve_path(resolved)?))
}