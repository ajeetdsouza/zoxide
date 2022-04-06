//! Test clap generated completions.
#![cfg(feature = "nix-dev")]

use assert_cmd::Command;

#[test]
fn completions_bash() {
    let source = include_str!("../contrib/completions/zoxide.bash");
    Command::new("bash").args(&["--noprofile", "--norc", "-c", source]).assert().success().stdout("").stderr("");
}

// Elvish: the completions file uses editor commands to add completions to the shell. However,
// Elvish does not support running editor commands from a script, so we can't create a test for
// this. See: https://github.com/elves/elvish/issues/1299

#[test]
fn completions_fish() {
    let source = include_str!("../contrib/completions/zoxide.fish");
    let tempdir = tempfile::tempdir().unwrap();
    let tempdir = tempdir.path().to_str().unwrap();

    Command::new("fish")
        .env("HOME", tempdir)
        .args(&["--command", source, "--private"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn completions_powershell() {
    let source = include_str!("../contrib/completions/_zoxide.ps1");
    Command::new("pwsh")
        .args(&["-NoLogo", "-NonInteractive", "-NoProfile", "-Command", source])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn completions_zsh() {
    let source = r#"
    set -eu
    completions='./contrib/completions'
    test -d "$completions"
    fpath=("$completions" $fpath)
    autoload -Uz compinit
    compinit -u
    "#;

    Command::new("zsh").args(&["-c", source, "--no-rcs"]).assert().success().stdout("").stderr("");
}
