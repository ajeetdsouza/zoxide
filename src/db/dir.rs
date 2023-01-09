use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::util::{DAY, HOUR, WEEK};

#[derive(Clone, Debug, Deserialize, Serialize)]
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
}

impl<'a> DirDisplay<'a> {
    fn new(dir: &'a Dir) -> Self {
        Self { dir, separator: ' ', now: None }
    }

    pub fn with_score(mut self, now: Epoch) -> Self {
        self.now = Some(now);
        self
    }

    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }
}

impl Display for DirDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(now) = self.now {
            let score = self.dir.score(now).clamp(0.0, 9999.0);
            write!(f, "{score:>6.1}{}", self.separator)?;
        }
        write!(f, "{}", self.dir.path)
    }
}

pub type Rank = f64;
pub type Epoch = u64;
