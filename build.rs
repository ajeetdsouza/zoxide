use std::process::Command;

fn main() {
    let status_code = Command::new("git")
        .args(&["describe", "--tags", "--exact-match", "--dirty"])
        .status()
        .ok()
        .and_then(|status| status.code());

    let is_tagged = match status_code {
        Some(code) => code == 0,
        None => false,
    };

    // If this is a tagged commit (a release), we don't want to include the
    // commit hash in the version output.
    let revision = if is_tagged {
        String::new()
    } else {
        let mut hash = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|proc| String::from_utf8(proc.stdout).ok())
            .unwrap_or_default();

        if !hash.is_empty() {
            hash = format!("-{}", hash);
        }

        hash
    };

    println!("cargo:rustc-env=GIT_HASH={}", revision);
}
