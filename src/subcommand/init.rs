use anyhow::{bail, Result};
use clap::arg_enum;
use structopt::StructOpt;

use std::io::{self, Write};

#[derive(Debug, StructOpt)]
#[structopt(about = "Generates shell configuration")]
pub struct Init {
    #[structopt(possible_values = &Shell::variants(), case_insensitive = true)]
    shell: Shell,

    #[structopt(
        long,
        help = "Prevents zoxide from defining any aliases other than 'z'"
    )]
    no_define_aliases: bool,

    #[structopt(
        long,
        help = "Chooses event on which an entry is added to the database",
        possible_values = &Hook::variants(),
        default_value = "prompt",
        case_insensitive = true
    )]
    hook: Hook,
}

impl Init {
    pub fn run(&self) -> Result<()> {
        let config = match self.shell {
            Shell::bash => BASH_CONFIG,
            Shell::fish => FISH_CONFIG,
            Shell::posix => POSIX_CONFIG,
            Shell::zsh => ZSH_CONFIG,
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // If any `writeln!` call fails to write to stdout, we assume the user's
        // computer is on fire and panic.
        writeln!(handle, "{}", config.z).unwrap();
        if !self.no_define_aliases {
            writeln!(handle, "{}", config.alias).unwrap();
        }

        match self.hook {
            Hook::none => (),
            Hook::prompt => writeln!(handle, "{}", config.hook.prompt).unwrap(),
            Hook::pwd => match config.hook.pwd {
                Some(pwd_hook) => writeln!(handle, "{}", pwd_hook).unwrap(),
                None => bail!("PWD hooks are currently unsupported on this shell."),
            },
        }

        Ok(())
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Shell {
        bash,
        fish,
        posix,
        zsh,
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Hook {
        none,
        prompt,
        pwd,
    }
}

const BASH_CONFIG: ShellConfig = ShellConfig {
    z: BASH_Z,
    alias: BASH_ALIAS,
    hook: HookConfig {
        prompt: BASH_HOOK_PROMPT,
        pwd: Some(BASH_HOOK_PWD),
    },
};

const FISH_CONFIG: ShellConfig = ShellConfig {
    z: FISH_Z,
    alias: FISH_ALIAS,
    hook: HookConfig {
        prompt: FISH_HOOK_PROMPT,
        pwd: Some(FISH_HOOK_PWD),
    },
};

const POSIX_CONFIG: ShellConfig = ShellConfig {
    z: POSIX_Z,
    alias: POSIX_ALIAS,
    hook: HookConfig {
        prompt: POSIX_HOOK_PROMPT,
        pwd: None,
    },
};

const ZSH_CONFIG: ShellConfig = ShellConfig {
    z: ZSH_Z,
    alias: ZSH_ALIAS,
    hook: HookConfig {
        prompt: ZSH_HOOK_PROMPT,
        pwd: Some(ZSH_HOOK_PWD),
    },
};

struct ShellConfig {
    z: &'static str,
    alias: &'static str,
    hook: HookConfig,
}

struct HookConfig {
    prompt: &'static str,
    pwd: Option<&'static str>,
}

const BASH_Z: &str = r#"
_z_cd() {
    cd "$@" || return "$?"

    if [ -n "$_ZO_ECHO" ]; then
        echo "$PWD"
    fi
}

z() {
    if [ "$#" -eq 0 ]; then
        _z_cd ~ || return "$?"
    elif [ "$#" -eq 1 ] && [ "$1" = '-' ]; then
        if [ -n "$OLDPWD" ]; then
            _z_cd "$OLDPWD" || return "$?"
        else
            echo 'zoxide: $OLDPWD is not set'
            return 1
        fi
    else
        result="$(zoxide query "$@")" || return "$?"
        if [ -d "$result" ]; then
            _z_cd "$result" || return "$?"
        elif [ -n "$result" ]; then
            echo "$result"
        fi
    fi
}
"#;

const FISH_Z: &str = r#"
function _z_cd
    cd $argv
    or return $status

    commandline -f repaint

    if test -n "$_ZO_ECHO"
        echo $PWD
    end
end

function z
    set argc (count $argv)

    if test $argc -eq 0
        _z_cd $HOME
        or return $status

    else if test $argc -eq 1 -a $argv[1] = '-'
        _z_cd -
        or return $status

    else
        # FIXME: use string-collect from fish 3.1.0 once it has wider adoption
        set -l IFS ''
        set -l result (zoxide query $argv)

        if test -d $result
            _z_cd $result
            or return $status
        else if test -n "$result"
            echo $result
        end
    end
end
"#;

const POSIX_Z: &str = BASH_Z;

const ZSH_Z: &str = BASH_Z;

const BASH_ALIAS: &str = r#"
alias zi='z -i'
alias za='zoxide add'
alias zq='zoxide query'
alias zr='zoxide remove'
"#;

const FISH_ALIAS: &str = r#"
abbr -a zi 'z -i'
abbr -a za 'zoxide add'
abbr -a zq 'zoxide query'
abbr -a zr 'zoxide remove'
"#;

const POSIX_ALIAS: &str = BASH_ALIAS;

const ZSH_ALIAS: &str = BASH_ALIAS;

const BASH_HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add
}

case "$PROMPT_COMMAND" in
    *_zoxide_hook*) ;;
    *) PROMPT_COMMAND="_zoxide_hook${PROMPT_COMMAND:+;${PROMPT_COMMAND}}" ;;
esac
"#;

const FISH_HOOK_PROMPT: &str = r#"
function _zoxide_hook --on-event fish_prompt
    zoxide add
end
"#;

const POSIX_HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add > /dev/null
}

case "$PS1" in
    *\$\(_zoxide_hook\)*) ;;
    *) PS1="\$(_zoxide_hook)${PS1}" ;;
esac
"#;

const ZSH_HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add
}

[[ -n "${precmd_functions[(r)_zoxide_hook]}" ]] || {
    precmd_functions+=(_zoxide_hook)
}
"#;

const BASH_HOOK_PWD: &str = r#"
_zoxide_hook() {
    if [ -z "${_ZO_PWD}" ]; then
        _ZO_PWD="${PWD}"
    elif [ "${_ZO_PWD}" != "${PWD}" ]; then
        _ZO_PWD="${PWD}"
        zoxide add
    fi
}

case "$PROMPT_COMMAND" in
    *_zoxide_hook*) ;;
    *) PROMPT_COMMAND="_zoxide_hook${PROMPT_COMMAND:+;${PROMPT_COMMAND}}" ;;
esac
"#;

const FISH_HOOK_PWD: &str = r#"
function _zoxide_hook --on-variable PWD
    zoxide add
end
"#;

const ZSH_HOOK_PWD: &str = r#"
_zoxide_hook() {
    zoxide add
}

chpwd_functions=(${chpwd_functions[@]} "_zoxide_hook")
"#;
