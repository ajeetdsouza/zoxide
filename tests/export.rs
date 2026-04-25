use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn zoxide() -> Command {
    Command::cargo_bin("zoxide").unwrap()
}

#[test]
fn export_missing_format() {
    let out_dir = TempDir::new().unwrap();
    let out_file = out_dir.path().join("export.json");

    zoxide()
        .args(["export", "--out", out_file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn export_invalid_format() {
    let out_dir = TempDir::new().unwrap();
    let out_file = out_dir.path().join("export.json");

    zoxide()
        .args(["export", "--format", "xml", "--out", out_file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn export_missing_out() {
    zoxide()
        .args(["export", "--format", "json"])
        .assert()
        .failure();
}

#[test]
fn export_valid_json() {
    let data_dir = TempDir::new().unwrap();
    let test_dir = TempDir::new().unwrap();
    let out_dir = TempDir::new().unwrap();
    let out_file = out_dir.path().join("export.json");

    zoxide()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["add", test_dir.path().to_str().unwrap()])
        .assert()
        .success();

    zoxide()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["export", "--format", "json", "--out", out_file.to_str().unwrap()])
        .assert()
        .success();

    assert!(out_file.exists());
}

#[test]
fn export_valid_csv() {
    let data_dir = TempDir::new().unwrap();
    let test_dir = TempDir::new().unwrap();
    let out_dir = TempDir::new().unwrap();
    let out_file = out_dir.path().join("export.csv");

    zoxide()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["add", test_dir.path().to_str().unwrap()])
        .assert()
        .success();

    zoxide()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["export", "--format", "csv", "--out", out_file.to_str().unwrap()])
        .assert()
        .success();

    assert!(out_file.exists());

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("path,rank,last_accessed"));
}
