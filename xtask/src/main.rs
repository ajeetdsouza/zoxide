use anyhow::{bail, Context, Result};
use clap::Parser;
use ignore::Walk;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() -> Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = dir.parent().with_context(|| format!("could not find workspace root: {}", dir.display()))?;
    env::set_current_dir(dir).with_context(|| format!("could not set current directory: {}", dir.display()))?;
    let nix_enabled = enable_nix();

    let app = App::parse();
    match app {
        App::CI => run_ci(nix_enabled)?,
        App::Fmt { check } => run_fmt(nix_enabled, check)?,
        App::Lint => run_lint(nix_enabled)?,
        App::Test { name } => run_tests(nix_enabled, &name)?,
    }

    Ok(())
}

#[derive(Parser)]
enum App {
    CI,
    Fmt {
        #[clap(long)]
        check: bool,
    },
    Lint,
    Test {
        #[clap(default_value = "")]
        name: String,
    },
}

trait CommandExt {
    fn _run(self) -> Result<()>;
}

impl CommandExt for &mut Command {
    fn _run(self) -> Result<()> {
        println!(">>> {:?}", self);
        let status = self.status().with_context(|| format!("command failed to start: {:?}", self))?;
        if !status.success() {
            bail!("command failed: {:?} with status: {:?}", self, status);
        }
        Ok(())
    }
}

fn run_ci(nix_enabled: bool) -> Result<()> {
    let color: &[&str] = if is_ci() { &["--color=always"] } else { &[] };
    Command::new("cargo").args(&["check", "--all-features"]).args(color)._run()?;

    run_fmt(nix_enabled, true)?;
    run_lint(nix_enabled)?;
    run_tests(nix_enabled, "")
}

fn run_fmt(nix_enabled: bool, check: bool) -> Result<()> {
    // Run cargo-fmt.
    let color: &[&str] = if is_ci() { &["--color=always"] } else { &[] };
    let check_args: &[&str] = if check { &["--check", "--files-with-diff"] } else { &[] };
    Command::new("cargo").args(&["fmt", "--all", "--"]).args(color).args(check_args)._run()?;

    // Run nixfmt.
    if nix_enabled {
        for result in Walk::new("./") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("nix")) {
                let check_args: &[&str] = if check { &["--check"] } else { &[] };
                Command::new("nixfmt").args(check_args).arg("--").arg(path)._run()?;
            }
        }
    }

    Ok(())
}

fn run_lint(nix_enabled: bool) -> Result<()> {
    // Run cargo-clippy.
    let color: &[&str] = if is_ci() { &["--color=always"] } else { &[] };
    Command::new("cargo")
        .args(&["clippy", "--all-features", "--all-targets"])
        .args(color)
        .args(&["--", "-Dwarnings"])
        ._run()?;

    if nix_enabled {
        // Run cargo-audit.
        let color: &[&str] = if is_ci() { &["--color=always"] } else { &[] };
        Command::new("cargo").args(&["audit", "--deny=warnings"]).args(color)._run()?;

        // Run markdownlint.
        for result in Walk::new("./") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("md")) {
                Command::new("markdownlint").arg(path)._run()?;
            }
        }

        // Run mandoc with linting enabled.
        for result in Walk::new("./man/") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("1")) {
                Command::new("mandoc").args(&["-man", "-Wall", "-Tlint", "--"]).arg(path)._run()?;
            }
        }
    }

    Ok(())
}

fn run_tests(nix_enabled: bool, name: &str) -> Result<()> {
    let color: &[&str] = if is_ci() { &["--color=always"] } else { &[] };
    let features: &[&str] = if nix_enabled { &["--all-features"] } else { &[] };
    Command::new("cargo").args(&["test", "--no-fail-fast", "--workspace"]).args(color).args(features).arg(name)._run()
}

fn is_ci() -> bool {
    env::var_os("CI").is_some()
}

fn enable_nix() -> bool {
    let nix_supported = cfg!(any(target_os = "linux", target_os = "macos"));
    if !nix_supported {
        return false;
    }
    let nix_enabled = env::var_os("IN_NIX_SHELL").unwrap_or_default() == "pure";
    if nix_enabled {
        env::set_var("CARGO_TARGET_DIR", "target_nix");
        return true;
    }
    let nix_detected = Command::new("nix-shell").arg("--version").status().map(|s| s.success()).unwrap_or(false);
    if !nix_detected {
        return false;
    }

    println!("Detected Nix in environment, re-running in Nix.");
    let args = env::args();
    let cmd = shell_words::join(args);

    let status = Command::new("nix-shell").args(&["--pure", "--run", &cmd, "--", "shell.nix"]).status().unwrap();
    process::exit(status.code().unwrap_or(1));
}
