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

fn z(z_cmd: &str) -> String {
    format!(
        r#"
function {} {{
    function z_cd($dir) {{
        try {{
            Set-Location $dir -ea Stop
            if ($env:_ZO_ECHO -eq "1") {{
                Write-Host "$PWD"
            }}
        }} catch {{
        }}
    }}

    if ($args.Length -eq 0) {{
        z_cd ~
    }}
    elseif ($args.Length -eq 1 -and $args[0] -eq '-') {{
        z_cd -
    }}
    else {{
        $result = zoxide query @args
        if ($LASTEXITCODE -eq 0 -and ((Test-Path $result) -eq $true)) {{
            z_cd $result
        }} else {{
            $result
        }}
    }}
}}
"#,
        z_cmd
    )
}

fn alias(z_cmd: &str) -> String {
    format!(
        r#"
function zi {{ {} -i @args }}

function za {{ zoxide add @args }}

function zq {{ zoxide query @args }}
function zqi {{ zoxide query -i @args }}

function zr {{ zoxide remove @args }}
function zri {{ zoxide remove -i @args }}
"#,
        z_cmd
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
$ExecutionContext.InvokeCommand.LocationChangedAction = {
    $null = zoxide add
}
"#;

    Ok(Cow::Borrowed(HOOK_PWD))
}

