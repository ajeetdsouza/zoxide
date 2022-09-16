use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{self, Command};

use anyhow::{bail, Context, Result};
use clap::Parser;
use ignore::Walk;

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
    fn run(self) -> Result<()>;
}

impl CommandExt for &mut Command {
    fn run(self) -> Result<()> {
        println!(">>> {self:?}");
        let status = self.status().with_context(|| format!("command failed to start: {self:?}"))?;
        if !status.success() {
            bail!("command failed: {self:?} with status: {status:?}");
        }
        Ok(())
    }
}

fn run_ci(nix_enabled: bool) -> Result<()> {
    // Run cargo-clippy.
    Command::new("cargo").args(&["clippy", "--all-features", "--all-targets"]).args(&["--", "-Dwarnings"]).run()?;
    run_fmt(nix_enabled, true)?;
    run_lint(nix_enabled)?;
    run_tests(nix_enabled, "")
}

fn run_fmt(nix_enabled: bool, check: bool) -> Result<()> {
    // Run cargo-fmt.
    // let check_args: &[&str] = if check { &["--check", "--files-with-diff"] } else { &[] };
    // Command::new("cargo").args(&["fmt", "--all", "--"]).args(check_args).run()?;

    // Run nixfmt.
    if nix_enabled {
        for result in Walk::new("./") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("nix")) {
                let check_args: &[&str] = if check { &["--check"] } else { &[] };
                Command::new("nixfmt").args(check_args).arg("--").arg(path).run()?;
            }
        }
    }

    Ok(())
}

fn run_lint(nix_enabled: bool) -> Result<()> {
    if nix_enabled {
        // Run cargo-audit.
        Command::new("cargo").args(&["audit", "--deny=warnings"]).run()?;

        // Run markdownlint.
        for result in Walk::new("./") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("md")) {
                Command::new("markdownlint").arg(path).run()?;
            }
        }

        // Run mandoc with linting enabled.
        for result in Walk::new("./man/") {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("1")) {
                Command::new("mandoc").args(&["-man", "-Wall", "-Tlint", "--"]).arg(path).run()?;
            }
        }
    }

    Ok(())
}

fn run_tests(nix_enabled: bool, name: &str) -> Result<()> {
    let args: &[&str] = if nix_enabled { &["nextest", "run", "--all-features"] } else { &["test"] };
    Command::new("cargo").args(args).args(&["--no-fail-fast", "--workspace", "--", name]).run()
}

fn enable_nix() -> bool {
    let nix_supported = cfg!(any(target_os = "linux", target_os = "macos"));
    if !nix_supported {
        return false;
    }
    let nix_enabled = env::var_os("IN_NIX_SHELL").unwrap_or_default() == "pure";
    if nix_enabled {
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
