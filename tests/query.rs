use std::fs;

use assert_cmd::cargo_bin_cmd;

#[test]
fn inherited_base_dir() {
    let temp = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(temp.path()).unwrap();
    let base = root.join("project one");
    let first = base.join("src");
    let second = base.join("other").join("src");
    let outside = root.join("project one-extra").join("src");
    let zoxide = || {
        let mut cmd = cargo_bin_cmd!();
        cmd.env("_ZO_DATA_DIR", root.join("db"))
            .env_remove("_ZO_BASE_DIR")
            .env_remove("_ZO_EXCLUDE_DIRS")
            .env_remove("_ZO_MAXAGE")
            .env_remove("_ZO_RESOLVE_SYMLINKS");
        cmd
    };

    for (path, score) in [(&first, "2"), (&second, "1"), (&outside, "10")] {
        fs::create_dir_all(path).unwrap();
        zoxide().args(["add", "--score", score]).arg(path).assert().success();
    }

    let unscoped = format!("{}\n", outside.display());
    zoxide().args(["query", "src"]).assert().success().stdout(unscoped.clone());
    zoxide()
        .env("_ZO_BASE_DIR", &base)
        .args(["query", "src"])
        .assert()
        .success()
        .stdout(format!("{}\n", first.display()));
    zoxide()
        .env("_ZO_BASE_DIR", &first)
        .args(["query", "--base-dir"])
        .arg(second.parent().unwrap())
        .arg("src")
        .assert()
        .success()
        .stdout(format!("{}\n", second.display()));
    zoxide()
        .env("_ZO_BASE_DIR", &base)
        .args(["query", "--base-dir=", "src"])
        .assert()
        .success()
        .stdout(unscoped.clone());
    zoxide().env("_ZO_BASE_DIR", "").args(["query", "src"]).assert().success().stdout(unscoped);
    zoxide()
        .env("_ZO_BASE_DIR", &base)
        .args(["query", "--list", "--exclude"])
        .arg(&first)
        .arg("src")
        .assert()
        .success()
        .stdout(format!("{}\n", second.display()));
}
