//! Syntax checking for manpages.
#![cfg(feature = "nix_tests")]

use assert_cmd::Command;

use std::fs;

#[test]
fn mandoc_lint() {
    let paths = fs::read_dir("man")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension() == Some("1".as_ref()) {
                Some(path.to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Command::new("mandoc")
        .args(&["-man", "-Wall", "-Tlint", "--"])
        .args(&paths)
        .assert()
        .success()
        .stdout("")
        .stderr("");
}
