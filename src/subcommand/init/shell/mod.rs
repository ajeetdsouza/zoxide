pub mod bash;
pub mod fish;
pub mod posix;
pub mod zsh;

use anyhow::Result;

use std::borrow::Cow;

pub struct ShellConfig {
    pub z: fn(&str) -> String,
    pub alias: fn(&str) -> String,
    pub hook: HookConfig,
}

pub struct HookConfig {
    pub prompt: &'static str,
    pub pwd: fn() -> Result<Cow<'static, str>>,
}
