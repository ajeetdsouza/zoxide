use clap::IntoApp;
use clap_generate::{generate_to, generators::*};

use std::env;
use std::process::Command;

include!("src/app.rs");

fn completions() {
    let mut app = Cli::into_app();
    let bin_name = env!("CARGO_PKG_NAME");
    let out_dir = "contrib/completions";

    generate_to::<Bash, _, _>(&mut app, bin_name, out_dir);
    generate_to::<Elvish, _, _>(&mut app, bin_name, out_dir);
    generate_to::<Fish, _, _>(&mut app, bin_name, out_dir);
    generate_to::<PowerShell, _, _>(&mut app, bin_name, out_dir);
    generate_to::<Zsh, _, _>(&mut app, bin_name, out_dir);
}

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
    completions();
}
