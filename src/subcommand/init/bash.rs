use anyhow::Result;

use std::io::Write;

use super::{Hook, Init};
use crate::config;

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
        Hook::none => NOT_CONFIGURED,
        Hook::prompt => {
            "\
__zoxide_hook() {
    zoxide add \"$(__zoxide_pwd)\"
}"
        }
        Hook::pwd => {
            "\
__zoxide_hook() {
    local -r __zoxide_pwd_tmp=\"$(__zoxide_pwd)\"
    if [ -z \"$__zoxide_pwd_old\" ]; then
        __zoxide_pwd_old=\"$__zoxide_pwd_tmp\"
    elif [ \"$__zoxide_pwd_old\" != \"$__zoxide_pwd_tmp\" ]; then
        __zoxide_pwd_old=\"$__zoxide_pwd_tmp\"
        zoxide add \"$__zoxide_pwd_old\"
    fi
}"
        }
    };

    let hook_init = match options.hook {
        Hook::none => NOT_CONFIGURED,
        _ => {
            "\
case \"$PROMPT_COMMAND\" in
    *__zoxide_hook*) ;;
    *) PROMPT_COMMAND=\"${PROMPT_COMMAND:+${PROMPT_COMMAND};}__zoxide_hook\" ;;
esac"
        }
    };

    let aliases = if options.no_aliases {
        NOT_CONFIGURED.into()
    } else {
        format!(
            "\
alias {}='__zoxide_z'
alias {cmd}i='__zoxide_zi'
alias {cmd}a='__zoxide_za'

alias {cmd}q='__zoxide_zq'
alias {cmd}qi='__zoxide_zqi'

alias {cmd}r='__zoxide_zr'
alias {cmd}ri='__zoxide_zri'",
            cmd = options.cmd
        )
    };

    write!(
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
        local __zoxide_result
        __zoxide_result=\"$(zoxide query -- \"$@\")\" && __zoxide_cd \"$__zoxide_result\"
    fi
}}

# Jump to a directory using interactive search.
__zoxide_zi() {{
    local __zoxide_result
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
    local __zoxide_result
    __zoxide_result=\"$(zoxide query -i -- \"$@\")\" && zoxide remove \"$__zoxide_result\"
}}

# =============================================================================
#
# Convenient aliases for zoxide. Disable these using --no-aliases.
#

{aliases}

# =============================================================================
#
# To initialize zoxide with bash, add the following line to your bash
# configuration file (usually ~/.bashrc):
#
# eval \"$(zoxide init bash)\"
",
        __zoxide_pwd = __zoxide_pwd,
        __zoxide_cd = __zoxide_cd,
        __zoxide_hook = __zoxide_hook,
        hook_init = hook_init,
        aliases = aliases,
    )?;

    Ok(())
}
