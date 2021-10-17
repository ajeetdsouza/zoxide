use std::process::Command;
use std::{env, io};

fn main() {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let version = match env::var_os("PROFILE") {
        Some(profile) if profile == "release" => format!("v{}", pkg_version),
        _ => git_version().unwrap_or_else(|| format!("v{}-unknown", pkg_version)),
    };
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);

    // Since we are generating completions in the package directory, we need to set this so that
    // Cargo doesn't rebuild every time.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=tests");

    generate_completions().unwrap();
}

fn git_version() -> Option<String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let mut git = Command::new("git");
    git.args(&["-C", &dir, "describe", "--tags", "--broken"]);

    let output = git.output().ok()?;
    if !output.status.success() || output.stdout.is_empty() || !output.stderr.is_empty() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn generate_completions() -> io::Result<()> {
    #[path = "src/app/_app.rs"]
    mod app;

    use app::App;
    use clap::IntoApp;
    use clap_generate::generate_to;
    use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};

    let app = &mut App::into_app();
    let bin_name = env!("CARGO_PKG_NAME");
    let out_dir = "contrib/completions";

    generate_to::<Bash, _, _>(app, bin_name, out_dir)?;
    generate_to::<Elvish, _, _>(app, bin_name, out_dir)?;
    generate_to::<Fish, _, _>(app, bin_name, out_dir)?;
    generate_to::<PowerShell, _, _>(app, bin_name, out_dir)?;
    generate_to::<Zsh, _, _>(app, bin_name, out_dir)?;

    Ok(())
}
