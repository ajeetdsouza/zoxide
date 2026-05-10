use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str;

use anyhow::{Context, Result, anyhow};

use crate::db::Dir;
use crate::import::{ImportError, Importer};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct Z {}

impl Importer for Z {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        let path = data_path()?;
        let file = File::open(&path).with_context(|| format!("could not read {path:?}"))?;
        let reader = BufReader::new(file);
        Ok(Iter::new(reader, path))
    }
}

pub(crate) struct Iter<R: BufRead> {
    reader: R,
    buf: Vec<u8>,
    line_num: usize,
    path: PathBuf,
}

impl<R: BufRead> Iter<R> {
    pub(crate) fn new(reader: R, path: PathBuf) -> Self {
        Self { reader, buf: Vec::new(), line_num: 0, path }
    }

    fn err(&self, source: anyhow::Error) -> ImportError {
        ImportError { path: Some(self.path.clone()), line_num: self.line_num, source }
    }

    fn parse_line(&self, line: &[u8]) -> Result<Dir<'static>, ImportError> {
        let line =
            str::from_utf8(line).map_err(|e| self.err(anyhow!(e).context("invalid utf-8")))?;
        let err = || self.err(anyhow!("invalid entry: {line}"));

        // z stores entries as `path|rank|last_accessed`. Use `rsplitn` so paths
        // containing `|` are preserved.
        let mut split = line.rsplitn(3, '|');

        let last_accessed = split.next().ok_or_else(err)?;
        let last_accessed = last_accessed.parse::<u64>().map_err(|_| err())?;

        let rank = split.next().ok_or_else(err)?;
        let rank = rank.parse::<f64>().map_err(|_| err())?;

        let path = split.next().ok_or_else(err)?;

        Ok(Dir { path: Cow::Owned(path.to_string()), rank, last_accessed })
    }
}

impl<R: BufRead> Iterator for Iter<R> {
    type Item = Result<Dir<'static>, ImportError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            self.line_num += 1;

            match self.reader.read_until(b'\n', &mut self.buf) {
                Ok(0) => return None,
                Ok(_) => {
                    if self.buf.last() == Some(&b'\n') {
                        self.buf.pop();
                    }
                    if self.buf.last() == Some(&b'\r') {
                        self.buf.pop();
                    }
                    if self.buf.is_empty() {
                        continue;
                    }
                    return Some(self.parse_line(&self.buf));
                }
                Err(e) => return Some(Err(self.err(anyhow::Error::from(e)))),
            }
        }
    }
}

/// Mirrors z's path logic:
///
/// ```sh
/// local datafile="${_Z_DATA:-$HOME/.z}"
/// ```
fn data_path() -> Result<PathBuf> {
    match env::var_os("_Z_DATA") {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            let mut path = dirs::home_dir().context("could not find home directory")?;
            path.push(".z");
            Ok(path)
        }
    }
}
