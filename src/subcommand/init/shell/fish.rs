use super::{ShellConfig, HookConfig};

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
        set -l result (zoxide query $argv)

        if test -d $result; and string length -q -- $result
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

fn alias(z_cmd: &str) -> String {
    format!(
        r#"
abbr -a zi '{} -i'

abbr -a za 'zoxide add'

abbr -a zq 'zoxide query'
abbr -a zqi 'zoxide query -i'

abbr -a zr 'zoxide remove'
abbr -a zri 'zoxide remove -i'
"#,
        z_cmd
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
