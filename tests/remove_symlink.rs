#![cfg(unix)]

use std::os::unix::fs::symlink;

use assert_cmd::Command;

#[test]
fn remove_symlink_when_paths_are_resolved() {
    let temp = tempfile::tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let target = temp.path().join("target");
    let link = temp.path().join("link");
    std::fs::create_dir(&target).unwrap();
    symlink(&target, &link).unwrap();

    let zoxide = || {
        let mut cmd = Command::cargo_bin("zoxide").unwrap();
        cmd.env("_ZO_DATA_DIR", &data_dir)
            .env("_ZO_RESOLVE_SYMLINKS", "1")
            .env("_ZO_EXCLUDE_DIRS", "");
        cmd
    };

    zoxide().arg("add").arg(&link).assert().success();
    zoxide()
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", target.display()));
    zoxide().arg("remove").arg(&link).assert().success();
    zoxide().args(["query", "--all", "--list"]).assert().success().stdout("");
}
