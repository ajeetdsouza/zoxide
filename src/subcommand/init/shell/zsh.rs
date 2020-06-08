use super::{posix, HookConfig, ShellConfig};

use anyhow::Result;

use std::borrow::Cow;

pub const CONFIG: ShellConfig = ShellConfig {
    z: posix::CONFIG.z,
    alias: posix::CONFIG.alias,
    hook: HookConfig {
        prompt: HOOK_PROMPT,
        pwd: hook_pwd,
    },
};

const HOOK_PROMPT: &str = r#"
_zoxide_hook() {
    zoxide add "$(pwd -L)"
}

[[ -n "${precmd_functions[(r)_zoxide_hook]}" ]] || {
    precmd_functions+=(_zoxide_hook)
}
"#;

const fn hook_pwd() -> Result<Cow<'static, str>> {
    const HOOK_PWD: &str = r#"
_zoxide_hook() {
    zoxide add "$(pwd -L)"
}

chpwd_functions=(${chpwd_functions[@]} "_zoxide_hook")
"#;

    Ok(Cow::Borrowed(HOOK_PWD))
}
