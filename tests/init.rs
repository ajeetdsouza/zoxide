use assert_cmd::Command;
use rstest::rstest;

#[rstest]
fn init_db_only(
    #[values("bash", "elvish", "fish", "nushell", "posix", "powershell", "tcsh", "xonsh", "zsh")]
    shell: &str,
    #[values(false, true)] db_only: bool,
    #[values(false, true)] no_cmd: bool,
) {
    let mut command = Command::cargo_bin("zoxide").unwrap();
    command.args(["init", shell]);
    if db_only {
        command.arg("--db-only");
    }
    if no_cmd {
        command.arg("--no-cmd");
    }
    command.assert().success().stderr("");
}

#[cfg(feature = "nix-dev")]
fn navigation(shell: &str, db_only: bool, script: &str, args: &[&str]) {
    let temp = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(temp.path()).unwrap();
    let local = root.join("local");
    let database = root.join("database");
    std::fs::create_dir_all(local.join("match space")).unwrap();
    std::fs::create_dir_all(database.join("match space")).unwrap();
    std::fs::create_dir_all(local.join("untracked")).unwrap();
    let data = root.join("data");
    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", &data)
        .env_remove("_ZO_EXCLUDE_DIRS")
        .env_remove("_ZO_MAXAGE")
        .args(["add", "--"])
        .arg(database.join("match space"))
        .assert()
        .success();
    let mut init = Command::cargo_bin("zoxide").unwrap();
    init.args(["init", shell, "--hook", if shell == "posix" { "prompt" } else { "none" }]);
    if db_only {
        init.arg("--db-only");
    }
    let output = init.env_remove("_ZO_ECHO").env_remove("_ZO_RESOLVE_SYMLINKS").output().unwrap();
    assert!(output.status.success());
    let source = String::from_utf8(output.stdout).unwrap() + script;
    let bin = assert_cmd::cargo::cargo_bin("zoxide");
    let path = std::env::join_paths(
        std::iter::once(bin.parent().unwrap().to_path_buf())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    Command::new(if shell == "powershell" { "pwsh" } else { "bash" })
        .args(args)
        .arg(source)
        .current_dir(&local)
        .env("PATH", path)
        .env("_ZO_DATA_DIR", data)
        .env_remove("_ZO_EXCLUDE_DIRS")
        .env_remove("_ZO_MAXAGE")
        .env_remove("_ZO_RESOLVE_SYMLINKS")
        .env("EXPECTED", if db_only { database } else { local.clone() }.join("match space"))
        .env("START", &local)
        .env("DB_ONLY", if db_only { "1" } else { "0" })
        .assert()
        .success()
        .stderr("");
}

#[cfg(feature = "nix-dev")]
#[rstest]
fn powershell_db_only_navigation(#[values(false, true)] db_only: bool) {
    navigation(
        "powershell",
        db_only,
        r#"
$ErrorActionPreference = 'Stop'
z 'match space'
if ((Get-Location).Path -ne $env:EXPECTED) { throw 'wrong destination' }
z -
if ((Get-Location).Path -ne $env:START) { throw 'history shortcut failed' }
z 'untracked' 2>$null
if ($env:DB_ONLY -eq '1') {
    if ($LASTEXITCODE -eq 0) { throw 'untracked directory accepted' }
    if ((Get-Location).Path -ne $env:START) { throw 'untracked directory visited' }
} else {
    if ((Get-Location).Path -ne (Join-Path $env:START 'untracked')) { throw 'direct navigation failed' }
    z -
}
z 'not-in-database' 2>$null
if ($LASTEXITCODE -eq 0) { throw 'missing query succeeded' }
if ((Get-Location).Path -ne $env:START) { throw 'failed query changed directory' }
z
if ((Get-Location).Path -ne $HOME) { throw 'home navigation failed' }
"#,
        &["-NoLogo", "-NonInteractive", "-NoProfile", "-Command"],
    );
}

#[cfg(feature = "nix-dev")]
#[rstest]
fn bash_db_only_navigation(
    #[values("bash", "posix")] shell: &str,
    #[values(false, true)] db_only: bool,
) {
    navigation(
        shell,
        db_only,
        r#"
z 'match space'
if command -v cygpath >/dev/null; then EXPECTED=$(cygpath -u "$EXPECTED"); START=$(cygpath -u "$START"); fi
[ "$PWD" = "$EXPECTED" ] || exit 10
z -
[ "$PWD" = "$START" ] || exit 11
if z 'not-in-database' 2>/dev/null; then exit 12; fi
[ "$PWD" = "$START" ] || exit 13
z
[ "$PWD" = "$HOME" ] || exit 14
"#,
        &["--noprofile", "--norc", "-e", "-u", "-c"],
    );
}
