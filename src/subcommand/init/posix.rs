use super::{Hook, Init};
use crate::config;
use crate::util;

use std::io::Write;

use anyhow::Result;
use uuid::Uuid;

pub fn run<W: Write>(writer: &mut W, options: &Init) -> Result<()> {
    const NOT_CONFIGURED: &str = "\
# -- not configured --";

    let __zoxide_pwd = if config::zo_resolve_symlinks() {
        "\
__zoxide_pwd() {
    pwd -P
}"
    } else {
        "\
__zoxide_pwd() {
    pwd -L
}"
    };

    let __zoxide_cd = if config::zo_echo() {
        "\
__zoxide_cd() {
    cd \"$@\" || return \"$?\"
    __zoxide_pwd
}"
    } else {
        "\
__zoxide_cd() {
    cd \"$@\" || return \"$?\"
}"
    };

    let __zoxide_hook = match options.hook {
        Hook::none => NOT_CONFIGURED.into(),
        Hook::prompt => "\
__zoxide_hook() {
    zoxide add \"$(__zoxide_pwd)\"
}"
        .into(),
        Hook::pwd => {
            let mut tmp_path = std::env::temp_dir();
            tmp_path.push("zoxide");
            let tmp_path_str = util::path_to_str(&tmp_path)?;

            let pwd_path = tmp_path.join(format!("pwd-{}", Uuid::new_v4()));
            let pwd_path_str = util::path_to_str(&pwd_path)?;

            format!(
                "\
# PWD hooks in POSIX use a temporary file, located at `$__zoxide_pwd_path`, to track
# changes in the current directory. These files are removed upon restart,
# but they should ideally also be cleaned up once the shell exits using traps.
#
# This can be done as follows:
#
# trap '__zoxide_cleanup' EXIT HUP KILL TERM
# trap '__zoxide_cleanup; trap - INT; kill -s INT \"$$\"' INT
# trap '__zoxide_cleanup; trap - QUIT; kill -s QUIT \"$$\"' QUIT
#
# By default, traps are not set up because they override all previous traps.
# It is therefore up to the user to add traps to their shell configuration.

__zoxide_tmp_path={tmp_path}
__zoxide_pwd_path={pwd_path}

__zoxide_cleanup() {{
    rm -f \"$__zoxide_pwd_path\"
}}

__zoxide_setpwd() {{
    mkdir -p \"$__zoxide_tmp_path\"
    echo \"$PWD\" > \"$__zoxide_pwd_path\"
}}

__zoxide_setpwd

__zoxide_hook() {{
    _ZO_OLDPWD=\"$(cat \"$__zoxide_pwd_path\")\"
    if [ -z \"$_ZO_OLDPWD\" ] || [ \"$_ZO_OLDPWD\" != \"$PWD\" ]; then
        __zoxide_setpwd && zoxide add \"$(pwd -L)\" > /dev/null
    fi
}}",
                tmp_path = posix_quote(tmp_path_str),
                pwd_path = posix_quote(pwd_path_str),
            )
        }
    };

    let hook_init = match options.hook {
        Hook::none => NOT_CONFIGURED,
        _ => {
            "\
case \"$PS1\" in
    *\\$\\(__zoxide_hook\\)*) ;;
    *) PS1=\"${PS1}\\$(__zoxide_hook)\" ;;
esac"
        }
    };

    let aliases = if options.no_aliases {
        NOT_CONFIGURED.into()
    } else {
        format!(
            "\
alias {cmd}='__zoxide_z'
alias {cmd}i='__zoxide_zi'

alias {cmd}a='__zoxide_za'

alias {cmd}q='__zoxide_zq'
alias {cmd}qi='__zoxide_zqi'

alias {cmd}r='__zoxide_zr'
alias {cmd}ri='__zoxide_zri'",
            cmd = options.cmd
        )
    };

    writeln!(
        writer,
        "\
# =============================================================================
#
# Utility functions for zoxide.
#

# pwd based on the value of _ZO_RESOLVE_SYMLINKS.
{__zoxide_pwd}

# cd + custom logic based on the value of _ZO_ECHO.
{__zoxide_cd}

# =============================================================================
#
# Hook configuration for zoxide.
#

# Hook to add new entries to the database.
{__zoxide_hook}

# Initialize hook.
{hook_init}

# =============================================================================
#
# When using zoxide with --no-aliases, alias these internal functions as
# desired.
#

# Jump to a directory using only keywords.
__zoxide_z() {{
    if [ \"$#\" -eq 0 ]; then
        __zoxide_cd ~
    elif [ \"$#\" -eq 1 ] && [ \"$1\" = '-' ]; then
        if [ -n \"$OLDPWD\" ]; then
            __zoxide_cd \"$OLDPWD\"
        else
            echo \"zoxide: \\$OLDPWD is not set\"
            return 1
        fi
    else
        __zoxide_result=\"$(zoxide query -- \"$@\")\" && __zoxide_cd \"$__zoxide_result\"
    fi
}}

# Jump to a directory using interactive search.
__zoxide_zi() {{
    __zoxide_result=\"$(zoxide query -i -- \"$@\")\" && __zoxide_cd \"$__zoxide_result\"
}}

# Add a new entry to the database.
alias __zoxide_za='zoxide add'

# Query an entry from the database using only keywords.
alias __zoxide_zq='zoxide query'

# Query an entry from the database using interactive selection.
alias __zoxide_zqi='zoxide query -i'

# Remove an entry from the database using the exact path.
alias __zoxide_zr='zoxide remove'

# Remove an entry from the database using interactive selection.
__zoxide_zri() {{
    __zoxide_result=\"$(zoxide query -i -- \"$@\")\" && zoxide remove \"$__zoxide_result\"
}}

# =============================================================================
#
# Convenient aliases for zoxide. Disable these using --no-aliases.
#

{aliases}

# =============================================================================
#
# To initialize zoxide with your POSIX shell, add the following line to your
# shell configuration file:
#
# eval \"$(zoxide init posix --prompt hook)\"
",
        __zoxide_pwd = __zoxide_pwd,
        __zoxide_cd = __zoxide_cd,
        __zoxide_hook = __zoxide_hook,
        hook_init = hook_init,
        aliases = aliases,
    )?;

    Ok(())
}

fn posix_quote(string: &str) -> String {
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
