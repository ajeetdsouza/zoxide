use std::fs;

use anyhow::{bail, Context, Result};

use crate::cmd::{Import, ImportFrom, Run};
use crate::db::Database;

impl Run for Import {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;
        if !self.merge && !db.dirs().is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        let buffer = fs::read(&self.path).with_context(|| {
            format!("could not open database for importing: {}", &self.path.display())
        })?;

        if matches!(self.from, ImportFrom::Zoxide) {
            from_self(&mut db, &buffer)?;
        } else {
            let buffer = std::str::from_utf8(&buffer).with_context(|| {
                format!("could not open database for importing: {}", &self.path.display())
            })?;
            match self.from {
                ImportFrom::Autojump => import_autojump(&mut db, buffer),
                ImportFrom::Z => import_z(&mut db, buffer),
                ImportFrom::Zoxide => unreachable!(),
            }
            .context("import error")?;
        }

        db.save()
    }
}

fn from_self(db: &mut Database, buffer: &[u8]) -> Result<()> {
    for dir in Database::deserialize(buffer).context("could not deserialize database")? {
        db.add_unchecked(dir.path, dir.rank, dir.last_accessed);
    }
    db.dedup();
    Ok(())
}

fn import_autojump(db: &mut Database, buffer: &str) -> Result<()> {
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let (rank, path) =
            line.split_once('\t').with_context(|| format!("invalid entry: {line}"))?;

        let mut rank = rank.parse::<f64>().with_context(|| format!("invalid rank: {rank}"))?;
        // Normalize the rank using a sigmoid function. Don't import actual ranks from
        // autojump, since its scoring algorithm is very different and might
        // take a while to get normalized.
        rank = sigmoid(rank);

        db.add_unchecked(path, rank, 0);
    }

    if db.dirty() {
        db.dedup();
    }
    Ok(())
}

fn import_z(db: &mut Database, buffer: &str) -> Result<()> {
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.rsplitn(3, '|');

        let last_accessed = split.next().with_context(|| format!("invalid entry: {line}"))?;
        let last_accessed =
            last_accessed.parse().with_context(|| format!("invalid epoch: {last_accessed}"))?;

        let rank = split.next().with_context(|| format!("invalid entry: {line}"))?;
        let rank = rank.parse().with_context(|| format!("invalid rank: {rank}"))?;

        let path = split.next().with_context(|| format!("invalid entry: {line}"))?;

        db.add_unchecked(path, rank, last_accessed);
    }

    if db.dirty() {
        db.dedup();
    }
    Ok(())
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{sigmoid, *};
    use crate::db::{Database, Dir};

    #[test]
    fn from_autojump() {
        let data_dir = tempfile::tempdir().unwrap();
        let mut db = Database::open_dir(data_dir.path()).unwrap();
        for (path, rank, last_accessed) in [
            ("/quux/quuz", 1.0, 100),
            ("/corge/grault/garply", 6.0, 600),
            ("/waldo/fred/plugh", 3.0, 300),
            ("/xyzzy/thud", 8.0, 800),
            ("/foo/bar", 9.0, 900),
        ] {
            db.add_unchecked(path, rank, last_accessed);
        }

        let buffer = "\
7.0	/baz
2.0	/foo/bar
5.0	/quux/quuz";
        import_autojump(&mut db, buffer).unwrap();

        db.sort_by_path();
        println!("got: {:?}", &db.dirs());

        let exp = [
            Dir { path: "/baz".into(), rank: sigmoid(7.0), last_accessed: 0 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/foo/bar".into(), rank: 9.0 + sigmoid(2.0), last_accessed: 900 },
            Dir { path: "/quux/quuz".into(), rank: 1.0 + sigmoid(5.0), last_accessed: 100 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
        ];
        println!("exp: {exp:?}");

        for (dir1, dir2) in db.dirs().iter().zip(exp) {
            assert_eq!(dir1.path, dir2.path);
            assert!((dir1.rank - dir2.rank).abs() < 0.01);
            assert_eq!(dir1.last_accessed, dir2.last_accessed);
        }
    }

    #[test]
    fn from_z() {
        let data_dir = tempfile::tempdir().unwrap();
        let mut db = Database::open_dir(data_dir.path()).unwrap();
        for (path, rank, last_accessed) in [
            ("/quux/quuz", 1.0, 100),
            ("/corge/grault/garply", 6.0, 600),
            ("/waldo/fred/plugh", 3.0, 300),
            ("/xyzzy/thud", 8.0, 800),
            ("/foo/bar", 9.0, 900),
        ] {
            db.add_unchecked(path, rank, last_accessed);
        }

        let buffer = "\
/baz|7|700
/quux/quuz|4|400
/foo/bar|2|200
/quux/quuz|5|500";
        import_z(&mut db, buffer).unwrap();

        db.sort_by_path();
        println!("got: {:?}", &db.dirs());

        let exp = [
            Dir { path: "/baz".into(), rank: 7.0, last_accessed: 700 },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600 },
            Dir { path: "/foo/bar".into(), rank: 11.0, last_accessed: 900 },
            Dir { path: "/quux/quuz".into(), rank: 10.0, last_accessed: 500 },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800 },
        ];
        println!("exp: {exp:?}");

        for (dir1, dir2) in db.dirs().iter().zip(exp) {
            assert_eq!(dir1.path, dir2.path);
            assert!((dir1.rank - dir2.rank).abs() < 0.01);
            assert_eq!(dir1.last_accessed, dir2.last_accessed);
        }
    }

    #[test]
    fn test_from_self() {
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let second_path = if cfg!(windows) { r"C:\bar\foo" } else { "/bar/foo" };
        let now = 946684800;
        let before = 946684700;

        let source_data_dir = tempfile::tempdir().unwrap();
        {
            let mut db = Database::open_dir(source_data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.save().unwrap();
            assert_eq!(db.dirs().len(), 1);
            let dir = &db.dirs()[0];
            assert_eq!(dir.path, path);
            assert_eq!(dir.last_accessed, now);
            assert_eq!(dir.rank, 1.0);
        }

        let dest_data_dir = tempfile::tempdir().unwrap();
        let mut db = Database::open_dir(dest_data_dir.path()).unwrap();
        db.add(path, 1.0, before);
        db.add(second_path, 1.0, before);
        db.save().unwrap();

        let source_buf = fs::read(source_data_dir.path().join("db.zo")).unwrap();

        super::from_self(&mut db, &source_buf).unwrap();

        assert_eq!(db.dirs().len(), 2);
        let dir = &db.dirs()[0];
        assert_eq!(dir.path, second_path);
        assert_eq!(dir.last_accessed, before);
        assert_eq!(dir.rank, 1.0);
        let dir2 = &db.dirs()[1];
        assert_eq!(dir2.path, path);
        assert_eq!(dir2.last_accessed, now);
        assert_eq!(dir2.rank, 2.0);
    }
}
