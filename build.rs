use std::env;
use std::process::Command;

fn git_version() -> Option<String> {
    let mut git = Command::new("git");
    git.args(&["describe", "--tags", "--broken"]);

    let output = git.output().ok()?;
    if !output.status.success() || output.stdout.is_empty() || !output.stderr.is_empty() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn crate_version() -> String {
    // unwrap is safe here, since Cargo will always supply this variable.
    format!("v{}", env::var("CARGO_PKG_VERSION").unwrap())
}

fn generate_completions() {
    #[path = "src/app/_app.rs"]
    mod app;

    use app::App;
    use clap::IntoApp;
    use clap_generate::generate_to;
    use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};

    let app = &mut App::into_app();
    let bin_name = &env::var("CARGO_PKG_NAME").unwrap();
    let out_dir = "contrib/completions";

    generate_to::<Bash, _, _>(app, bin_name, out_dir);
    generate_to::<Elvish, _, _>(app, bin_name, out_dir);
    generate_to::<Fish, _, _>(app, bin_name, out_dir);
    generate_to::<PowerShell, _, _>(app, bin_name, out_dir);
    generate_to::<Zsh, _, _>(app, bin_name, out_dir);
}

fn main() {
    // Packaged releases of zoxide almost always use the source tarball
    // provided by GitHub, which does not include the `.git` folder. Since this
    // feature is only useful for development, we can silently fall back to
    // using the crate version.
    let version = git_version().unwrap_or_else(crate_version);
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);

    // Since we are generating completions in the package directory, we need to
    // set this so that Cargo doesn't rebuild every time.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=tests");

    generate_completions();
}
