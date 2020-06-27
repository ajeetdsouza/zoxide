use super::{HookConfig, ShellConfig};

use anyhow::Result;

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
function {} {{
    function z_cd($dir) {{
        Set-Location $dir -ea Stop
        if ($env:_ZO_ECHO -eq "1") {{
            Write-Host "$PWD"
        }}
    }}

    if ($args.Length -eq 0) {{
        z_cd ~
    }}
    elseif ($args.Length -eq 1 -and $args[0] -eq '-') {{
        z_cd -
    }}
    else {{
        $_zoxide_result = zoxide query @args
        if ($LASTEXITCODE -eq 0 -and $_zoxide_result -is [string] -and (Test-Path $_zoxide_result)) {{
            z_cd $_zoxide_result
        }} else {{
            $_zoxide_result
        }}
    }}
}}
"#,
        cmd
    )
}

fn alias(cmd: &str) -> String {
    format!(
        r#"
function {0}i {{ {0} -i @args }}

function {0}a {{ zoxide add @args }}

function {0}q {{ zoxide query @args }}
function {0}qi {{ zoxide query -i @args }}

function {0}r {{ zoxide remove @args }}
function {0}ri {{ zoxide remove -i @args }}
"#,
        cmd
    )
}

const HOOK_PROMPT: &str = r#"
$PreZoxidePrompt = $function:prompt
function prompt {
    $null = zoxide add
    & $PreZoxidePrompt
}
"#;

const fn hook_pwd() -> Result<Cow<'static, str>> {
    const HOOK_PWD: &str = r#"
if ($PSVersionTable.PSVersion.Major -ge 6) {
    $ExecutionContext.InvokeCommand.LocationChangedAction = {
        $null = zoxide add
    }
} else {
    Write-Error "pwd hook requires pwsh - use 'zoxide init powershell --hook prompt'"
}
"#;

    Ok(Cow::Borrowed(HOOK_PWD))
}
