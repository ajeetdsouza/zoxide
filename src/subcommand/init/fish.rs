use anyhow::Result;

use std::io::Write;

use super::{Hook, Init};
use crate::config;

pub fn run<W: Write>(writer: &mut W, options: &Init) -> Result<()> {
    const NOT_CONFIGURED: &str = "\
# -- not configured --";

    let __zoxide_pwd = if config::zo_resolve_symlinks() {
        "\
function __zoxide_pwd
    pwd -P
end"
    } else {
        "\
function __zoxide_pwd
    pwd -L
end"
    };

    let __zoxide_cd = if config::zo_echo() {
        "\
function __zoxide_cd
    cd $argv
    or return $status

    commandline -f repaint
    __zoxide_pwd
end"
    } else {
        "\
function __zoxide_cd
    cd $argv
    or return $status

    commandline -f repaint
end"
    };

    let __zoxide_hook = "\
function __zoxide_hook
    zoxide add (__zoxide_pwd)
end";

    let hook_init = match options.hook {
        Hook::none => NOT_CONFIGURED,
        Hook::prompt => {
            "\
function __zoxide_hook_prompt --on-event fish_prompt
    __zoxide_hook
end"
        }
        Hook::pwd => {
            "\
function __zoxide_hook_pwd --on-variable PWD
    __zoxide_hook
end"
        }
    };

    let aliases = if options.no_aliases {
        NOT_CONFIGURED.into()
    } else {
        format!(
            "\
function {cmd}
    __zoxide_z $argv
end

function {cmd}i
    __zoxide_zi $argv
end

function {cmd}a
    __zoxide_za $argv
end

function {cmd}q
    __zoxide_zq $argv
end

function {cmd}qi
    __zoxide_zqi $argv
end

function {cmd}r
    __zoxide_zr $argv
end

function {cmd}ri
    __zoxide_zri $argv
end",
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
function __zoxide_z
    set argc (count $argv)

    if test $argc -eq 0
        __zoxide_cd $HOME
    else if begin; test $argc -eq 1; and test $argv[1] = '-'; end
        __zoxide_cd -
    else
        set -l __zoxide_result (zoxide query -- $argv)
        and __zoxide_cd $__zoxide_result
    end
end

# Jump to a directory using interactive search.
function __zoxide_zi
    set -l __zoxide_result (zoxide query -i -- $argv)
    and __zoxide_cd $__zoxide_result
end

# Add a new entry to the database.
abbr -a __zoxide_za 'zoxide add'

# Query an entry from the database using only keywords.
abbr -a __zoxide_zq 'zoxide query'

# Query an entry from the database using interactive selection.
abbr -a __zoxide_zqi 'zoxide query -i'

# Remove an entry from the database using the exact path.
abbr -a __zoxide_zr 'zoxide remove'

# Remove an entry from the database using interactive selection.
function __zoxide_zri
    set -l __zoxide_result (zoxide query -i -- $argv)
    and zoxide remove $__zoxide_result
end

# =============================================================================
#
# Convenient aliases for zoxide. Disable these using --no-aliases.
#

{aliases}

# =============================================================================
#
# To initialize zoxide with fish, add the following line to your fish
# configuration file (usually ~/.config/fish/config.fish):
#
# zoxide init fish | source
",
        __zoxide_pwd = __zoxide_pwd,
        __zoxide_cd = __zoxide_cd,
        __zoxide_hook = __zoxide_hook,
        hook_init = hook_init,
        aliases = aliases,
    )?;

    Ok(())
}
