use super::{Epoch, Query, Rank};

use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir {
    pub path: String,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir {
    pub fn is_match(&self, query: &Query) -> bool {
        query.matches(&self.path) && Path::new(&self.path).is_dir()
    }

    pub fn get_score(&self, now: Epoch) -> Rank {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

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

    pub fn display(&self) -> DirDisplay {
        DirDisplay { dir: self }
    }

    pub fn display_score(&self, now: Epoch) -> DirDisplayScore {
        DirDisplayScore { dir: self, now }
    }
}

pub struct DirDisplay<'a> {
    dir: &'a Dir,
}

impl Display for DirDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dir.path)
    }
}

pub struct DirDisplayScore<'a> {
    dir: &'a Dir,
    now: Epoch,
}

impl Display for DirDisplayScore<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let score = self.dir.get_score(self.now);
        let score = if score > 9999.0 {
            9999
        } else if score > 0.0 {
            score as _
        } else {
            0
        };
        write!(f, "{:>4} {}", score, self.dir.path)
    }
}
