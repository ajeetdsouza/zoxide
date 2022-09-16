use std::process::Command;
use std::{env, io};

fn main() {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let version = match env::var_os("PROFILE") {
        Some(profile) if profile == "release" => format!("v{pkg_version}"),
        _ => git_version().unwrap_or_else(|| format!("v{pkg_version}-unknown")),
    };
    println!("cargo:rustc-env=ZOXIDE_VERSION={version}");

    // Since we are generating completions in the package directory, we need to set this so that
    // Cargo doesn't rebuild every time.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=tests/");

    generate_completions().unwrap();
}

fn git_version() -> Option<String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let mut git = Command::new("git");
    git.args(&["-C", dir, "describe", "--tags", "--match=v*.*.*", "--always", "--broken"]);

    let output = git.output().ok()?;
    if !output.status.success() || output.stdout.is_empty() || !output.stderr.is_empty() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn generate_completions() -> io::Result<()> {
    #[path = "src/cmd/cmd.rs"]
    mod cmd;

    use clap::CommandFactory;
    use clap_complete::generate_to;
    use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
    use clap_complete_fig::Fig;
    use cmd::Cmd;

    let cmd = &mut Cmd::command();
    let bin_name = env!("CARGO_PKG_NAME");
    let out_dir = "contrib/completions";

    generate_to(Bash, cmd, bin_name, out_dir)?;
    generate_to(Elvish, cmd, bin_name, out_dir)?;
    generate_to(Fig, cmd, bin_name, out_dir)?;
    generate_to(Fish, cmd, bin_name, out_dir)?;
    generate_to(PowerShell, cmd, bin_name, out_dir)?;
    generate_to(Zsh, cmd, bin_name, out_dir)?;

    Ok(())
}
