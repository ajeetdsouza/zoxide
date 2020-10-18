use anyhow::{Context, Result};
use askama::Template;

use std::io::Write;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hook {
    None,
    Prompt,
    Pwd,
}

pub trait Generator {
    fn generate<W: Write>(&self, writer: &mut W) -> Result<()>;
}

impl<T: Template> Generator for T {
    fn generate<W: Write>(&self, writer: &mut W) -> Result<()> {
        let source = &self.render().context("could not render template")?;
        writeln!(writer, "{}", source).context("could not write to output")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Opts<'a> {
    pub cmd: Option<&'a str>,
    pub hook: Hook,
    pub echo: bool,
    pub resolve_symlinks: bool,
}

#[derive(Debug, Template)]
#[template(path = "bash.txt")]
pub struct Bash<'a>(pub &'a Opts<'a>);

impl<'a> Deref for Bash<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug, Template)]
#[template(path = "fish.txt")]
pub struct Fish<'a>(pub &'a Opts<'a>);

impl<'a> Deref for Fish<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug, Template)]
#[template(path = "posix.txt")]
pub struct Posix<'a>(pub &'a Opts<'a>);

impl<'a> Deref for Posix<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug, Template)]
#[template(path = "powershell.txt")]
pub struct PowerShell<'a>(pub &'a Opts<'a>);

impl<'a> Deref for PowerShell<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug, Template)]
#[template(path = "xonsh.txt")]
pub struct Xonsh<'a>(pub &'a Opts<'a>);

impl<'a> Deref for Xonsh<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
#[derive(Debug, Template)]
#[template(path = "zsh.txt")]
pub struct Zsh<'a>(pub &'a Opts<'a>);

impl<'a> Deref for Zsh<'a> {
    type Target = Opts<'a>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
