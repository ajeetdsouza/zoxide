use git2::{DescribeFormatOptions, DescribeOptions, Repository};

use std::env;

fn git_version() -> Option<String> {
    let path = env::var("CARGO_MANIFEST_DIR").ok()?;
    let repository = Repository::open(path).ok()?;

    let mut options = DescribeOptions::new();
    options.describe_tags();
    let describe = repository.describe(&options).ok()?;

    let mut format = DescribeFormatOptions::new();
    format.dirty_suffix("-dirty");

    describe.format(Some(&format)).ok()
}

fn main() {
    let version = git_version()
        .or_else(|| env::var("CARGO_PKG_VERSION").ok())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);
}
