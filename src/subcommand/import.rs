use crate::db::{Db, Dir};
use crate::util::{canonicalize, get_db, path_to_str};

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use std::fs;
use std::path::{Path, PathBuf};

/// Import from z database
#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Import {
    path: PathBuf,

    /// Merge entries into existing database
    #[structopt(long)]
    merge: bool,
}

impl Import {
    pub fn run(&self) -> Result<()> {
        import(&self.path, self.merge)
    }
}

fn import<P: AsRef<Path>>(path: P, merge: bool) -> Result<()> {
    let path = path.as_ref();
    let mut db = get_db()?;

    if !db.dirs.is_empty() && !merge {
        bail!(
            "To prevent conflicts, you can only import from z with an empty zoxide database!\n\
             If you wish to merge the two, specify the `--merge` flag."
        );
    }

    let buffer = fs::read_to_string(&path)
        .with_context(|| format!("could not read z database: {}", path.display()))?;

    for (idx, line) in buffer.lines().enumerate() {
        if let Err(e) = import_line(&mut db, line) {
            let line_num = idx + 1;
            eprintln!("Error on line {}: {}", line_num, e);
        }
    }

    db.modified = true;
    println!("Completed import.");

    Ok(())
}

fn import_line(db: &mut Db, line: &str) -> Result<()> {
    let mut split_line = line.rsplitn(3, '|');

    let (path, epoch_str, rank_str) = (|| {
        let epoch_str = split_line.next()?;
        let rank_str = split_line.next()?;
        let path = split_line.next()?;
        Some((path, epoch_str, rank_str))
    })()
    .with_context(|| format!("invalid entry: {}", line))?;

    let epoch = epoch_str
        .parse::<i64>()
        .with_context(|| format!("invalid epoch: {}", epoch_str))?;

    let rank = rank_str
        .parse::<f64>()
        .with_context(|| format!("invalid rank: {}", rank_str))?;

    let path = canonicalize(&path)?;
    let path = path_to_str(&path)?;

    // If the path exists in the database, add the ranks and set the epoch to
    // the more recent of the parsed epoch and the already present epoch.
    if let Some(dir) = db.dirs.iter_mut().find(|dir| dir.path == path) {
        dir.rank += rank;
        dir.last_accessed = epoch.max(dir.last_accessed);
    } else {
        db.dirs.push(Dir {
            path: path.to_string(),
            rank,
            last_accessed: epoch,
        });
    }

    Ok(())
}
