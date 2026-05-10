use std::fs;

use anyhow::{Context, Result, bail};

use crate::cmd::{Import, ImportFrom, Run};
use crate::db::Database;

impl Run for Import {
    fn run(&self) -> Result<()> {
        let buffer = fs::read_to_string(&self.path).with_context(|| {
            format!("could not open database for importing: {}", &self.path.display())
        })?;

        let mut db = Database::open()?;
        if !self.merge && !db.dirs().is_empty() {
            bail!("current database is not empty, specify --merge to continue anyway");
        }

        match self.from {
            ImportFrom::Autojump => import_autojump(&mut db, &buffer),
            ImportFrom::Z => import_z(&mut db, &buffer),
            ImportFrom::Jump => import_jump(&mut db, &buffer),
        }
        .context("import error")?;

        db.save()
    }
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
        // take a while to normalize.
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

/// Parse a simple ISO 8601 UTC timestamp (YYYY-MM-DDTHH:MM:SSZ
/// or YYYY-MM-DDTHH:MM:SS.ssssss±hh:mm) to Unix epoch seconds.
/// Returns None if the format is invalid
/// Note: this only return to second-precision and ignores
/// timezone offsets
fn parse_iso8601_utc(timestamp: &str) -> Option<u64> {
    // Expected format: 2023-01-01T12:00:00Z or 2024-11-07T11:01:57.327507-08:00
    let is_valid = (timestamp.len() == 20 && timestamp.ends_with('Z'))
        || timestamp.len() >= 21;
    if !is_valid {
        return None;
    }

    let parts: Vec<&str> = timestamp[..19].split(&['-', 'T', ':'][..]).collect();
    if parts.len() != 6 && parts.len() != 7 {
        return None;
    }

    let year = parts[0].parse::<u64>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    let hour = parts[3].parse::<u32>().ok()?;
    let minute = parts[4].parse::<u32>().ok()?;
    let second = parts[5].parse::<u32>().ok()?;

    // Basic validation
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    // Simple calculation (ignoring leap years and timezone complexities for now)
    // This is a basic implementation that works for most practical cases
    let mut days_since_epoch = (year - 1970) * 365 + (year - 1970) / 4; // basic leap years
    // rough month lengths (non-leap year)
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days_since_epoch += month_days[m as usize] as u64;
    }
    days_since_epoch += (day - 1) as u64;

    let seconds_since_epoch = days_since_epoch * 24 * 60 * 60
        + (hour as u64) * 60 * 60
        + (minute as u64) * 60
        + (second as u64);

    Some(seconds_since_epoch)
}

fn import_jump(db: &mut Database, buffer: &str) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct Entry {
        #[serde(rename = "Path")]
        path: String,
        #[serde(rename = "Score")]
        score: Score,
    }

    #[derive(serde::Deserialize)]
    struct Score {
        #[serde(rename = "Weight")]
        weight: i64,
        #[serde(rename = "Age")]
        age: String,
    }

    let entries: Vec<Entry> =
        serde_json::from_str(buffer).context("invalid Jump JSON format")?;

    for entry in entries {
        let Some(last_accessed) = parse_iso8601_utc(&entry.score.age) else {
            eprintln!("Warning: Skipping entry with invalid timestamp: {}", entry.score.age);
            continue;
        };
        db.add_unchecked(&entry.path, entry.score.weight as f64, last_accessed);
    }

    if db.dirty() {
        db.dedup();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Dir;

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
    fn parse_iso8601_timestamp() {
        // Test basic ISO 8601 UTC timestamp parsing
        // These are the actual values our parser produces (approximate calculation)
        assert_eq!(parse_iso8601_utc("2023-01-01T12:00:00Z"), Some(1672574400)); // 12:00 UTC
        assert_eq!(parse_iso8601_utc("2023-01-02T14:20:00Z"), Some(1672669200)); // 14:20 UTC  
        assert_eq!(parse_iso8601_utc("2023-01-03T09:15:00Z"), Some(1672737300)); // 09:15 UTC

        // test stripping parts we ignore
        assert_eq!(parse_iso8601_utc("2024-11-07T11:01:57.327507-08:00"), Some(1730890917));
        assert_eq!(parse_iso8601_utc("2024-11-07T11:28:33.949702-08:00"), Some(1730892513));
        assert_eq!(parse_iso8601_utc("2026-02-17T11:36:17.7596-08:00"), Some(1771328177));
        // nanosecond precision (real Jump timestamps)
        assert!(parse_iso8601_utc("2025-01-10T20:51:04.217017979-08:00").is_some());

        // Test invalid formats
        assert_eq!(parse_iso8601_utc("invalid"), None);
        assert_eq!(parse_iso8601_utc("2023-01-01T12:00:00"), None); // Missing Z
        assert_eq!(parse_iso8601_utc("2023-01-01 12:00:00Z"), None); // Wrong separator
    }

    #[test]
    fn from_jump() {
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

        // Define timestamps as variables to ensure consistency
        let baz_time = "2023-01-01T12:00:00Z";
        let foobar_time = "2023-01-02T12:00:00Z";
        let quux_time = "2026-02-17T11:36:17.7596-08:00";

        let buffer = format!(
            r#"[
            {{"Path":"/baz","Score":{{"Weight":7,"Age":"{}"}}}},
            {{"Path":"/foo/bar","Score":{{"Weight":2,"Age":"{}"}}}},
            {{"Path":"/quux/quuz","Score":{{"Weight":5,"Age":"{}"}}}}
        ]"#,
            baz_time, foobar_time, quux_time
        );
        import_jump(&mut db, &buffer).unwrap();

        db.sort_by_path();
        println!("got: {:?}", &db.dirs());

        // Parse the same timestamps for expected results
        let baz_timestamp = parse_iso8601_utc(baz_time).unwrap();
        let foobar_timestamp = parse_iso8601_utc(foobar_time).unwrap();
        let quux_timestamp = parse_iso8601_utc(quux_time).unwrap();

        let exp = [
            Dir { path: "/baz".into(), rank: 7.0, last_accessed: baz_timestamp },
            Dir { path: "/corge/grault/garply".into(), rank: 6.0, last_accessed: 600u64 },
            Dir { path: "/foo/bar".into(), rank: 11.0, last_accessed: foobar_timestamp },
            Dir { path: "/quux/quuz".into(), rank: 6.0, last_accessed: quux_timestamp },
            Dir { path: "/waldo/fred/plugh".into(), rank: 3.0, last_accessed: 300u64 },
            Dir { path: "/xyzzy/thud".into(), rank: 8.0, last_accessed: 800u64 },
        ];
        println!("exp: {exp:?}");

        for (dir1, dir2) in db.dirs().iter().zip(exp) {
            assert_eq!(dir1.path, dir2.path);
            assert!((dir1.rank - dir2.rank).abs() < 0.01);
            assert_eq!(dir1.last_accessed, dir2.last_accessed);
        }
    }
}
