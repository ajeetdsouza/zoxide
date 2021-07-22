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
    use rstest::rstest;

    #[rstest]
    fn bash_bash(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Bash(&opts).render().unwrap();

        Command::new("bash")
            .args(&["--noprofile", "--norc", "-c", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn bash_shellcheck(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Bash(&opts).render().unwrap();

        Command::new("shellcheck")
            .args(&["--enable", "all", "--shell", "bash", "-"])
            .write_stdin(source)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn bash_shfmt(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = Bash(&opts).render().unwrap();
        source.push('\n');

        Command::new("shfmt")
            .args(&["-d", "-s", "-ln", "bash", "-i", "4", "-ci", "-"])
            .write_stdin(source)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn elvish_elvish(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = String::new();

        // Filter out lines using edit:*, since those functions
        // are only available in the interactive editor.
        for line in
            Elvish(&opts).render().unwrap().split('\n').filter(|line| !line.contains("edit:"))
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

    #[rstest]
    fn fish_fish(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Fish(&opts).render().unwrap();

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

    #[rstest]
    fn fish_fishindent(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = Fish(&opts).render().unwrap();
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

    #[rstest]
    fn nushell_nushell(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Nushell(&opts).render().unwrap();

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

    #[rstest]
    fn posix_bashposix(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Posix(&opts).render().unwrap();

        let assert = Command::new("bash")
            .args(&["--posix", "--noprofile", "--norc", "-c", &source])
            .assert()
            .success()
            .stderr("");

        if opts.hook != InitHook::Pwd {
            assert.stdout("");
        }
    }

    #[rstest]
    fn posix_dash(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Posix(&opts).render().unwrap();

        let assert = Command::new("dash").args(&["-c", &source]).assert().success().stderr("");
        if opts.hook != InitHook::Pwd {
            assert.stdout("");
        }
    }

    #[rstest]
    fn posix_shellcheck_(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Posix(&opts).render().unwrap();

        Command::new("shellcheck")
            .args(&["--enable", "all", "--shell", "sh", "-"])
            .write_stdin(source)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn posix_shfmt(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = Posix(&opts).render().unwrap();
        source.push('\n');

        Command::new("shfmt")
            .args(&["-d", "-s", "-ln", "posix", "-i", "4", "-ci", "-"])
            .write_stdin(source)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn powershell_pwsh(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = "Set-StrictMode -Version latest\n".to_string();
        Powershell(&opts).render_into(&mut source).unwrap();

        Command::new("pwsh")
            .args(&["-NoLogo", "-NonInteractive", "-NoProfile", "-Command", &source])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn xonsh_black(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = Xonsh(&opts).render().unwrap();
        source.push('\n');

        Command::new("black")
            .args(&["--check", "--diff", "-"])
            .write_stdin(source)
            .assert()
            .success()
            .stdout("");
    }

    #[rstest]
    fn xonsh_mypy(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Xonsh(&opts).render().unwrap();

        Command::new("mypy").args(&["--command", &source]).assert().success().stderr("");
    }

    #[rstest]
    fn xonsh_pylint(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let mut source = Xonsh(&opts).render().unwrap();
        source.push('\n');

        let tempdir = tempfile::tempdir().unwrap();
        let tempdir = tempdir.path().to_str().unwrap();

        Command::new("pylint")
            .args(&["--from-stdin", "zoxide"])
            .env("HOME", tempdir)
            .write_stdin(source)
            .assert()
            .success()
            .stderr("");
    }

    #[rstest]
    fn xonsh_xonsh(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Xonsh(&opts).render().unwrap();

        // We can't pass the source directly to `xonsh -c` due to
        // a bug: <https://github.com/xonsh/xonsh/issues/3959>
        Command::new("xonsh")
            .args(&[
                "-c",
                "import sys; execx(sys.stdin.read(), 'exec', __xonsh__.ctx, filename='zoxide')",
                "--no-rc",
            ])
            .write_stdin(source.as_bytes())
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    #[rstest]
    fn zsh_shellcheck(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Zsh(&opts).render().unwrap();

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

    #[rstest]
    fn zsh_zsh(
        #[values(None, Some("z"))] cmd: Option<&str>,
        #[values(InitHook::None, InitHook::Prompt, InitHook::Pwd)] hook: InitHook,
        #[values(false, true)] echo: bool,
        #[values(false, true)] resolve_symlinks: bool,
    ) {
        let opts = Opts { cmd, hook, echo, resolve_symlinks };
        let source = Zsh(&opts).render().unwrap();

        Command::new("zsh")
            .args(&["-c", &source, "--no-globalrcs", "--no-rcs"])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}
