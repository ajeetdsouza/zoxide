use clap::ArgEnum;

#[derive(Debug, Eq, PartialEq)]
pub struct Opts<'a> {
    pub cmd: Option<&'a str>,
    pub hook: Hook,
    pub echo: bool,
    pub resolve_symlinks: bool,
}

impl Opts<'_> {
    #[cfg(unix)]
    pub const DEVNULL: &'static str = "/dev/null";
    #[cfg(windows)]
    pub const DEVNULL: &'static str = "NUL";
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

#[derive(ArgEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hook {
    None,
    Prompt,
    Pwd,
}

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::*;

    use askama::Template;
    use assert_cmd::Command;
    use once_cell::sync::OnceCell;
    use seq_macro::seq;

    use std::env;

    macro_rules! with_opts_size {
        ($macro:ident) => {
            $macro!(24);
        };
    }

    fn opts() -> &'static [Opts<'static>] {
        static OPTS: OnceCell<Vec<Opts>> = OnceCell::new();
        const BOOLS: &[bool] = &[false, true];
        const HOOKS: &[Hook] = &[Hook::None, Hook::Prompt, Hook::Pwd];
        const CMDS: &[Option<&str>] = &[None, Some("z")];

        OPTS.get_or_init(|| {
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

            // Verify that the value hardcoded into `with_opts_size` is correct.
            macro_rules! id {
                ($x:literal) => {
                    $x
                };
            }
            assert_eq!(opts.len(), with_opts_size!(id));

            opts
        })
    }

    fn make_cmd(cmd: &str) -> Command {
        let tempdir = tempfile::tempdir().unwrap();
        let tempdir = tempdir.path();

        let mut vars = vec![("HOME", tempdir.into()), ("PYLINTHOME", tempdir.into())];
        if let Some(var) = env::var_os("PATH") {
            vars.push(("PATH", var));
        }
        if let Some(var) = env::var_os("PYTHONNOUSERSITE") {
            vars.push(("PYTHONNOUSERSITE", var));
        }
        if let Some(var) = env::var_os("PYTHONPATH") {
            vars.push(("PYTHONPATH", var));
        }

        let mut cmd = Command::new(cmd);
        cmd.env_clear().envs(vars);

        cmd
    }

    macro_rules! generate_tests {
        ($N:literal) => {
            seq!(i in 0..$N {
                #[test]
                fn bash_bash_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Bash(opts).render().unwrap();
                    make_cmd("bash")
                        .args(&["-c", &source, "--noediting", "--noprofile", "--norc"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn bash_shellcheck_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Bash(opts).render().unwrap();
                    make_cmd("shellcheck")
                        .args(&["--enable", "all", "--shell", "bash", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn bash_shfmt_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = Bash(opts).render().unwrap();
                    source.push('\n');
                    // FIXME: caused by <https://github.com/djc/askama/issues/377>
                    let source = source.as_str().trim_start();
                    make_cmd("shfmt")
                        .args(&["-d", "-s", "-ln", "bash", "-i", "4", "-ci", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn fish_fish_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Fish(opts).render().unwrap();
                    make_cmd("fish")
                        .args(&["--command", &source, "--private"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                // TODO: fishindent

                #[test]
                fn posix_bashposix_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    let assert = make_cmd("bash")
                        .args(&["--posix", "-c", &source, "--noediting", "--noprofile", "--norc"])
                        .assert()
                        .success()
                        .stderr("");

                    if opts.hook != Hook::Pwd {
                        assert.stdout("");
                    }
                }

                #[test]
                fn posix_dash_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    let assert = make_cmd("dash")
                        .args(&["-c", &source])
                        .assert()
                        .success()
                        .stderr("");

                    if opts.hook != Hook::Pwd {
                        assert.stdout("");
                    }
                }

                #[test]
                fn posix_shellcheck_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    make_cmd("shellcheck")
                        .args(&["--enable", "all", "--shell", "sh", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn posix_shfmt_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = Posix(opts).render().unwrap();
                    source.push('\n');
                    // FIXME: caused by <https://github.com/djc/askama/issues/377>
                    let source = source.as_str().trim_start();
                    make_cmd("shfmt")
                        .args(&["-d", "-s", "-ln", "posix", "-i", "4", "-ci", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn powershell_pwsh_#i() {
                let opts = dbg!(&opts()[i]);
                    let source = PowerShell(opts).render().unwrap();
                    make_cmd("pwsh")
                        .args(&["-Command", &source, "-NoLogo", "-NonInteractive", "-NoProfile"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn xonsh_black_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = Xonsh(opts).render().unwrap();
                    source.push('\n');
                    make_cmd("black")
                        .args(&["--check", "--diff", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("");
                }

                #[test]
                fn xonsh_mypy_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Xonsh(opts).render().unwrap();
                    make_cmd("mypy")
                        .args(&["--command", &source])
                        .assert()
                        .success()
                        .stderr("");
                }

                #[test]
                fn xonsh_pylint_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = Xonsh(opts).render().unwrap();
                    source.push('\n');
                    make_cmd("pylint")
                        .args(&["--from-stdin", "zoxide"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stderr("");
                }

                #[test]
                // FIXME: caused by <https://github.com/xonsh/xonsh/issues/3959>
                #[ignore]
                fn xonsh_xonsh_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Xonsh(opts).render().unwrap();
                    make_cmd("xonsh")
                        .args(&["-c", &source, "--no-rc"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn zsh_zsh_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Zsh(opts).render().unwrap();
                    make_cmd("zsh")
                        .args(&["-c", &source, "--no-rcs"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }
            });
        }
    }

    with_opts_size!(generate_tests);
}
