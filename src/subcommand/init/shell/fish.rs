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
        set -l _zoxide_result (zoxide query $argv)

        if test -d $_zoxide_result; and string length -q -- $_zoxide_result
            _z_cd $_zoxide_result
            or return $status
        else if test -n "$_zoxide_result"
            echo $_zoxide_result
        end
    end
end
"#,
        cmd
    )
}

fn alias(cmd: &str) -> String {
    format!(
        r#"
abbr -a {0}i '{0} -i'

abbr -a {0}a 'zoxide add'

abbr -a {0}q 'zoxide query'
abbr -a {0}qi 'zoxide query -i'

abbr -a {0}r 'zoxide remove'
function {0}ri
    set result (zoxide query -i $argv)
    and zoxide remove $result
end
"#,
        cmd
    )
}

const HOOK_PROMPT: &str = r#"
function _zoxide_hook --on-event fish_prompt
    zoxide add
end
"#;

const fn hook_pwd() -> Result<Cow<'static, str>> {
    const HOOK_PWD: &str = r#"
function _zoxide_hook --on-variable PWD
    zoxide add
end
"#;

    Ok(Cow::Borrowed(HOOK_PWD))
}
