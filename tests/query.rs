use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn zoxide() -> (Command, TempDir) {
    let data = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("zoxide").unwrap();
    cmd.env("_ZO_DATA_DIR", data.path());
    (cmd, data)
}

#[test]
fn query_existing_file_is_not_a_directory() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("reading");
    fs::write(&file, b"").unwrap();

    let (mut cmd, _data) = zoxide();
    let assert = cmd.args(["query", file.to_str().unwrap()]).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("not a directory"),
        "stderr should mention the file is not a directory, got: {stderr}"
    );
}

#[test]
fn query_missing_keyword_is_no_match() {
    let (mut cmd, _data) = zoxide();
    let assert =
        cmd.args(["query", "definitely-does-not-exist-zoxide-query-test"]).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("no match found"),
        "stderr should be the usual no-match error, got: {stderr}"
    );
}
