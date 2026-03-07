#![cfg(not(windows))]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;

fn fake_path(script: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fzf");
    fs::write(&path, script).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    let path_env = format!("{}:{}", dir.path().display(), env::var("PATH").unwrap_or_default());
    (dir, path_env)
}

#[test]
fn interactive_query_is_silent_when_fzf_reports_no_match() {
    let home = tempfile::tempdir().unwrap();
    let (_bin_dir, path_env) = fake_path("#!/bin/sh\ncat >/dev/null\nexit 1\n");

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("HOME", home.path())
        .env("PATH", path_env)
        .args(["query", "--interactive"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
}

#[test]
fn interactive_query_is_silent_when_fzf_returns_empty_output() {
    let home = tempfile::tempdir().unwrap();
    let (_bin_dir, path_env) = fake_path("#!/bin/sh\ncat >/dev/null\nexit 0\n");

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("HOME", home.path())
        .env("PATH", path_env)
        .args(["query", "--interactive"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
}
