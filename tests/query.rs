use assert_cmd::Command;

#[test]
fn interactive_query_accepts_existing_path() {
    let data_dir = tempfile::tempdir().unwrap();
    let target_parent = tempfile::tempdir().unwrap();
    let target = target_parent.path().join("project");
    std::fs::create_dir(&target).unwrap();

    let expected = target.to_string_lossy();

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["query", "--interactive", "--", expected.as_ref()])
        .assert()
        .success()
        .stdout(format!("{expected}\n"))
        .stderr("");
}
