use assert_cmd::Command;
use tempfile::TempDir;

fn zoxide(data: &TempDir) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("zoxide"));
    command
        .env("_ZO_DATA_DIR", data.path())
        .env("_ZO_EXCLUDE_DIRS", "")
        .env("_ZO_RESOLVE_SYMLINKS", "0")
        .env("_ZO_MAXAGE", "10000");
    command
}

#[test]
fn add_rejects_non_directories_by_default() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    let file = entries.path().join("executable");
    std::fs::write(&file, "").unwrap();

    for path in [file, entries.path().join("missing")] {
        let output = zoxide(&data).arg("add").arg(&path).assert().failure();
        assert!(String::from_utf8_lossy(&output.get_output().stderr).contains("not a directory:"));
    }
    zoxide(&data).args(["query", "--all", "--list"]).assert().success().stdout("");
}

#[test]
fn add_directory() {
    let data = tempfile::tempdir().unwrap();
    let directory = tempfile::tempdir().unwrap();
    zoxide(&data).arg("add").arg(directory.path()).assert().success();
    zoxide(&data)
        .arg("query")
        .assert()
        .success()
        .stdout(format!("{}\n", directory.path().display()));
}

#[test]
fn add_non_directories_and_increment_rank() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    let file = entries.path().join("executable");
    std::fs::write(&file, "").unwrap();
    let missing = entries.path().join("missing");

    zoxide(&data)
        .args(["add", "--allow-non-directories"])
        .args([&file, &missing])
        .assert()
        .success();
    for path in [&file, &missing] {
        zoxide(&data)
            .args(["query", "--all", "--score"])
            .arg(path.file_name().unwrap())
            .assert()
            .success()
            .stdout(format!("   4.0 {}\n", path.display()));
    }

    zoxide(&data)
        .args(["add", "--allow-non-directories", "--score", "2"])
        .arg(&missing)
        .assert()
        .success();
    zoxide(&data).args(["query", "--all", "--list", "--score"]).assert().success().stdout(format!(
        "  12.0 {}\n   4.0 {}\n",
        missing.display(),
        file.display()
    ));

    // Ordinary queries must not return files or missing paths for navigation.
    zoxide(&data).args(["query", "--list"]).assert().success().stdout("");
    zoxide(&data).args(["query", "--all", "--list"]).assert().success().stdout(format!(
        "{}\n{}\n",
        missing.display(),
        file.display()
    ));
}

#[test]
fn add_non_directory_resolves_relative_paths() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    zoxide(&data)
        .current_dir(entries.path())
        .args(["add", "--allow-non-directories", "./parent/../missing"])
        .assert()
        .success();
    zoxide(&data)
        .args(["query", "--all"])
        .assert()
        .success()
        .stdout(format!("{}\n", entries.path().join("missing").display()));
}

#[test]
fn add_non_directories_preserves_exclusions() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    let excluded = entries.path().join("excluded");
    zoxide(&data)
        .env("_ZO_EXCLUDE_DIRS", &excluded)
        .args(["add", "--allow-non-directories"])
        .arg(&excluded)
        .assert()
        .success();
    zoxide(&data).args(["query", "--all", "--list"]).assert().success().stdout("");
}

#[test]
fn add_non_directories_preserves_character_filter() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    for name in ["line\nbreak", "carriage\rreturn"] {
        zoxide(&data)
            .args(["add", "--allow-non-directories"])
            .arg(entries.path().join(name))
            .assert()
            .success();
    }
    zoxide(&data).args(["query", "--all", "--list"]).assert().success().stdout("");
}

#[test]
fn add_non_directories_preserves_canonicalization() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    let file = entries.path().join("executable");
    std::fs::write(&file, "").unwrap();
    zoxide(&data)
        .env("_ZO_RESOLVE_SYMLINKS", "1")
        .args(["add", "--allow-non-directories"])
        .arg(&file)
        .assert()
        .success();
    zoxide(&data)
        .args(["query", "--all"])
        .assert()
        .success()
        .stdout(format!("{}\n", dunce::canonicalize(&file).unwrap().display()));
    zoxide(&data)
        .env("_ZO_RESOLVE_SYMLINKS", "1")
        .args(["add", "--allow-non-directories"])
        .arg(entries.path().join("missing"))
        .assert()
        .failure();
}

#[cfg(unix)]
#[test]
fn add_non_directories_preserves_default_home_exclusion() {
    let data = tempfile::tempdir().unwrap();
    let entries = tempfile::tempdir().unwrap();
    let home = entries.path().join("home");
    zoxide(&data)
        .env_remove("_ZO_EXCLUDE_DIRS")
        .env("HOME", &home)
        .args(["add", "--allow-non-directories"])
        .arg(&home)
        .assert()
        .success();
    zoxide(&data).args(["query", "--all", "--list"]).assert().success().stdout("");
}
