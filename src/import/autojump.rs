use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::{env, str};

use anyhow::{Context, Result, anyhow};

use crate::db::Dir;
use crate::import::{ImportError, Importer};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct Autojump {}

impl Importer for Autojump {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        let path = data_path()?;
        let file = File::open(&path).with_context(|| format!("could not read {path:?}"))?;
        let reader = BufReader::new(file);
        Ok(Iter::new(reader, path))
    }
}

struct Iter<R: BufRead> {
    reader: R,
    buf: Vec<u8>,
    line_num: usize,
    path: PathBuf,
}

impl<R: BufRead> Iter<R> {
    fn new(reader: R, path: PathBuf) -> Self {
        Self { reader, buf: Vec::new(), line_num: 0, path }
    }

    fn err(&self, source: anyhow::Error) -> ImportError {
        ImportError { path: Some(self.path.clone()), line_num: self.line_num, source }
    }

    fn parse_line(&self, line: &[u8]) -> Result<Dir<'static>, ImportError> {
        let line =
            str::from_utf8(line).map_err(|e| self.err(anyhow!(e).context("invalid utf-8")))?;

        let (rank, path) =
            line.split_once('\t').ok_or_else(|| self.err(anyhow!("invalid entry: {line}")))?;
        let rank = rank
            .parse::<f64>()
            .map_err(|e| self.err(anyhow!(e).context(format!("invalid rank: {rank}"))))?;

        // Normalize the rank using a sigmoid function. Don't import actual ranks from
        // autojump, since its scoring algorithm is very different and might
        // take a while to normalize.
        let rank = sigmoid(rank);

        Ok(Dir { path: Cow::Owned(path.to_string()), rank, last_accessed: 0 })
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

/// Mirrors autojump's path logic:
///
/// ```python
/// if is_osx():
///     data_home = os.path.join(os.path.expanduser('~'), 'Library')
/// elif is_windows():
///     data_home = os.getenv('APPDATA')
/// else:
///     data_home = os.getenv(
///         'XDG_DATA_HOME',
///         os.path.join(os.path.expanduser('~'), '.local', 'share'),
///     )
/// data_path = os.path.join(data_home, 'autojump', 'autojump.txt')
/// ```
fn data_path() -> Result<PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        let mut path = dirs::home_dir().context("could not find home directory")?;
        path.push("Library");
        path
    } else if cfg!(target_os = "windows") {
        let appdata = env::var_os("APPDATA").context("%APPDATA% is not set")?;
        PathBuf::from(appdata)
    } else if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(xdg)
    } else {
        let mut path = dirs::home_dir().context("could not find home directory")?;
        path.push(".local");
        path.push("share");
        path
    };
    path.push("autojump");
    path.push("autojump.txt");
    Ok(path)
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
