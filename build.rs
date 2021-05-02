use std::env;
use std::process::Command;

fn git_version() -> Option<String> {
    // Packages releases of zoxide almost always use the source tarball
    // provided by GitHub, which does not include the `.git` folder. Since this
    // feature is only useful for development, there's no need of printing a
    // warning here.
    let mut git = Command::new("git");
    git.args(&["describe", "--tags", "--broken"]);

    let output = git.output().ok()?;
    String::from_utf8(output.stdout).ok()
}

fn crate_version() -> String {
    // unwrap is safe here, since Cargo will always supply this variable.
    format!("v{}", env::var("CARGO_PKG_VERSION").unwrap())
}

fn main() {
    let version = git_version().unwrap_or_else(crate_version);
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);
}
