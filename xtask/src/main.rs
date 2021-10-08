use anyhow::{bail, Context, Result};
use clap::Clap;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() -> Result<()> {
    let nix_enabled = enable_nix();

    let app = App::parse();
    match app {
        App::Audit => run_audit(&[] as &[&str])?,
        App::CI => run_ci(nix_enabled)?,
        App::Fmt => run_fmt(&[] as &[&str])?,
        App::Markdownlint => run_markdownlint()?,
        App::Test { args } => run_test(nix_enabled, &args)?,
    }

    Ok(())
}

#[derive(Clap)]
enum App {
    Audit,
    CI,
    Fmt,
    Markdownlint,
    Test { args: Vec<String> },
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

fn run_audit<S: AsRef<OsStr>>(args: &[S]) -> Result<()> {
    Command::new("cargo").args(&["audit", "--deny=warnings"]).args(args)._run()
}

fn run_clippy<S: AsRef<OsStr>>(args: &[S]) -> Result<()> {
    Command::new("cargo").args(&["clippy", "--all-features", "--all-targets"]).args(args)._run()
}

fn run_ci(nix_enabled: bool) -> Result<()> {
    let color = if env::var_os("CI").is_some() { "--color=always" } else { "--color=auto" };
    run_fmt(&["--check", color, "--files-with-diff"])?;
    run_clippy(&[color])?;
    run_test(nix_enabled, &[color, "--no-fail-fast"])?;
    if nix_enabled {
        run_audit(&[] as &[&str])?; // FIXME: add "color" when cargo-audit 0.15.3 is released
        run_markdownlint()?;
    }
    Ok(())
}

fn run_fmt<S: AsRef<OsStr>>(rustfmt_args: &[S]) -> Result<()> {
    Command::new("cargo").args(&["fmt", "--all", "--"]).args(rustfmt_args)._run()
}

fn run_markdownlint() -> Result<()> {
    Command::new("markdownlint").args(&["--ignore-path=.gitignore", "."])._run()
}

fn run_test<S: AsRef<OsStr>>(nix_enabled: bool, args: &[S]) -> Result<()> {
    Command::new("cargo")
        .args(&["test", "--workspace", if nix_enabled { "--features=nix" } else { "" }])
        .args(args)
        ._run()
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
    let mut nix_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    nix_path.push("../shell.nix");
    let status = Command::new("nix-shell").args(&["--pure", "--run", &cmd, "--"]).arg(nix_path).status().unwrap();
    process::exit(status.code().unwrap_or(1));
}
