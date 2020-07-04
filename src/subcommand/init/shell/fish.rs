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
function _z_cd
    cd $argv
    or return $status

    commandline -f repaint

    if test "$_ZO_ECHO" = "1"
        echo $PWD
    end
end

function {0}
    set argc (count $argv)

    if test $argc -eq 0
        _z_cd $HOME
    else if begin; test $argc -eq 1; and test $argv[1] = '-'; end
        _z_cd -
    else
        set -l _zoxide_result (zoxide query -- $argv)
        and _z_cd $_zoxide_result
    end
end

function {0}i
    set -l _zoxide_result (zoxide query -i -- $argv)
    and _z_cd $_zoxide_result
end
"#,
        cmd
    )
}

fn alias(cmd: &str) -> String {
    format!(
        r#"
abbr -a {0}a 'zoxide add'

abbr -a {0}q 'zoxide query'
abbr -a {0}qi 'zoxide query -i'

abbr -a {0}r 'zoxide remove'
function {0}ri
    set -l _zoxide_result (zoxide query -i -- $argv)
    and zoxide remove $_zoxide_result
end
"#,
        cmd
    )
}

const HOOK_PROMPT: &str = r#"
function _zoxide_hook --on-event fish_prompt
    zoxide add (pwd -L)
end
"#;

const fn hook_pwd() -> Result<Cow<'static, str>> {
    const HOOK_PWD: &str = r#"
function _zoxide_hook --on-variable PWD
    zoxide add (pwd -L)
end
"#;

    Ok(Cow::Borrowed(HOOK_PWD))
}
