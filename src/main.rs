mod config;
mod db;
mod dir;
mod error;
mod types;
mod util;

use crate::db::DB;
use crate::error::AppError;
use crate::types::Timestamp;
use crate::util::{fzf_helper, get_current_time, get_db_path, process_query};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand};
use failure::ResultExt;
use std::env;
use std::path::Path;

fn zoxide_query(
    db: &mut DB,
    keywords: &[String],
    now: Timestamp,
) -> Result<Option<String>, failure::Error> {
    if let [path] = keywords {
        if Path::new(&path).is_dir() {
            return Ok(Some(path.to_owned()));
        }
    }

    if let Some(dir) = db.query(keywords, now) {
        return Ok(Some(dir.path));
    }

    Ok(None)
}

fn zoxide_query_interactive(
    db: &mut DB,
    keywords: &[String],
    now: Timestamp,
) -> Result<Option<String>, failure::Error> {
    db.remove_invalid();

    let dirs = db
        .dirs
        .iter()
        .filter(|dir| dir.is_match(keywords))
        .cloned()
        .collect();

    fzf_helper(now, dirs)
}

fn zoxide_app() -> App<'static, 'static> {
    app_from_crate!()
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a new directory or increment its rank")
                .author(crate_authors!())
                .version(crate_version!())
                .arg(Arg::with_name("PATH")),
        )
        .subcommand(
            SubCommand::with_name("query")
                .about("Search for a directory")
                .author(crate_authors!())
                .version(crate_version!())
                .arg(
                    Arg::with_name("interactive")
                        .short("i")
                        .long("interactive")
                        .takes_value(false)
                        .help("Opens an interactive selection menu using fzf"),
                )
                .arg(Arg::with_name("KEYWORD").min_values(0)),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Remove a directory")
                .author(crate_authors!())
                .version(crate_version!())
                .arg(Arg::with_name("PATH").required(true)),
        )
}

fn zoxide() -> Result<(), failure::Error> {
    let matches = zoxide_app().get_matches();

    let db_path = get_db_path()?;
    let mut db = DB::open(&db_path)?;

    if let Some(matches) = matches.subcommand_matches("query") {
        let now = get_current_time()?;
        let keywords = process_query(matches.values_of("KEYWORD").unwrap_or_default());

        let path_opt = if matches.is_present("interactive") {
            zoxide_query_interactive(&mut db, &keywords, now)
        } else {
            zoxide_query(&mut db, &keywords, now)
        }?;

        if let Some(path) = path_opt {
            print!("query: {}", path);
        }
    } else if let Some(matches) = matches.subcommand_matches("add") {
        let now = get_current_time()?;
        match matches.value_of_os("PATH") {
            Some(path) => db.add(path, now)?,
            None => {
                let path = env::current_dir().with_context(|_| AppError::GetCurrentDirError)?;
                db.add(path, now)?;
            }
        };
    } else if let Some(matches) = matches.subcommand_matches("remove") {
        // unwrap is safe here because PATH has been set as a required field
        let path = matches.value_of_os("PATH").unwrap();
        db.remove(path)?;
    }

    db.save(db_path)
}

fn main() {
    if let Err(err) = zoxide() {
        eprintln!("zoxide: {}", err);
        std::process::exit(1);
    }
}
