use clap::ArgEnum;

#[derive(Debug)]
pub struct Opts<'a> {
    pub cmd: Option<&'a str>,
    pub hook: Hook,
    pub echo: bool,
    pub resolve_symlinks: bool,
}

impl Opts<'_> {
    pub const DEVNULL: &'static str = if cfg!(windows) { "NUL" } else { "/dev/null" };
}

macro_rules! make_template {
    ($name:ident, $path:expr) => {
        #[derive(::std::fmt::Debug, ::askama::Template)]
        #[template(path = $path)]
        pub struct $name<'a>(pub &'a self::Opts<'a>);

        impl<'a> ::std::ops::Deref for $name<'a> {
            type Target = self::Opts<'a>;
            fn deref(&self) -> &Self::Target {
                self.0
            }
        }
    };
}

make_template!(Bash, "bash.txt");
make_template!(Fish, "fish.txt");
make_template!(Posix, "posix.txt");
make_template!(PowerShell, "powershell.txt");
make_template!(Xonsh, "xonsh.txt");
make_template!(Zsh, "zsh.txt");

#[derive(ArgEnum, Clone, Copy, Debug, PartialEq)]
pub enum Hook {
    None,
    Prompt,
    Pwd,
}

#[cfg(test)]
mod tests {
    use super::*;

    use askama::Template;
    use assert_cmd::Command;
    use once_cell::sync::OnceCell;

    fn opts() -> &'static [Opts<'static>] {
        static OPTS: OnceCell<Vec<Opts>> = OnceCell::new();
        OPTS.get_or_init(|| {
            const BOOLS: &[bool] = &[false, true];
            const HOOKS: &[Hook] = &[Hook::None, Hook::Prompt, Hook::Pwd];
            const CMDS: &[Option<&str>] = &[None, Some("z")];

            let mut opts = Vec::new();
            for &echo in BOOLS {
                for &resolve_symlinks in BOOLS {
                    for &hook in HOOKS {
                        for &cmd in CMDS {
                            let opt = Opts {
                                echo,
                                resolve_symlinks,
                                hook,
                                cmd,
                            };
                            opts.push(opt);
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
            let source = Bash(opts).render().unwrap();
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
            let source = Posix(opts).render().unwrap();
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
            let source = Posix(opts).render().unwrap();
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
            let source = Fish(opts).render().unwrap();
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
            let source = PowerShell(opts).render().unwrap();
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
            let source = Bash(opts).render().unwrap();
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
            let source = Posix(opts).render().unwrap();
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
            let source = Bash(opts).render().unwrap();
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
            let source = Posix(opts).render().unwrap();
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
            let source = Xonsh(opts).render().unwrap();
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
            let source = Zsh(opts).render().unwrap();
            Command::new("zsh")
                .args(&["-c", &source])
                .assert()
                .success()
                .stdout("")
                .stderr("");
        }
    }
}
