use std::borrow::Cow;
use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::str;

use anyhow::{Context, Result, anyhow};

use crate::db::{Dir, Epoch};
use crate::import::{ImportError, Importer};

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct Atuin {}

impl Importer for Atuin {
    fn dirs(&self) -> Result<impl Iterator<Item = Result<Dir<'static>, ImportError>>> {
        // atuin renders `{time}` as `YYYY-MM-DD HH:MM:SS` in UTC.
        let mut child = Command::new("atuin")
            .args(["history", "list", "--format={time}\t{directory}", "--print0"])
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to run `atuin`; is it installed and on PATH?")?;
        let stdout = child.stdout.take().expect("stdout piped");
        let reader = BufReader::new(stdout);
        Ok(Iter::new(reader, child))
    }
}

/// Iterates atuin's NUL-separated `{time}\t{directory}` records, emitting one
/// `Dir` per directory transition (consecutive same-path records collapse).
/// Owns the `Child` handle so the subprocess is reaped on Drop.
struct Iter {
    reader: BufReader<ChildStdout>,
    buf: Vec<u8>,
    line_num: usize,

    child: Child,
    prev_cwd: Option<String>,
}

impl Iter {
    fn new(reader: BufReader<ChildStdout>, child: Child) -> Self {
        Self { reader, buf: Vec::new(), line_num: 0, child, prev_cwd: None }
    }

    fn err(&self, source: anyhow::Error) -> ImportError {
        ImportError { path: None, line_num: self.line_num, source }
    }

    fn parse_line(&self, line: &[u8]) -> Result<Dir<'static>, ImportError> {
        let line =
            str::from_utf8(line).map_err(|e| self.err(anyhow!(e).context("invalid utf-8")))?;

        let (timestamp, path) =
            line.split_once('\t').ok_or_else(|| self.err(anyhow!("invalid entry: {line}")))?;

        let timestamp_format =
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let timestamp = time::PrimitiveDateTime::parse(timestamp, timestamp_format)
            .map_err(|e| self.err(anyhow!(e).context(format!("invalid timestamp: {timestamp:?}"))))?
            .assume_utc()
            .unix_timestamp();

        let dir = Dir {
            path: Cow::Owned(path.to_string()),
            rank: 1.0,
            last_accessed: timestamp as Epoch,
        };
        Ok(dir)
    }
}

impl Iterator for Iter {
    type Item = Result<Dir<'static>, ImportError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            self.line_num += 1;

            match self.reader.read_until(b'\0', &mut self.buf) {
                Ok(0) => return None,
                Ok(_) => {
                    if self.buf.last() == Some(&b'\0') {
                        self.buf.pop();
                    }
                    if self.buf.is_empty() {
                        continue;
                    }

                    let result = self.parse_line(&self.buf);
                    match &result {
                        Ok(dir) => {
                            let path = dir.path.as_ref();
                            if self.prev_cwd.as_deref() == Some(path) {
                                continue; // dedup consecutive same-path entries
                            }
                            self.prev_cwd = Some(path.to_string());
                            return Some(result);
                        }
                        Err(_) => return Some(result),
                    }
                }
                Err(e) => {
                    return Some(Err(self.err(anyhow!(e).context("could not read from atuin"))));
                }
            }
        }
    }
}

impl Drop for Iter {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
