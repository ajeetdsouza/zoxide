mod dir;
mod stream;

use std::borrow::Cow;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, Result, bail};
use bincode::Options;
use ouroboros::self_referencing;

pub use crate::db::dir::{Dir, Epoch, Rank};
pub use crate::db::stream::{Stream, StreamOptions};
use crate::{config, util};

/// Width of the timestamp and rank columns.
const COL_WIDTH: usize = 10;
/// The smallest rank the database file can hold.
const MIN_RANK: Rank = 0.01;
/// The largest rank the database file can hold.
const MAX_RANK: Rank = 9_999_999.99;
/// The byte order mark that some editors write at the start of UTF-8 files.
const UTF8_BOM: &[u8] = "\u{FEFF}".as_bytes();

#[self_referencing]
pub struct Database {
    path: PathBuf,
    bytes: Vec<u8>,
    #[borrows(bytes)]
    #[covariant]
    pub dirs: Vec<Dir<'this>>,
    dirty: bool,
}

impl Database {
    pub fn open() -> Result<Self> {
        let data_dir = config::data_dir()?;
        Self::open_dir(data_dir)
    }

    pub fn open_dir(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let path = data_dir.join("db.txt");
        let path = fs::canonicalize(&path).unwrap_or(path);

        match fs::read(&path) {
            Ok(bytes) => {
                return Self::try_new(path.clone(), bytes, |bytes| Self::deserialize(bytes), false)
                    .with_context(|| format!("could not parse database: {}", path.display()));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("could not read from database: {}", path.display()));
            }
        }

        // Migrate the legacy bincode database, if there is one.
        let path_legacy = data_dir.join("db.zo");
        match fs::read(&path_legacy) {
            Ok(bytes) => {
                return Self::try_new(path, bytes, |bytes| Self::deserialize_legacy(bytes), true)
                    .with_context(|| {
                        format!("could not parse database: {}", path_legacy.display())
                    });
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("could not read from database: {}", path_legacy.display())
                });
            }
        }

        // Create the data directory, but don't create any file yet.
        fs::create_dir_all(data_dir)
            .with_context(|| format!("unable to create data directory: {}", data_dir.display()))?;
        Ok(Self::new(path, Vec::new(), |_| Vec::new(), false))
    }

    pub fn save(&mut self) -> Result<()> {
        // Only write to disk if the database is modified.
        if !self.dirty() {
            return Ok(());
        }

        let bytes = Self::serialize(self.dirs())?;
        util::write(self.borrow_path(), bytes).context("could not write to database")?;
        self.with_dirty_mut(|dirty| *dirty = false);

        Ok(())
    }

    /// Increments the rank of a directory, or creates it if it does not exist.
    pub fn add(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => dir.rank = (dir.rank + by).max(0.0),
            None => {
                dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now })
            }
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Creates a new directory. This will create a duplicate entry if this
    /// directory is already in the database, it is expected that the user
    /// either does a check before calling this, or calls `dedup()`
    /// afterward.
    pub fn add_unchecked(&mut self, path: impl AsRef<str> + Into<String>, rank: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| {
            dirs.push(Dir { path: path.into().into(), rank, last_accessed: now })
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Increments the rank and updates the last_accessed of a directory, or
    /// creates it if it does not exist.
    pub fn add_update(&mut self, path: impl AsRef<str> + Into<String>, by: Rank, now: Epoch) {
        self.with_dirs_mut(|dirs| match dirs.iter_mut().find(|dir| dir.path == path.as_ref()) {
            Some(dir) => {
                dir.rank = (dir.rank + by).max(0.0);
                dir.last_accessed = now;
            }
            None => {
                dirs.push(Dir { path: path.into().into(), rank: by.max(0.0), last_accessed: now })
            }
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    /// Removes the directory with `path` from the store. This does not preserve
    /// ordering.
    pub fn remove(&mut self, path: impl AsRef<str>) -> bool {
        match self.dirs().iter().position(|dir| dir.path == path.as_ref()) {
            Some(idx) => {
                self.swap_remove(idx);
                true
            }
            None => false,
        }
    }

    pub fn swap_remove(&mut self, idx: usize) {
        self.with_dirs_mut(|dirs| dirs.swap_remove(idx));
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn age(&mut self, max_age: Rank) {
        let mut dirty = false;
        self.with_dirs_mut(|dirs| {
            let total_age = dirs.iter().map(|dir| dir.rank).sum::<Rank>();
            if total_age > max_age {
                let factor = 0.9 * max_age / total_age;
                for idx in (0..dirs.len()).rev() {
                    let dir = &mut dirs[idx];
                    dir.rank *= factor;
                    if dir.rank < 1.0 {
                        dirs.swap_remove(idx);
                    }
                }
                dirty = true;
            }
        });
        self.with_dirty_mut(|dirty_prev| *dirty_prev |= dirty);
    }

    pub fn dedup(&mut self) {
        // Sort by path, so that equal paths are next to each other.
        self.sort_by_path();

        let mut dirty = false;
        self.with_dirs_mut(|dirs| {
            for idx in (1..dirs.len()).rev() {
                // Check if curr_dir and next_dir have equal paths.
                let curr_dir = &dirs[idx];
                let next_dir = &dirs[idx - 1];
                if next_dir.path != curr_dir.path {
                    continue;
                }

                // Merge curr_dir's rank and last_accessed into next_dir.
                let rank = curr_dir.rank;
                let last_accessed = curr_dir.last_accessed;
                let next_dir = &mut dirs[idx - 1];
                next_dir.last_accessed = next_dir.last_accessed.max(last_accessed);
                next_dir.rank += rank;

                // Delete curr_dir.
                dirs.swap_remove(idx);
                dirty = true;
            }
        });
        self.with_dirty_mut(|dirty_prev| *dirty_prev |= dirty);
    }

    pub fn sort_by_path(&mut self) {
        self.with_dirs_mut(|dirs| dirs.sort_unstable_by(|dir1, dir2| dir1.path.cmp(&dir2.path)));
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn sort_by_score(&mut self, now: Epoch) {
        self.with_dirs_mut(|dirs| {
            dirs.sort_unstable_by(|dir1: &Dir, dir2: &Dir| {
                dir1.score(now).total_cmp(&dir2.score(now))
            })
        });
        self.with_dirty_mut(|dirty| *dirty = true);
    }

    pub fn dirty(&self) -> bool {
        *self.borrow_dirty()
    }

    pub fn dirs(&self) -> &[Dir<'_>] {
        self.borrow_dirs()
    }

    fn serialize(dirs: &[Dir<'_>]) -> Result<Vec<u8>> {
        // timestamp + tab + rank + tab + path + newline.
        let size_hint: usize =
            dirs.iter().map(|dir| COL_WIDTH + 1 + COL_WIDTH + 1 + dir.path.len() + 1).sum();
        let mut buffer = Vec::with_capacity(size_hint);

        for dir in dirs {
            let rank = dir.rank.clamp(MIN_RANK, MAX_RANK);
            write_padded_u64(&mut buffer, dir.last_accessed);
            buffer.push(b'\t');
            write_padded_rank(&mut buffer, rank).context("could not serialize database")?;
            buffer.push(b'\t');
            buffer.extend_from_slice(dir.path.as_bytes());
            buffer.push(b'\n');
        }
        Ok(buffer)
    }

    fn deserialize(bytes: &[u8]) -> Result<Vec<Dir<'_>>> {
        let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);

        let mut dirs = Vec::with_capacity(bytes.len() / 48);
        let mut errors = Vec::new();

        if let Ok(text) = str::from_utf8(bytes) {
            for (idx, line) in text.split('\n').enumerate() {
                match Self::deserialize_str_line(line) {
                    Ok(Some(dir)) => dirs.push(dir),
                    Ok(None) => {}
                    Err(e) => errors.push(format!("line {}: {e:#}", idx + 1)),
                }
            }
        } else {
            for (idx, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
                match Self::deserialize_line(line) {
                    Ok(Some(dir)) => dirs.push(dir),
                    Ok(None) => {}
                    Err(e) => errors.push(format!("line {}: {e:#}", idx + 1)),
                }
            }
        }

        if !errors.is_empty() {
            const MAX_ERRORS: usize = 8;
            let total = errors.len();
            if total > MAX_ERRORS {
                errors.truncate(MAX_ERRORS);
                errors.push(format!("... and {} more", total - MAX_ERRORS));
            }
            bail!("{}", errors.join("\n"));
        }
        Ok(dirs)
    }

    fn deserialize_line(line: &[u8]) -> Result<Option<Dir<'_>>> {
        let line = str::from_utf8(line).context("invalid UTF-8")?;
        Self::deserialize_str_line(line)
    }

    fn deserialize_str_line(line: &str) -> Result<Option<Dir<'_>>> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty()
            || (line.as_bytes().iter().all(u8::is_ascii_whitespace))
            || line.trim().is_empty()
        {
            return Ok(None);
        }

        const EXPECTED: &str = "expected: <timestamp> <rank> <path>";
        let (last_accessed, rank, path) = if let Some((ts, rest)) = line.split_once('\t')
            && !ts.is_empty()
            && !ts.as_bytes()[0].is_ascii_whitespace()
            && let Some((rk, p)) = rest.split_once('\t')
            && !rk.is_empty()
            && !rk.as_bytes()[0].is_ascii_whitespace()
            && !p.is_empty()
            && !p.as_bytes()[0].is_ascii_whitespace()
            && !p.chars().next().unwrap().is_whitespace()
        {
            (ts, rk, p)
        } else {
            let (last_accessed, rest) =
                line.trim_start().split_once(char::is_whitespace).context(EXPECTED)?;
            let (rank, path) =
                rest.trim_start().split_once(char::is_whitespace).context(EXPECTED)?;
            (last_accessed, rank, path.trim_start())
        };

        let last_accessed = parse_epoch_fast(last_accessed).unwrap_or_else(|| {
            last_accessed
                .parse::<Epoch>()
                .with_context(|| format!("could not parse timestamp: {last_accessed}"))
        })?;
        let rank = rank
            .parse::<Rank>()
            .with_context(|| format!("could not parse rank: {rank}"))?
            .clamp(MIN_RANK, MAX_RANK);
        if rank.is_nan() {
            bail!("could not parse rank: {rank}");
        }
        if path.is_empty() {
            bail!("{EXPECTED}");
        }
        Ok(Some(Dir { path: Cow::Borrowed(path), rank, last_accessed }))
    }

    fn deserialize_legacy(bytes: &[u8]) -> Result<Vec<Dir<'_>>> {
        // Assume a maximum size for the database. This prevents bincode from throwing
        // strange errors when it encounters invalid data.
        const MAX_SIZE: u64 = 32 << 20; // 32 MiB
        const VERSION: u32 = 3;
        let (bytes_version, bytes_dirs) =
            bytes.split_at_checked(size_of::<u32>()).context("corrupted data")?;

        let deserializer = &mut bincode::options().with_fixint_encoding().with_limit(MAX_SIZE);
        let version: u32 = deserializer.deserialize(bytes_version)?;
        if version != VERSION {
            bail!("corrupted data");
        }
        deserializer.deserialize(bytes_dirs).context("corrupted data")
    }
}

const DIGITS2: &[u8; 200] = b"\
0001020304050607080910111213141516171819\
2021222324252627282930313233343536373839\
4041424344454647484950515253545556575859\
6061626364656667686970717273747576777879\
8081828384858687888990919293949596979899";

#[inline]
fn write_padded_u64(buf: &mut Vec<u8>, mut val: u64) {
    if val < 10_000_000_000 {
        let mut out = [b'0'; COL_WIDTH];
        let mut i = COL_WIDTH;
        while i >= 2 {
            let rem = (val % 100) as usize;
            val /= 100;
            i -= 2;
            out[i] = DIGITS2[rem * 2];
            out[i + 1] = DIGITS2[rem * 2 + 1];
        }
        buf.extend_from_slice(&out);
    } else {
        let _ = write!(buf, "{val:0COL_WIDTH$}");
    }
}

#[inline]
fn write_padded_rank(buf: &mut Vec<u8>, rank: Rank) -> io::Result<()> {
    let scaled = rank * 100.0;
    let rounded = scaled.round_ties_even();
    // Fall back to core::fmt only on hairline half-way values where multiplying
    // by 100 could suffer double rounding, or non-finite values.
    if rounded.is_finite()
        && (0.0..=999_999_999.0).contains(&rounded)
        && ((scaled - scaled.floor()) - 0.5).abs() > 1e-9
    {
        let mut cents = rounded as u64;
        let frac = (cents % 100) as usize;
        cents /= 100;
        let mut out = [b'0'; COL_WIDTH];
        out[9] = DIGITS2[frac * 2 + 1];
        out[8] = DIGITS2[frac * 2];
        out[7] = b'.';
        let d2 = (cents % 100) as usize;
        cents /= 100;
        out[6] = DIGITS2[d2 * 2 + 1];
        out[5] = DIGITS2[d2 * 2];
        let d1 = (cents % 100) as usize;
        cents /= 100;
        out[4] = DIGITS2[d1 * 2 + 1];
        out[3] = DIGITS2[d1 * 2];
        let d0 = (cents % 100) as usize;
        cents /= 100;
        out[2] = DIGITS2[d0 * 2 + 1];
        out[1] = DIGITS2[d0 * 2];
        out[0] = b'0' + (cents as u8);
        buf.extend_from_slice(&out);
        Ok(())
    } else {
        write!(buf, "{rank:0COL_WIDTH$.2}")
    }
}

#[inline]
fn parse_epoch_fast(s: &str) -> Option<Result<Epoch>> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > 19 {
        return None;
    }
    let mut acc: u64 = 0;
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        acc = acc * 10 + u64::from(d);
    }
    Some(Ok(acc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let db = Database::open_dir(data_dir.path()).unwrap();
            assert_eq!(db.dirs().len(), 1);

            let dir = &db.dirs()[0];
            assert_eq!(dir.path, path);
            assert_eq!(dir.rank, 2.0);
            assert_eq!(dir.last_accessed, now);
        }
    }

    #[test]
    fn open_with_utf8_bom() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        let contents = format!("\u{FEFF}{now:010}\t0000002.00\t{path}\n");
        fs::write(data_dir.path().join("db.txt"), contents).unwrap();

        let db = Database::open_dir(data_dir.path()).unwrap();
        assert_eq!(db.dirs().len(), 1);

        let dir = &db.dirs()[0];
        assert_eq!(dir.path, path);
        assert_eq!(dir.rank, 2.0);
        assert_eq!(dir.last_accessed, now);
    }

    #[test]
    fn fast_format_matches_std_fmt() {
        for val in [0, 1, 9, 42, 999_999_999, 9_999_999_999, 10_000_000_000, u64::MAX] {
            let mut buf = Vec::new();
            write_padded_u64(&mut buf, val);
            assert_eq!(String::from_utf8(buf).unwrap(), format!("{val:0COL_WIDTH$}"));
        }

        for rank in [MIN_RANK, 0.005, 0.1, 1.0, 1.005, 1.25, 99.99, 12345.67, 999_999.995, MAX_RANK]
        {
            let mut buf = Vec::new();
            write_padded_rank(&mut buf, rank).unwrap();
            assert_eq!(String::from_utf8(buf).unwrap(), format!("{rank:0COL_WIDTH$.2}"));
        }
    }

    #[test]
    fn remove() {
        let data_dir = tempfile::tempdir().unwrap();
        let path = if cfg!(windows) { r"C:\foo\bar" } else { "/foo/bar" };
        let now = 946684800;

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            db.add(path, 1.0, now);
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.remove(path));
            db.save().unwrap();
        }

        {
            let mut db = Database::open_dir(data_dir.path()).unwrap();
            assert!(db.dirs().is_empty());
            assert!(!db.remove(path));
            db.save().unwrap();
        }
    }
}
