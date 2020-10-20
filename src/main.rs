use anyhow::{Context, Result};
use clap::{AppSettings, ArgEnum, Clap};
use once_cell::sync::OnceCell;
use zoxide::{config, util};
use zoxide_engine::{Dir, Query, Store};
use zoxide_shell::{self as zs, Generator};

use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn env_help() -> &'static str {
    static ENV_HELP: OnceCell<String> = OnceCell::new();
    ENV_HELP.get_or_init(|| {
        const PATH_SPLIT_SEPARATOR: u8 = if cfg!(any(target_os = "redox", target_os = "windows")) {
            b';'
        } else {
            b':'
        };

        format!(
            "\
ENVIRONMENT VARIABLES:
    _ZO_DATA_DIR            Path for zoxide data files (current: `{data_dir}`)
    _ZO_ECHO                Prints the matched directory before navigating to it when set to 1
    _ZO_EXCLUDE_DIRS        List of directories to be excluded, separated by `{split_paths_separator}`
    _ZO_FZF_OPTS            Custom flags to pass to fzf
    _ZO_MAXAGE              Maximum total age after which entries start getting deleted
    _ZO_RESOLVE_SYMLINKS    Resolve symlinks when storing paths",
            data_dir=config::zo_data_dir().unwrap_or_else(|_| "none".into()).display(),
            split_paths_separator=PATH_SPLIT_SEPARATOR as char)
    })
}

// TODO: import
// TODO: query interactive
#[derive(Debug, Clap)]
#[clap(
    about,
    author,
    global_setting(AppSettings::ColoredHelp),
    global_setting(AppSettings::GlobalVersion),
    global_setting(AppSettings::VersionlessSubcommands),
    version = env!("ZOXIDE_VERSION"))]
enum Opts {
    /// Adds a new directory or increments its rank
    Add { path: Option<PathBuf> },

    /// Generates shell configuration
    #[clap(after_help(env_help()))]
    Init {
        #[clap(arg_enum)]
        shell: Shell,

        /// Prevents zoxide from defining any commands
        #[clap(long)]
        no_aliases: bool,

        /// Renames the 'z' command and corresponding aliases
        #[clap(long, default_value = "z")]
        cmd: String,

        /// Chooses event upon which an entry is added to the database
        #[clap(arg_enum, long, default_value = "pwd")]
        hook: Hook,
    },

    /// Searches for a directory
    Query {
        keywords: Vec<String>,

        /// Lists all matching directories
        #[clap(long, short)]
        list: bool,

        /// Prints score with results
        #[clap(long, short)]
        score: bool,
    },

    /// Removes a directory
    Remove { path: String },
}

#[derive(ArgEnum, Debug)]
enum Shell {
    Bash,
    Fish,
    Posix,
    Powershell,
    Xonsh,
    Zsh,
}

#[derive(ArgEnum, Debug)]
enum Hook {
    None,
    Prompt,
    Pwd,
}

pub fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts {
        Opts::Add { path } => {
            let path = match path {
                Some(path) => {
                    if config::zo_resolve_symlinks() {
                        util::canonicalize(&path)
                    } else {
                        util::resolve_path(&path)
                    }
                }
                None => util::current_dir(),
            }?;

            if config::zo_exclude_dirs()?
                .iter()
                .any(|pattern| pattern.matches_path(&path))
            {
                return Ok(());
            }

            let path = util::path_to_str(&path)?;
            let now = util::current_time()?;

            let data_dir = config::zo_data_dir()?;
            let max_age = config::zo_maxage()?;

            let mut store = Store::open(&data_dir)?;
            store.add(path, now);
            store.age(max_age);

            Ok(())
        }

        Opts::Init {
            shell,
            no_aliases,
            cmd,
            hook,
        } => {
            let cmd = if no_aliases { None } else { Some(cmd.as_str()) };

            let hook = match hook {
                Hook::None => zs::Hook::None,
                Hook::Prompt => zs::Hook::Prompt,
                Hook::Pwd => zs::Hook::Pwd,
            };

            let echo = config::zo_echo();
            let resolve_symlinks = config::zo_resolve_symlinks();

            let opts = &zs::Opts {
                cmd,
                hook,
                echo,
                resolve_symlinks,
            };

            let stdout = io::stdout();
            let handle = &mut stdout.lock();

            match shell {
                Shell::Bash => zs::Bash(opts).generate(handle),
                Shell::Fish => zs::Fish(opts).generate(handle),
                Shell::Posix => zs::Posix(opts).generate(handle),
                Shell::Powershell => zs::PowerShell(opts).generate(handle),
                Shell::Xonsh => zs::Xonsh(opts).generate(handle),
                Shell::Zsh => zs::Zsh(opts).generate(handle),
            }?;

            Ok(())
        }

        Opts::Query {
            keywords,
            list,
            score,
        } => {
            let data_dir = config::zo_data_dir()?;
            let mut store = Store::open(&data_dir)?;

            let query = Query::new(&keywords);
            let now = util::current_time()?;

            let stdout = io::stdout();
            let mut handle = stdout.lock();

            let mut print_dir = |dir: &Dir| {
                if score {
                    let dir_score = dir.get_score(now);
                    let dir_score_clamped = if dir_score > 9999.0 {
                        9999
                    } else if dir_score > 0.0 {
                        dir_score as _
                    } else {
                        0
                    };
                    writeln!(&mut handle, "{:>4} {}", dir_score_clamped, dir.path)
                } else {
                    writeln!(&mut handle, "{}", dir.path)
                }
                .unwrap()
            };

            let mut matches = store
                .iter_matches(&query, now)
                .filter(|dir| Path::new(&dir.path).is_dir());

            if list {
                for dir in matches {
                    print_dir(dir);
                }
            } else {
                let dir = matches.next().context("no match found")?;
                print_dir(dir);
            }

            Ok(())
        }

        Opts::Remove { path } => {
            let data_dir = config::zo_data_dir()?;

            let mut store = Store::open(&data_dir)?;
            store.remove(path);

            Ok(())
        }
    }
}
