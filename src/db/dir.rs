use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::util::{DAY, HOUR, WEEK};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirV4<'a> {
    #[serde(borrow)]
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
    #[serde(borrow)]
    pub aliases: HashSet<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirV3<'a> {
    #[serde(borrow)]
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

pub trait Dir {
    fn path(&self) -> &str;
    fn score(&self, now: Epoch) -> Rank;
    fn aliases(&self) -> Option<impl Iterator<Item = impl AsRef<str>>>;
}

impl Dir for DirV4<'_> {
    fn path(&self) -> &str {
        &self.path
    }

    fn score(&self, now: Epoch) -> Rank {
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

    fn aliases(&self) -> Option<impl Iterator<Item = impl AsRef<str>>> {
        Some(self.aliases.iter())
    }
}

impl DirV4<'_> {
    pub fn display(&self) -> DirDisplay<'_, Self> {
        DirDisplay::new(self)
    }
}

impl Dir for DirV3<'_> {
    fn path(&self) -> &str {
        &self.path
    }

    fn score(&self, now: Epoch) -> Rank {
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

    fn aliases(&self) -> Option<impl Iterator<Item = impl AsRef<str>>> {
        let arr: Option<&[&str]> = None;
        arr.map(|a| a.iter())
    }
}

pub struct DirDisplay<'a, T: Dir> {
    dir: &'a T,
    now: Option<Epoch>,
    separator: char,
    aliases: bool,
}

impl<'a, T: Dir> DirDisplay<'a, T> {
    fn new(dir: &'a T) -> Self {
        Self { dir, separator: ' ', now: None, aliases: false }
    }

    pub fn with_score(mut self, now: Epoch) -> Self {
        self.now = Some(now);
        self
    }

    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    pub fn with_aliases(mut self, enable: bool) -> Self {
        self.aliases = enable;
        self
    }
}

impl<'a, T: Dir> Display for DirDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(now) = self.now {
            let score = self.dir.score(now).clamp(0.0, 9999.0);
            write!(f, "{score:>6.1}{}", self.separator)?;
        }

        if self.aliases
            && let Some(aliases) = self.dir.aliases()
        {
            for alias in aliases {
                write!(f, "{} ", alias.as_ref())?;
            }
            write!(f, "{}", self.separator)?;
        }

        write!(f, "{}", self.dir.path())
    }
}

pub type Rank = f64;
pub type Epoch = u64;
