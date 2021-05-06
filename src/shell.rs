use crate::app::InitHook;

#[derive(Debug, Eq, PartialEq)]
pub struct Opts<'a> {
    pub cmd: Option<&'a str>,
    pub hook: InitHook,
    pub echo: bool,
    pub resolve_symlinks: bool,
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
make_template!(Elvish, "elvish.txt");
make_template!(Fish, "fish.txt");
make_template!(Nushell, "nushell.txt");
make_template!(Posix, "posix.txt");
make_template!(Powershell, "powershell.txt");
make_template!(Xonsh, "xonsh.txt");
make_template!(Zsh, "zsh.txt");

#[cfg(feature = "shell_tests")]
#[cfg(test)]
mod tests {
    use super::*;

    use askama::Template;
    use assert_cmd::Command;
    use once_cell::sync::OnceCell;
    use seq_macro::seq;

    macro_rules! with_opts_size {
        ($macro:ident) => {
            $macro!(24);
        };
    }

    fn opts() -> &'static [Opts<'static>] {
        static OPTS: OnceCell<Vec<Opts>> = OnceCell::new();
        const BOOLS: &[bool] = &[false, true];
        const HOOKS: &[InitHook] = &[InitHook::None, InitHook::Prompt, InitHook::Pwd];
        const CMDS: &[Option<&str>] = &[None, Some("z")];

        OPTS.get_or_init(|| {
            let mut opts = Vec::new();
            for &echo in BOOLS {
                for &resolve_symlinks in BOOLS {
                    for &hook in HOOKS {
                        for &cmd in CMDS {
                            opts.push(Opts {
                                cmd,
                                hook,
                                echo,
                                resolve_symlinks,
                            });
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

    macro_rules! make_tests {
        ($N:literal) => {
            seq!(i in 0..$N {
                #[test]
                fn bash_bash_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Bash(opts).render().unwrap();
                    Command::new("bash")
                        .args(&["--noprofile", "--norc", "-c", &source])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn bash_shellcheck_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Bash(opts).render().unwrap();
                    Command::new("shellcheck")
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

                    Command::new("shfmt")
                        .args(&["-d", "-s", "-ln", "bash", "-i", "4", "-ci", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn elvish_elvish_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = String::new();

                    // Filter out lines using edit:*, since those functions
                    // are only available in the interactive editor.
                    for line in Elvish(opts)
                        .render()
                        .unwrap()
                        .split('\n')
                        .filter(|line| !line.contains("edit:"))
                    {
                        source.push_str(line);
                        source.push('\n');
                    }

                    Command::new("elvish")
                        .args(&["-c", &source, "-norc"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn fish_fish_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Fish(opts).render().unwrap();

                    let tempdir = tempfile::tempdir().unwrap();
                    let tempdir = tempdir.path().to_str().unwrap();

                    Command::new("fish")
                        .env("HOME", tempdir)
                        .args(&["--command", &source, "--private"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn fish_fishindent_#i() {
                    let opts = dbg!(&opts()[i]);
                    let mut source = Fish(opts).render().unwrap();
                    source.push('\n');

                    let tempdir = tempfile::tempdir().unwrap();
                    let tempdir = tempdir.path().to_str().unwrap();

                    Command::new("fish")
                        .env("HOME", tempdir)
                        .args(&["--command", "fish_indent", "--private"])
                        .write_stdin(source.to_string())
                        .assert()
                        .success()
                        .stdout(source)
                        .stderr("");
                }

                #[test]
                fn nushell_nushell_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Nushell(opts).render().unwrap();

                    let tempdir = tempfile::tempdir().unwrap();
                    let tempdir = tempdir.path().to_str().unwrap();

                    let assert = Command::new("nu")
                        .env("HOME", tempdir)
                        .args(&["--commands", &source])
                        .assert()
                        .success()
                        .stderr("");

                    if opts.hook != InitHook::Pwd {
                        assert.stdout("");
                    }
                }

                #[test]
                fn posix_bashposix_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    let assert = Command::new("bash")
                        .args(&["--posix", "--noprofile", "--norc", "-c", &source])
                        .assert()
                        .success()
                        .stderr("");

                    if opts.hook != InitHook::Pwd {
                        assert.stdout("");
                    }
                }

                #[test]
                fn posix_dash_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    let assert = Command::new("dash")
                        .args(&["-c", &source])
                        .assert()
                        .success()
                        .stderr("");

                    if opts.hook != InitHook::Pwd {
                        assert.stdout("");
                    }
                }

                #[test]
                fn posix_shellcheck_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Posix(opts).render().unwrap();
                    Command::new("shellcheck")
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
                    Command::new("shfmt")
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
                    let source = Powershell(opts).render().unwrap();
                    Command::new("pwsh")
                        .args(&["-NoLogo", "-NonInteractive", "-NoProfile", "-Command", &source])
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
                    Command::new("black")
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
                    Command::new("mypy")
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
                    Command::new("pylint")
                        .args(&["--from-stdin", "zoxide"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stderr("");
                }

                #[test]
                fn xonsh_xonsh_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Xonsh(opts).render().unwrap();

                    // We can't pass the source directly to `xonsh -c` due to
                    // a bug: <https://github.com/xonsh/xonsh/issues/3959>
                    Command::new("xonsh")
                        .args(&[
                            "-c",
                            "import sys; execx(sys.stdin.read(), 'exec', __xonsh__.ctx, filename='zoxide')",
                            "--no-rc"
                        ])
                        .write_stdin(source.as_bytes())
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn zsh_shellcheck_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Zsh(opts).render().unwrap();
                    // ShellCheck doesn't support zsh yet.
                    // https://github.com/koalaman/shellcheck/issues/809
                    Command::new("shellcheck")
                        .args(&["--enable", "all", "--shell", "bash", "-"])
                        .write_stdin(source)
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }

                #[test]
                fn zsh_zsh_#i() {
                    let opts = dbg!(&opts()[i]);
                    let source = Zsh(opts).render().unwrap();
                    Command::new("zsh")
                        .args(&["-c", &source, "--no-rcs"])
                        .assert()
                        .success()
                        .stdout("")
                        .stderr("");
                }
            });
        }
    }

    with_opts_size!(make_tests);
}
