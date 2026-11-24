//! Exercise directory jumps through the generated shell integrations.
#![cfg(any(feature = "nix-dev", windows))]

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

use assert_cmd::Command;
#[cfg(feature = "nix-dev")]
use rstest::rstest;

struct Shell {
    name: &'static str,
    source: String,
    home: PathBuf,
    data: PathBuf,
    path: std::ffi::OsString,
}

impl Shell {
    fn new(name: &'static str, root: &Path) -> Self {
        let shell = match name {
            "windows-powershell" => "powershell",
            _ if name.starts_with("posix-") => "posix",
            _ => name,
        };
        let source = Command::cargo_bin("zoxide")
            .unwrap()
            .args(["init", shell, "--hook", "none"])
            .env_remove("_ZO_ECHO")
            .env_remove("_ZO_RESOLVE_SYMLINKS")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let mut source = String::from_utf8(source).unwrap();
        if shell == "elvish" {
            // Editor commands are unavailable in noninteractive Elvish.
            source = source
                .lines()
                .filter(|line| !line.contains("edit:"))
                .map(|line| format!("{line}\n"))
                .collect();
        }
        let binary = assert_cmd::cargo::cargo_bin("zoxide");
        let path = env::join_paths(
            [binary.parent().unwrap().to_path_buf()]
                .into_iter()
                .chain(env::split_paths(&env::var_os("PATH").unwrap())),
        )
        .unwrap();
        Self { name, source, home: root.to_path_buf(), data: root.join("data"), path }
    }

    fn jump(&self, start: &Path, args: &[&str], expected: &Path) {
        self.check_jump(start, args, expected, false);
    }

    fn check_jump(&self, start: &Path, args: &[&str], expected: &Path, error: bool) {
        let args = args.iter().map(|arg| format!("'{arg}'")).collect::<Vec<_>>();
        let (jump, pwd) = match self.name {
            "xonsh" => {
                (format!("assert __zoxide_z([{}]) == 0", args.join(", ")), "print(__zoxide_pwd())")
            }
            "elvish" => (format!("__zoxide_z {}", args.join(" ")), "builtin:echo $pwd"),
            "nushell" => (format!("__zoxide_z {}", args.join(" ")), "$env.PWD"),
            "powershell" | "windows-powershell" => (
                format!("__zoxide_z {}", args.join(" ")),
                "(Microsoft.PowerShell.Management\\Get-Location).ProviderPath",
            ),
            _ => (format!("__zoxide_z {}", args.join(" ")), "pwd"),
        };
        let source = format!("{}\n{jump}\n{pwd}\n", self.source);
        let (program, options): (&str, &[&str]) = match self.name {
            "bash" => ("bash", &["--noprofile", "--norc", "-e", "-c"]),
            "posix-bash" => ("bash", &["--posix", "--noprofile", "--norc", "-e", "-c"]),
            "posix-dash" => ("dash", &["-e", "-c"]),
            "posix-ksh" => ("ksh", &["-e", "-c"]),
            "fish" => ("fish", &["--no-config", "--private", "-c"]),
            "elvish" => ("elvish", &["-norc", "-c"]),
            "nushell" => ("nu", &["--no-config-file", "--commands"]),
            "powershell" => ("pwsh", &["-NoLogo", "-NonInteractive", "-NoProfile", "-Command"]),
            "windows-powershell" => {
                ("powershell", &["-NoLogo", "-NonInteractive", "-NoProfile", "-Command"])
            }
            "tcsh" => ("tcsh", &["-e", "-f", "-s"]),
            "xonsh" => ("xonsh", &["--no-rc", "-c"]),
            "zsh" => ("zsh", &["--no-globalrcs", "--no-rcs", "-e", "-c"]),
            _ => unreachable!(),
        };
        let mut command = Command::new(program);
        command.args(options);
        if self.name == "tcsh" {
            command.write_stdin(source);
        } else {
            command.arg(source);
        }
        let assert = command
            .current_dir(start)
            .env("HOME", &self.home)
            .env("_ZO_DATA_DIR", &self.data)
            .env("PATH", &self.path)
            .env_remove("_ZO_ECHO")
            .env_remove("_ZO_RESOLVE_SYMLINKS")
            .env_remove("CDPATH")
            .timeout(Duration::from_secs(10))
            .assert()
            .success();
        let output = assert.get_output();
        assert_eq!(output.stderr.is_empty(), !error, "{:?}", output);
        assert_eq!(
            Path::new(String::from_utf8_lossy(&output.stdout).trim()),
            expected,
            "{} {args:?}",
            self.name,
        );
    }
}

#[cfg(feature = "nix-dev")]
#[rstest]
fn parent_shorthand_shells(
    #[values(
        "bash",
        "elvish",
        "fish",
        "nushell",
        "posix-bash",
        "posix-dash",
        "posix-ksh",
        "powershell",
        "tcsh",
        "xonsh",
        "zsh"
    )]
    name: &'static str,
) {
    parent_shorthand(name);
}

#[cfg(windows)]
#[test]
fn parent_shorthand_windows() {
    parent_shorthand("windows-powershell");
}

fn parent_shorthand(name: &'static str) {
    let tempdir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(tempdir.path()).unwrap();
    let start = root.join("space path/a/b/c/d");
    fs::create_dir_all(&start).unwrap();
    let shell = Shell::new(name, &root);

    // Keep a matching database entry present while exercising the shorthand.
    let query = root.join("... needle");
    let ordinary = root.join("needle...suffix");
    fs::create_dir(&query).unwrap();
    fs::create_dir(&ordinary).unwrap();
    Command::cargo_bin("zoxide")
        .unwrap()
        .arg("add")
        .arg("--")
        .args([&query, &ordinary])
        .env("_ZO_DATA_DIR", &shell.data)
        .assert()
        .success();

    for dots in ["..", "...", "....", ".....", "......"] {
        let parent = start.ancestors().nth(dots.len() - 1).unwrap();
        shell.jump(&start, &[dots], parent);
    }
    let filesystem_root = root.ancestors().last().unwrap();
    shell.jump(filesystem_root, &["......"], filesystem_root);
    shell.jump(&start, &["."], &start);
    shell.jump(&start, &["../.."], start.ancestors().nth(2).unwrap());

    let child = start.join("space child");
    fs::create_dir(&child).unwrap();
    shell.jump(&start, &["./space child"], &child);
    shell.jump(&start, &["needle...suffix"], &ordinary);
    shell.jump(&start, &["...", "needle"], &query);
    if matches!(name, "fish" | "powershell" | "windows-powershell") {
        // Windows PowerShell rejects the newline during native path checking.
        shell.check_jump(&start, &["...\n"], &start, true);
    }
    Command::cargo_bin("zoxide")
        .unwrap()
        .args(["query", "--", "...", "needle"])
        .env("_ZO_DATA_DIR", &shell.data)
        .assert()
        .success()
        .stdout(format!("{}\n", query.display()));

    if !cfg!(windows) {
        // A file is not a directory and must not take precedence over shorthand.
        fs::write(start.join("...."), "").unwrap();
        shell.jump(&start, &["...."], start.ancestors().nth(3).unwrap());

        let literal = start.join("...");
        fs::create_dir(&literal).unwrap();
        let expected = if name == "nushell" {
            // Nushell's native cd already expands dots, even for existing names.
            start.ancestors().nth(2).unwrap()
        } else {
            &literal
        };
        shell.jump(&start, &["..."], expected);
        shell.jump(&start, &["./..."], expected);
    }
}
