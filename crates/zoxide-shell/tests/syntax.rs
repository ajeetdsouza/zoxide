use askama::Template;
use assert_cmd::Command;
use once_cell::sync::OnceCell;
use zoxide_shell::{Bash, Fish, Hook, Opts, Posix, PowerShell, Xonsh, Zsh};

fn opts() -> &'static [Opts<'static>] {
    static OPTS: OnceCell<Vec<Opts>> = OnceCell::new();
    OPTS.get_or_init(|| {
        let mut opts = Vec::new();
        for &echo in &[false, true] {
            for &resolve_symlinks in &[false, true] {
                for &hook in &[Hook::None, Hook::Prompt, Hook::Pwd] {
                    for &cmd in &[None, Some("z")] {
                        opts.push(Opts {
                            echo,
                            resolve_symlinks,
                            hook,
                            cmd,
                        });
                    }
                }
            }
        }
        opts
    })
}

#[test]
fn test_bash() {
    for opts in opts() {
        let source = crate::Bash(opts).render().unwrap();
        Command::new("bash")
            .args(&["-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_bash_posix() {
    for opts in opts() {
        let source = crate::Posix(opts).render().unwrap();
        let assert = Command::new("bash")
            .args(&["--posix", "-c", &source])
            .assert()
            .success()
            .stderr("");

        if opts.hook != Hook::Pwd {
            assert.stdout("");
        }
    }
}

#[test]
fn test_dash() {
    for opts in opts() {
        let source = crate::Posix(opts).render().unwrap();
        let assert = Command::new("bash")
            .args(&["--posix", "-c", &source])
            .assert()
            .success()
            .stderr("");

        if opts.hook != Hook::Pwd {
            assert.stdout("");
        }
    }
}

#[test]
fn test_fish() {
    for opts in opts() {
        let source = crate::Fish(opts).render().unwrap();
        Command::new("fish")
            .args(&["-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_pwsh() {
    for opts in opts() {
        let source = crate::PowerShell(opts).render().unwrap();
        Command::new("pwsh")
            .args(&["-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_shellcheck_bash() {
    for opts in opts() {
        let source = crate::Bash(opts).render().unwrap();
        Command::new("shellcheck")
            .args(&["--shell", "bash", "-"])
            .write_stdin(source.as_bytes())
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_shellcheck_sh() {
    for opts in opts() {
        let source = crate::Posix(opts).render().unwrap();
        Command::new("shellcheck")
            .args(&["--shell", "sh", "-"])
            .write_stdin(source.as_bytes())
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_shfmt_bash() {
    for opts in opts() {
        let source = crate::Bash(opts).render().unwrap();
        Command::new("shfmt")
            .args(&["-d", "-s", "-ln", "bash", "-i", "4", "-ci", "-"])
            .write_stdin(source.as_bytes())
            .write_stdin(b"\n".as_ref())
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_shfmt_posix() {
    for opts in opts() {
        let source = crate::Posix(opts).render().unwrap();
        Command::new("shfmt")
            .args(&["-d", "-s", "-ln", "posix", "-i", "4", "-ci", "-"])
            .write_stdin(source.as_bytes())
            .write_stdin(b"\n".as_ref())
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_xonsh() {
    for opts in opts() {
        let source = crate::Xonsh(opts).render().unwrap();
        Command::new("xonsh")
            .args(&["-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}

#[test]
fn test_zsh() {
    for opts in opts() {
        let source = crate::Zsh(opts).render().unwrap();
        Command::new("zsh")
            .args(&["-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}
