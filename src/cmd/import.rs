use std::fs;

use anyhow::{bail, Context, Result};

use crate::cmd::{Import, ImportFrom, Run};
use crate::config;
use crate::db::{Database, DatabaseFile, Dir};

impl Run for Import {
    fn run(&self) -> Result<()> {
        let buffer = fs::read_to_string(&self.path)
            .with_context(|| format!("could not open database for importing: {}", &self.path.display()))?;

        let data_dir = config::data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let db = &mut db.open()?;
        if !self.merge && !db.dirs.is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        match self.from {
            ImportFrom::Autojump => from_autojump(db, &buffer),
            ImportFrom::Z => from_z(db, &buffer),
        }
        .context("import error")?;

        db.save()
    }
}

fn from_autojump<'a>(db: &mut Database<'a>, buffer: &'a str) -> Result<()> {
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.splitn(2, '\t');

        let rank = split.next().with_context(|| format!("invalid entry: {line}"))?;
        let mut rank = rank.parse::<f64>().with_context(|| format!("invalid rank: {rank}"))?;
        // Normalize the rank using a sigmoid function. Don't import actual ranks from autojump,
        // since its scoring algorithm is very different and might take a while to get normalized.
        rank = sigmoid(rank);

        let path = split.next().with_context(|| format!("invalid entry: {line}"))?;

        db.dirs.push(Dir { path: path.into(), rank, last_accessed: 0 });
        db.modified = true;
    }

    if db.modified {
        db.dedup();
    }

    Ok(())
}

fn from_z<'a>(db: &mut Database<'a>, buffer: &'a str) -> Result<()> {
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.rsplitn(3, '|');

        let last_accessed = split.next().with_context(|| format!("invalid entry: {line}"))?;
        let last_accessed = last_accessed.parse().with_context(|| format!("invalid epoch: {last_accessed}"))?;

        let rank = split.next().with_context(|| format!("invalid entry: {line}"))?;
        let rank = rank.parse().with_context(|| format!("invalid rank: {rank}"))?;

        let path = split.next().with_context(|| format!("invalid entry: {line}"))?;

        db.dirs.push(Dir { path: path.into(), rank, last_accessed });
        db.modified = true;
    }

    if db.modified {
        db.dedup();
    }

    Ok(())
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::sigmoid;
    use crate::db::{Database, Dir};

    #[test]
    fn from_autojump() {
        let buffer = r#"
7.0	/baz
2.0	/foo/bar
5.0	/quux/quuz
"#;

        let dirs = vec![
            Dir { path: "/quux/quuz".into(), rank: 1.0, last_accessed: 100 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
            Dir { path: "/foo/bar".into(), rank: 9.0, last_accessed: 900 },
        ];
        let data_dir = tempfile::tempdir().unwrap();
        let data_dir = &data_dir.path().to_path_buf();
        let mut db = Database { dirs: dirs.into(), modified: false, data_dir };

        super::from_autojump(&mut db, buffer).unwrap();
        db.dirs.sort_by(|dir1, dir2| dir1.path.cmp(&dir2.path));
        println!("got: {:?}", &db.dirs.as_slice());

        let exp = &[
            Dir { path: "/baz".into(), rank: sigmoid(7.0), last_accessed: 0 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/foo/bar".into(), rank: 9.0 + sigmoid(2.0), last_accessed: 900 },
            Dir { path: "/quux/quuz".into(), rank: 1.0 + sigmoid(5.0), last_accessed: 100 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
        ];
        println!("exp: {exp:?}");

        for (dir1, dir2) in db.dirs.iter().zip(exp) {
            assert_eq!(dir1.path, dir2.path);
            assert!((dir1.rank - dir2.rank).abs() < 0.01);
            assert_eq!(dir1.last_accessed, dir2.last_accessed);
        }
    }

    #[test]
    fn from_z() {
        let buffer = r#"
/baz|7|700
/quux/quuz|4|400
/foo/bar|2|200
/quux/quuz|5|500
"#;

        let dirs = vec![
            Dir { path: "/quux/quuz".into(), rank: 1.0, last_accessed: 100 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
            Dir { path: "/foo/bar".into(), rank: 9.0, last_accessed: 900 },
        ];
        let data_dir = tempfile::tempdir().unwrap();
        let data_dir = &data_dir.path().to_path_buf();
        let mut db = Database { dirs: dirs.into(), modified: false, data_dir };

        super::from_z(&mut db, buffer).unwrap();
        db.dirs.sort_by(|dir1, dir2| dir1.path.cmp(&dir2.path));
        println!("got: {:?}", &db.dirs.as_slice());

        let exp = &[
            Dir { path: "/baz".into(), rank: 7.0, last_accessed: 700 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/foo/bar".into(), rank: 11.0, last_accessed: 900 },
            Dir { path: "/quux/quuz".into(), rank: 10.0, last_accessed: 500 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
        ];
        println!("exp: {exp:?}");

        for (dir1, dir2) in db.dirs.iter().zip(exp) {
            assert_eq!(dir1.path, dir2.path);
            assert!((dir1.rank - dir2.rank).abs() < 0.01);
            assert_eq!(dir1.last_accessed, dir2.last_accessed);
        }
    }
}
