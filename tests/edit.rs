//! Test `zoxide edit` subcommands.

use std::fs;

use assert_cmd::Command;

/// `zoxide edit <delete|increment|decrement>` prints the reload list for fzf.
/// The list must be sorted by descending score, regardless of the order the
/// entries happen to be stored in.
#[test]
fn edit_delete_output_is_sorted_by_score() {
    let data_dir = tempfile::tempdir().unwrap();
    let dirs = tempfile::tempdir().unwrap();

    // Add five directories with increasing scores.
    let paths: Vec<String> = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|name| {
            let path = dirs.path().join(name);
            fs::create_dir(&path).unwrap();
            path.to_str().unwrap().to_string()
        })
        .collect();
    for (idx, path) in paths.iter().enumerate() {
        Command::cargo_bin("zoxide")
            .unwrap()
            .env("_ZO_DATA_DIR", data_dir.path())
            .args(["add", "--score", &(idx + 1).to_string(), "--", path])
            .assert()
            .success();
    }

    // Deleting an entry must not disturb the ordering of the rest.
    let output = Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir.path())
        .args(["edit", "delete", "--", &paths[1]])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();

    let entries: Vec<(f64, &str)> = output
        .split_terminator('\0')
        .map(|line| {
            let (score, path) = line.split_once('\t').unwrap();
            (score.trim().parse::<f64>().unwrap(), path)
        })
        .collect();

    let expected: Vec<&str> =
        vec![&paths[4], &paths[3], &paths[2], &paths[0]].into_iter().map(String::as_str).collect();
    let got: Vec<&str> = entries.iter().map(|(_, path)| *path).collect();
    assert_eq!(expected, got);

    assert!(
        entries.windows(2).all(|w| w[0].0 >= w[1].0),
        "scores are not in descending order: {entries:?}"
    );
}
