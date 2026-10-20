#![allow(clippy::module_inception)]

use std::path::PathBuf;

use clap::builder::{IntoResettable, Resettable, StyledStr};
use clap::{Parser, Subcommand, ValueEnum, ValueHint};

struct HelpTemplate;

impl IntoResettable<StyledStr> for HelpTemplate {
    fn into_resettable(self) -> Resettable<StyledStr> {
        color_print::cstr!("\
{before-help}<bold><underline>{name} {version}</underline></bold>
{author}
https://github.com/ajeetdsouza/zoxide

{about}

{usage-heading}
{tab}{usage}

{all-args}{after-help}

<bold><underline>Environment variables:</underline></bold>
{tab}<bold>_ZO_DATA_DIR</bold>        {tab}Path for zoxide data files
{tab}<bold>_ZO_ECHO</bold>            {tab}Print the matched directory before navigating to it when set to 1
{tab}<bold>_ZO_EXCLUDE_DIRS</bold>    {tab}List of directory globs to be excluded
{tab}<bold>_ZO_FZF_OPTS</bold>        {tab}Custom flags to pass to fzf
{tab}<bold>_ZO_MAXAGE</bold>          {tab}Maximum total age after which entries start getting deleted
{tab}<bold>_ZO_RESOLVE_SYMLINKS</bold>{tab}Resolve symlinks when storing paths").into_resettable()
    }
}

#[derive(Debug, Parser)]
#[clap(
    about,
    author,
    help_template = HelpTemplate,
    disable_help_subcommand = true,
    propagate_version = true,
    version,
)]
pub enum Cmd {
    Add(Add),
    Edit(Edit),
    Import(Import),
    Init(Init),
    Query(Query),
    Remove(Remove),
}

/// Add a new directory or increment its rank
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Add {
    #[clap(num_args = 1.., required = true, value_hint = ValueHint::DirPath)]
    pub paths: Vec<PathBuf>,

    /// The rank to increment the entry if it exists or initialize it with if it
    /// doesn't
    #[clap(short, long)]
    pub score: Option<f64>,
}

/// Edit the database
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Edit {
    #[clap(subcommand)]
    pub cmd: Option<EditCommand>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum EditCommand {
    #[clap(hide = true)]
    Decrement { path: String },
    #[clap(hide = true)]
    Delete { path: String },
    #[clap(hide = true)]
    Increment { path: String },
    #[clap(hide = true)]
    Reload,
}

/// Import entries from another application
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Import {
    #[clap(subcommand)]
    pub from: ImportFrom,

    /// Merge into existing database
    #[clap(long, global = true)]
    pub merge: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ImportFrom {
    /// Import from atuin
    Atuin,
    /// Import from autojump
    Autojump,
    /// Import from fasd
    Fasd,
    /// Import from z
    Z,
    /// Import from z.lua
    #[clap(name = "z.lua")]
    ZLua,
    /// Import from zsh-z
    #[clap(name = "zsh-z")]
    ZshZ,
}

/// Generate shell configuration
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Init {
    #[clap(value_enum)]
    pub shell: InitShell,

    /// Prevents zoxide from defining the `z` and `zi` commands
    #[clap(long, alias = "no-aliases")]
    pub no_cmd: bool,

    /// Changes the prefix of the `z` and `zi` commands
    #[clap(long, default_value = "z")]
    pub cmd: String,

    /// Changes how often zoxide increments a directory's score
    #[clap(value_enum, long, default_value = "pwd")]
    pub hook: InitHook,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitHook {
    None,
    Prompt,
    Pwd,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum InitShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    #[clap(alias = "ksh")]
    Posix,
    Powershell,
    Tcsh,
    Xonsh,
    Zsh,
}

/// Search for a directory in the database
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Query {
    pub keywords: Vec<String>,

    /// Show unavailable directories
    #[clap(long, short)]
    pub all: bool,

    /// Use interactive selection
    #[clap(long, short, conflicts_with = "list")]
    pub interactive: bool,

    /// List all matching directories
    #[clap(long, short, conflicts_with = "interactive")]
    pub list: bool,

    /// Print score with results
    #[clap(long, short)]
    pub score: bool,

    /// Exclude the current directory
    #[clap(long, value_hint = ValueHint::DirPath, value_name = "path")]
    pub exclude: Option<String>,

    /// Only search within this directory
    #[clap(long, value_hint = ValueHint::DirPath, value_name = "path")]
    pub base_dir: Option<String>,
}

/// Remove a directory from the database
#[derive(Debug, Parser)]
#[clap(
    author,
    help_template = HelpTemplate,
)]
pub struct Remove {
    #[clap(value_hint = ValueHint::DirPath)]
    pub paths: Vec<String>,
}

#[cfg(feature = "nix-dev")]
#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::{env, fs};

    use clap::CommandFactory;
    use clap_complete::Generator;
    use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
    use clap_complete_fig::Fig;
    use clap_complete_nushell::Nushell;

    use super::*;

    #[test]
    fn completions() {
        check(Bash);
        check(Elvish);
        check(Fig);
        check(Fish);
        check(Nushell);
        check(PowerShell);
        check(Zsh);
    }

    /// Checks that the committed completions match the CLI, or regenerates
    /// them when `ZOXIDE_UPDATE_COMPLETIONS` is set (see `just completions`).
    fn check(generator: impl Generator) {
        const BIN_NAME: &str = env!("CARGO_PKG_NAME");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contrib/completions")
            .join(generator.file_name(BIN_NAME));

        let mut generated = Vec::new();
        clap_complete::generate(generator, &mut Cmd::command(), BIN_NAME, &mut generated);

        if env::var_os("ZOXIDE_UPDATE_COMPLETIONS").is_some() {
            fs::write(&path, generated).unwrap();
        } else {
            let committed = fs::read(&path).unwrap();
            assert!(
                committed == generated,
                "{} is out of date, run `just completions` to regenerate it",
                path.display()
            );
        }
    }
}
