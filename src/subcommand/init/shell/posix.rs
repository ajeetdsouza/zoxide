use super::{HookConfig, ShellConfig};
use crate::util;

use anyhow::Result;
use uuid::Uuid;

use std::borrow::Cow;

pub const CONFIG: ShellConfig = ShellConfig {
    z,
    alias,
    hook: HookConfig {
        prompt: HOOK_PROMPT,
        pwd: hook_pwd,
    },
};

fn z(cmd: &str) -> String {
    format!(
        r#"
_z_cd() {{
    cd "$@" || return "$?"

    if [ "$_ZO_ECHO" = "1" ]; then
        echo "$PWD"
    fi
}}

{0}() {{
    if [ "$#" -eq 0 ]; then
        _z_cd ~
    elif [ "$#" -eq 1 ] && [ "$1" = '-' ]; then
        if [ -n "$OLDPWD" ]; then
            _z_cd "$OLDPWD"
        else
            echo 'zoxide: $OLDPWD is not set'
            return 1
        fi
    else
        _zoxide_result="$(zoxide query -- "$@")" && _z_cd "$_zoxide_result"
    fi
}}

{0}i() {{
    _zoxide_result="$(zoxide query -i -- "$@")" && _z_cd "$_zoxide_result"
}}
"#,
        cmd
    )
}

fn alias(cmd: &str) -> String {
    format!(
        r#"
alias {0}a='zoxide add'

alias {0}q='zoxide query'
alias {0}qi='zoxide query -i'

alias {0}r='zoxide remove'
{0}ri() {{
    _zoxide_result="$(zoxide query -i -- "$@")" && zoxide remove "$_zoxide_result"
}}
"#,
        cmd
    )
}

const HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add "$(pwd -L)"
}

case "$PS1" in
    *\$\(_zoxide_hook\)*) ;;
    *) PS1="\$(_zoxide_hook)${PS1}" ;;
esac
"#;

fn hook_pwd() -> Result<Cow<'static, str>> {
    let mut tmp_path = std::env::temp_dir();
    tmp_path.push("zoxide");
    let tmp_path_str = util::path_to_str(&tmp_path)?;

    let pwd_path = tmp_path.join(format!("pwd-{}", Uuid::new_v4()));
    let pwd_path_str = util::path_to_str(&pwd_path)?;

    let hook_pwd = format!(
        r#"
# PWD hooks in POSIX use a temporary file, located at `$_ZO_PWD_PATH`, to track
# changes in the current directory. These files are removed upon restart,
# but they should ideally also be cleaned up once the shell exits using traps.
#
# This can be done as follows:
#
# trap '_zoxide_cleanup' EXIT HUP KILL TERM
# trap '_zoxide_cleanup; trap - INT; kill -s INT "$$"' INT
# trap '_zoxide_cleanup; trap - QUIT; kill -s QUIT "$$"' QUIT
#
# By default, traps are not set up because they override all previous traps.
# It is therefore up to the user to add traps to their shell configuration.

_ZO_TMP_PATH={}
_ZO_PWD_PATH={}

_zoxide_cleanup() {{
    rm -f "$_ZO_PWD_PATH"
}}

_zoxide_setpwd() {{
    mkdir -p "$_ZO_TMP_PATH"
    echo "$PWD" > "$_ZO_PWD_PATH"
}}

_zoxide_setpwd

_zoxide_hook() {{
    _ZO_OLDPWD="$(cat "$_ZO_PWD_PATH")"
    if [ -z "$_ZO_OLDPWD" ] || [ "$_ZO_OLDPWD" != "$PWD" ]; then
        _zoxide_setpwd && zoxide add "$(pwd -L)" > /dev/null
    fi
}}

case "$PS1" in
    *\$\(_zoxide_hook\)*) ;;
    *) PS1="\$(_zoxide_hook)${{PS1}}" ;;
esac"#,
        quote(tmp_path_str),
        quote(pwd_path_str),
    );

    Ok(Cow::Owned(hook_pwd))
}

fn quote(string: &str) -> String {
    let mut quoted = String::with_capacity(string.len() + 2);

    quoted.push('\'');
    for ch in string.chars() {
        match ch {
            '\\' => quoted.push_str(r"\\"),
            '\'' => quoted.push_str(r"'\''"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('\'');

    quoted
}
