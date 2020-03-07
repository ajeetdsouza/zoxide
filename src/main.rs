mod config;
mod db;
mod dir;
mod error;
mod types;
mod util;

use crate::config::get_zo_data;
use crate::db::DB;
use crate::error::AppError;
use crate::types::Timestamp;
use crate::util::{fzf_helper, get_current_time};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand};
use failure::ResultExt;
use std::env;
use std::path::Path;

fn zoxide_query(db: &mut DB, mut keywords: Vec<String>, now: Timestamp) -> Option<String> {
    if let [path] = keywords.as_slice() {
        if Path::new(path).is_dir() {
            return Some(path.to_owned());
        }
    }

    for keyword in &mut keywords {
        keyword.make_ascii_lowercase();
    }

    if let Some(dir) = db.query(&keywords, now) {
        return Some(dir.path);
    }

    None
}

fn zoxide_query_interactive(
    db: &mut DB,
    mut keywords: Vec<String>,
    now: Timestamp,
) -> Result<Option<String>, failure::Error> {
    db.remove_invalid();

    for keyword in &mut keywords {
        keyword.make_ascii_lowercase();
    }

    let dirs = db
        .dirs
        .iter()
        .filter(|dir| dir.is_match(&keywords))
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

    let db_path = get_zo_data()?;
    let mut db = DB::open(&db_path)?;

    if let Some(matches) = matches.subcommand_matches("query") {
        let now = get_current_time()?;

        let keywords = matches
            .values_of_os("KEYWORD")
            .unwrap_or_default()
            .map(|keyword| match keyword.to_str() {
                Some(keyword) => Ok(keyword.to_owned()),
                None => Err(AppError::UnicodeError),
            })
            .collect::<Result<Vec<String>, _>>()?;

        let path_opt = if matches.is_present("interactive") {
            zoxide_query_interactive(&mut db, keywords, now)
        } else {
            Ok(zoxide_query(&mut db, keywords, now))
        }?;

        if let Some(path) = path_opt {
            println!("query: {}", path.trim());
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
