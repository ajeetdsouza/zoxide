use std::borrow::Cow;
use std::env;
use std::io;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use ratatui::text::Line;
use skim::tui::event::{Action, ActionCallback, Event};
use skim::{DisplayContext, Matches, Skim, SkimItem, SkimOptions};

use crate::cmd::{EditCommand, Query};
use crate::db::{Database, Dir, Epoch, Stream};
use crate::error::SilentExit;
use crate::{config, util};

pub fn options(overrides: &[&str]) -> Result<SkimOptions> {
    let skim_opts = env::var_os("SKIM_DEFAULT_OPTIONS")
        .map(|opts| {
            opts.into_string().map_err(|_| anyhow!("invalid unicode in SKIM_DEFAULT_OPTIONS"))
        })
        .transpose()?;
    let fzf_opts = config::fzf_opts()
        .map(|opts| opts.into_string().map_err(|_| anyhow!("invalid unicode in _ZO_FZF_OPTS")))
        .transpose()?;
    parse_options(overrides, skim_opts.as_deref(), fzf_opts.as_deref())
}

fn parse_options(
    overrides: &[&str],
    skim_opts: Option<&str>,
    fzf_opts: Option<&str>,
) -> Result<SkimOptions> {
    let mut args = vec!["zoxide".to_owned()];
    if let Some(opts) = skim_opts {
        args.extend(shlex::split(opts).context("could not parse SKIM_DEFAULT_OPTIONS")?);
    }
    args.extend(overrides.iter().map(ToString::to_string));
    if let Some(opts) = fzf_opts {
        args.extend(shlex::split(opts).context("could not parse _ZO_FZF_OPTS")?);
    }
    SkimOptions::try_parse_from(args).context("invalid skim options")
}

impl SkimItem for Dir<'static> {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.path)
    }

    fn display(&self, mut context: DisplayContext) -> Line<'_> {
        const SCORE_WIDTH: usize = 7;
        match &mut context.matches {
            Matches::CharIndices(indices) => {
                indices.iter_mut().for_each(|index| *index += SCORE_WIDTH)
            }
            Matches::CharRange(start, end) | Matches::ByteRange(start, end) => {
                *start += SCORE_WIDTH;
                *end += SCORE_WIDTH;
            }
            Matches::None => {}
        }
        let now = util::current_time().unwrap_or(self.last_accessed);
        context.to_line(Cow::Owned(
            Dir::display(self).with_score(now).with_separator('\t').to_string(),
        ))
    }
}

pub fn query(query: &Query, stream: &mut Stream, now: Epoch) -> Result<()> {
    let mut dirs = Vec::new();
    while let Some(dir) = stream.next() {
        if Some(dir.path.as_ref()) != query.exclude.as_deref() {
            dirs.push(Dir {
                path: Cow::Owned(dir.path.clone().into_owned()),
                rank: dir.rank,
                last_accessed: dir.last_accessed,
            });
        }
    }
    let mut args = vec![
        "--exact",
        "--no-sort",
        "--bind=ctrl-z:ignore,btab:up,tab:down",
        "--cycle",
        "--keep-right",
        "--border=sharp",
        "--height=45%",
        "--info=inline",
        "--layout=reverse",
        "--tabstop=1",
        "--exit-0",
    ];
    if cfg!(unix) {
        args.extend([
            if cfg!(target_os = "linux") {
                r"--preview=\command -p ls -Cp --color=always --group-directories-first {}"
            } else {
                r"--preview=\command -p ls -Cp {}"
            },
            "--preview-window=down,30%,sharp",
        ]);
    }
    let (tx, rx) = skim::prelude::unbounded();
    tx.send(dirs.into_iter().map(|dir| Arc::new(dir) as _).collect())?;
    drop(tx);
    let output =
        Skim::run_with(options(&args)?, Some(rx)).map_err(|error| anyhow!(error.to_string()))?;
    if output.is_abort {
        bail!(SilentExit { code: 130 });
    }
    let dir = output
        .selected_items
        .first()
        .and_then(|item| item.as_any().downcast_ref::<Dir<'static>>())
        .context("no match found")?;
    if query.score {
        print!("{}", dir.display().with_score(now));
    } else {
        print!("{}", dir.path);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Edit {
    Decrement,
    Delete,
    Increment,
    Reload,
}

impl Edit {
    fn command(self, path: String) -> Option<EditCommand> {
        match self {
            Self::Decrement => Some(EditCommand::Decrement { path }),
            Self::Delete => Some(EditCommand::Delete { path }),
            Self::Increment => Some(EditCommand::Increment { path }),
            Self::Reload => None,
        }
    }
}

fn apply_edit(db: &mut Database, edit: &EditCommand, now: Epoch) {
    match edit {
        EditCommand::Decrement { path } => db.add(path, -1.0, now),
        EditCommand::Delete { path } => {
            db.remove(path);
        }
        EditCommand::Increment { path } => db.add(path, 1.0, now),
        EditCommand::Reload => {}
    }
}

fn owned_dirs(db: &Database) -> Vec<Dir<'static>> {
    db.dirs()
        .iter()
        .rev()
        .map(|dir| Dir {
            path: Cow::Owned(dir.path.clone().into_owned()),
            rank: dir.rank,
            last_accessed: dir.last_accessed,
        })
        .collect()
}

fn edit_action(dirs: Arc<Mutex<Vec<Dir<'static>>>>, edit: Edit) -> Action {
    Action::Custom(ActionCallback::new_sync(move |app| {
        let path = app.item_list.selected().and_then(|item| {
            item.as_any().downcast_ref::<Dir<'static>>().map(|dir| dir.path.clone().into_owned())
        });
        let command = path.and_then(|path| edit.command(path));
        if command.is_none() && !matches!(edit, Edit::Reload) {
            return Ok(Vec::new());
        }

        let mut db = Database::open().map_err(|error| io::Error::other(error.to_string()))?;
        if let Some(command) = command {
            apply_edit(
                &mut db,
                &command,
                util::current_time().map_err(|error| io::Error::other(error.to_string()))?,
            );
            db.save().map_err(|error| io::Error::other(error.to_string()))?;
        }
        let items = owned_dirs(&db);
        *dirs.lock().map_err(|_| io::Error::other("skim directory lock poisoned"))? = items.clone();
        Ok(vec![
            Event::ClearItems,
            Event::AppendItems(
                items.into_iter().map(|dir| Arc::new(dir) as Arc<dyn SkimItem>).collect(),
            ),
        ])
    }))
}

pub fn edit(db: &mut Database) -> Result<()> {
    let dirs = Arc::new(Mutex::new(owned_dirs(db)));
    let mut args = vec![
        "--exact",
        "--no-sort",
        "--bind=btab:up,ctrl-z:ignore,double-click:ignore,enter:abort,tab:down",
        "--cycle",
        "--keep-right",
        "--border=sharp",
        "--header=ctrl-r:reload     ctrl-d:delete\nctrl-w:increment  ctrl-s:decrement\n\n SCORE\tPATH",
        "--info=inline",
        "--layout=reverse",
        "--tabstop=1",
    ];
    if cfg!(unix) {
        args.extend([
            if cfg!(target_os = "linux") {
                r"--preview=\command -p ls -Cp --color=always --group-directories-first {}"
            } else {
                r"--preview=\command -p ls -Cp {}"
            },
            "--preview-window=down:30%",
        ]);
    }
    let mut options = options(&args)?;
    for (key, edit) in [
        ("ctrl-r", Edit::Reload),
        ("ctrl-d", Edit::Delete),
        ("ctrl-w", Edit::Increment),
        ("ctrl-s", Edit::Decrement),
    ] {
        options.keymap.insert(
            skim::binds::parse_key(key).map_err(|error| anyhow!(error))?,
            vec![edit_action(dirs.clone(), edit)],
        );
    }

    let items = dirs.lock().map_err(|_| anyhow!("skim directory lock poisoned"))?.clone();
    let output = Skim::run_items(options, items).map_err(|error| anyhow!(error.to_string()))?;
    if output.is_abort {
        bail!(SilentExit { code: 130 });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use skim::{DisplayContext, SkimItem};

    use super::{Edit, parse_options};
    use crate::db::Dir;
    use crate::{cmd::EditCommand, util};

    #[test]
    fn skim_options_and_edits() -> anyhow::Result<()> {
        let options = parse_options(
            &["--prompt=default"],
            Some("--prompt='skim: '"),
            Some("--prompt='zoxide: '"),
        )?;
        assert_eq!(options.prompt, "zoxide: ");
        assert!(matches!(
            Edit::Increment.command("/tmp".to_owned()),
            Some(EditCommand::Increment { path }) if path == "/tmp"
        ));
        assert!(Edit::Reload.command(String::new()).is_none());

        let dir =
            Dir { path: "/tmp".to_owned().into(), rank: 2.0, last_accessed: util::current_time()? };
        assert_eq!(SkimItem::display(&dir, DisplayContext::default()).to_string(), "   8.0\t/tmp");
        Ok(())
    }
}
