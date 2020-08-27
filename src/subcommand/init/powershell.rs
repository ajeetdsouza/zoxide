use anyhow::Result;

use std::io::Write;

use super::{Hook, Init};
use crate::config;

pub fn run<W: Write>(writer: &mut W, options: &Init) -> Result<()> {
    const NOT_CONFIGURED: &str = "\
# -- not configured --";

    let __zoxide_pwd = "\
function __zoxide_pwd {
    $(Get-Location).Path
}";

    let __zoxide_cd = if config::zo_echo() {
        "\
function __zoxide_cd($dir) {
    Set-Location $dir -ea Stop
    __zoxide_pwd
}"
    } else {
        "\
function __zoxide_cd($dir) {
    Set-Location $dir -ea Stop
}"
    };

    let __zoxide_hook = "\
function __zoxide_hook {
    zoxide add $(__zoxide_pwd)
}";

    let hook_init = match options.hook {
        Hook::none => NOT_CONFIGURED,
        Hook::prompt => {
            "\
$PreZoxidePrompt = $function:prompt
function prompt {
    $null = __zoxide_hook
    & $PreZoxidePrompt
}"
        }
        Hook::pwd => {
            "\
if ($PSVersionTable.PSVersion.Major -ge 6) {
    $ExecutionContext.InvokeCommand.LocationChangedAction = {
        $null = __zoxide_hook
    }
} else {
    Write-Error \"zoxide: PWD hooks are not supported below PowerShell 6, use 'zoxide init powershell --hook prompt' instead.\"
}"
        }
    };

    let aliases = if options.no_aliases {
        NOT_CONFIGURED.into()
    } else {
        format!(
            "\
Set-Alias {cmd} __zoxide_z
Set-Alias {cmd}i __zoxide_zi

Set-Alias {cmd}a __zoxide_za

Set-Alias {cmd}q __zoxide_zq
Set-Alias {cmd}qi __zoxide_zqi

Set-Alias {cmd}r __zoxide_zr
Set-Alias {cmd}ri __zoxide_zri",
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
function __zoxide_z {{
    if ($args.Length -eq 0) {{
        __zoxide_cd ~
    }}
    elseif ($args.Length -eq 1 -and $args[0] -eq '-') {{
        __zoxide_cd -
    }}
    else {{
        $__zoxide_result = zoxide query -- @args
        if ($LASTEXITCODE -eq 0) {{
            __zoxide_cd $__zoxide_result
        }}
    }}
}}

# Jump to a directory using interactive search.
function zi {{
    $__zoxide_result = zoxide query -i -- @args
    if ($LASTEXITCODE -eq 0) {{
        __zoxide_cd $__zoxide_result
    }}
}}

# Add a new entry to the database.
function __zoxide_za {{ zoxide add @args }}

# Query an entry from the database using only keywords.
function __zoxide_zq {{ zoxide query @args }}

# Query an entry from the database using interactive selection.
function __zoxide_zqi {{ zoxide query -i @args }}

# Remove an entry from the database using the exact path.
function __zoxide_zr {{ zoxide remove @args }}

# Remove an entry from the database using interactive selection.
function __zoxide_zri {{
    $_zoxide_result = zoxide query -i -- @args
    if ($LASTEXITCODE -eq 0) {{
        zoxide remove $_zoxide_result
    }}
}}

# =============================================================================
#
# Convenient aliases for zoxide. Disable these using --no-aliases.
#

{aliases}

# =============================================================================
#
# To initialize zoxide with PowerShell, add the following line to your
# PowerShell configuration file (the location is stored in $profile):
#
# Invoke-Expression (& {{
#     $hook = if ($PSVersionTable.PSVersion.Major -ge 6) {{
#         'pwd'
#     }} else {{
#         'prompt'
#     }}
#     (zoxide init powershell --hook $hook) -join \"`n\"
# }})
",
        __zoxide_pwd = __zoxide_pwd,
        __zoxide_cd = __zoxide_cd,
        __zoxide_hook = __zoxide_hook,
        hook_init = hook_init,
        aliases = aliases,
    )?;

    Ok(())
}
