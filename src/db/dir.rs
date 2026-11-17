use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::path::{MAIN_SEPARATOR, Path};

use serde::Deserialize;

use crate::util::{DAY, HOUR, WEEK};

#[derive(Clone, Debug, Deserialize)]
pub struct Dir<'a> {
    #[serde(borrow)]
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir<'_> {
    pub fn display(&self) -> DirDisplay<'_> {
        DirDisplay::new(self)
    }

    pub fn score(&self, now: Epoch) -> Rank {
        // The older the entry, the lesser its importance.
        let duration = now.saturating_sub(self.last_accessed);
        if duration < HOUR {
            self.rank * 4.0
        } else if duration < DAY {
            self.rank * 2.0
        } else if duration < WEEK {
            self.rank * 0.5
        } else {
            self.rank * 0.25
        }
    }
}

pub struct DirDisplay<'a> {
    dir: &'a Dir<'a>,
    now: Option<Epoch>,
    separator: char,
    home: Option<&'a str>,
}

impl<'a> DirDisplay<'a> {
    fn new(dir: &'a Dir) -> Self {
        Self { dir, separator: ' ', now: None, home: None }
    }

    pub fn with_score(mut self, now: Epoch) -> Self {
        self.now = Some(now);
        self
    }

    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Abbreviates paths under the given home directory using `~`.
    pub fn with_home(mut self, home: Option<&'a str>) -> Self {
        self.home = home;
        self
    }
}

impl Display for DirDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(now) = self.now {
            let score = self.dir.score(now).clamp(0.0, 9999.0);
            write!(f, "{score:>6.1}{}", self.separator)?;
        }

        // `strip_prefix` compares whole components, so `/home/alice2` is not
        // treated as being under `/home/alice`.
        let path = self.dir.path.as_ref();
        let rest = self.home.and_then(|home| Path::new(path).strip_prefix(home).ok());
        match rest.and_then(Path::to_str) {
            Some("") => write!(f, "~"),
            Some(rest) => write!(f, "~{MAIN_SEPARATOR}{rest}"),
            None => write!(f, "{path}"),
        }
    }
}

pub type Rank = f64;
pub type Epoch = u64;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/home/alice/foo/bar", Some("/home/alice"), "~/foo/bar")]
    #[case("/home/alice", Some("/home/alice"), "~")]
    #[case("/home/alice2/foo", Some("/home/alice"), "/home/alice2/foo")]
    #[case("/home", Some("/home/alice"), "/home")]
    #[case("/usr/bin", Some("/home/alice"), "/usr/bin")]
    #[case("/home/alice/foo", None, "/home/alice/foo")]
    fn display_home(#[case] path: &str, #[case] home: Option<&str>, #[case] expected: &str) {
        let dir = Dir { path: path.into(), rank: 1.0, last_accessed: 0 };
        let display = dir.display().with_home(home).to_string();
        assert_eq!(expected, display.replace(MAIN_SEPARATOR, "/"));
    }
}
