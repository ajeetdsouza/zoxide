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
        help = "Changes the name of the 'z' command",
        default_value = "z"
    )]
    z_cmd: String,

    #[structopt(
        long,
        help = "Prevents zoxide from defining any commands other than 'z'"
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

        let z = config.z;
        writeln!(handle, "{}", z(&self.z_cmd)).unwrap();

        if !self.no_define_aliases {
            let alias = config.alias;
            writeln!(handle, "{}", alias(&self.z_cmd)).unwrap();
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
    z: bash_z,
    alias: bash_alias,
    hook: HookConfig {
        prompt: BASH_HOOK_PROMPT,
        pwd: Some(BASH_HOOK_PWD),
    },
};

const FISH_CONFIG: ShellConfig = ShellConfig {
    z: fish_z,
    alias: fish_alias,
    hook: HookConfig {
        prompt: FISH_HOOK_PROMPT,
        pwd: Some(FISH_HOOK_PWD),
    },
};

const POSIX_CONFIG: ShellConfig = ShellConfig {
    z: posix_z,
    alias: posix_alias,
    hook: HookConfig {
        prompt: POSIX_HOOK_PROMPT,
        pwd: None,
    },
};

const ZSH_CONFIG: ShellConfig = ShellConfig {
    z: zsh_z,
    alias: zsh_alias,
    hook: HookConfig {
        prompt: ZSH_HOOK_PROMPT,
        pwd: Some(ZSH_HOOK_PWD),
    },
};

struct ShellConfig {
    z: fn(&str) -> String,
    alias: fn(&str) -> String,
    hook: HookConfig,
}

struct HookConfig {
    prompt: &'static str,
    pwd: Option<&'static str>,
}

fn fish_z(z_cmd: &str) -> String {
    format!(
        r#"
function _z_cd
    cd $argv
    or return $status

    commandline -f repaint

    if test -n "$_ZO_ECHO"
        echo $PWD
    end
end

function {}
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
"#,
        z_cmd
    )
}

fn posix_z(z_cmd: &str) -> String {
    format!(
        r#"
_z_cd() {{
    cd "$@" || return "$?"

    if [ -n "$_ZO_ECHO" ]; then
        echo "$PWD"
    fi
}}

{}() {{
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
}}
"#,
        z_cmd
    )
}

use posix_z as bash_z;
use posix_z as zsh_z;

fn fish_alias(z_cmd: &str) -> String {
    format!(
        r#"
abbr -a zi '{} -i'
abbr -a za 'zoxide add'
abbr -a zq 'zoxide query'
abbr -a zr 'zoxide remove'
"#,
        z_cmd
    )
}

fn posix_alias(z_cmd: &str) -> String {
    format!(
        r#"
alias zi='{} -i'
alias za='zoxide add'
alias zq='zoxide query'
alias zr='zoxide remove'
"#,
        z_cmd
    )
}

use posix_alias as bash_alias;
use posix_alias as zsh_alias;

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
