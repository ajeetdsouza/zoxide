use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::config::RankingMode;
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

    pub fn score(&self, now: Epoch, mode: RankingMode) -> Rank {
        match mode {
            RankingMode::Frecency => self.frecency(now),
            RankingMode::Recency => self.last_accessed as Rank,
        }
    }

    pub(crate) fn frecency(&self, now: Epoch) -> Rank {
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
            // Always display the frecency value so that `--score` stays
            // human-readable regardless of the active ranking mode.
            let score = self.dir.frecency(now).clamp(0.0, 9999.0);
            write!(f, "{score:>6.1}{}", self.separator)?;
        }
        write!(f, "{}", self.dir.path)
    }
}

pub type Rank = f64;
pub type Epoch = u64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::HOUR;

    fn dir(path: &'static str, rank: Rank, last_accessed: Epoch) -> Dir<'static> {
        Dir { path: Cow::Borrowed(path), rank, last_accessed }
    }

    #[test]
    fn frecency_mode_prefers_high_rank() {
        let now = 10 * HOUR;
        // Same age bucket (>1h, <1d). Higher rank wins.
        let popular = dir("/popular", 50.0, now - 2 * HOUR);
        let recent = dir("/recent", 1.0, now - 2 * HOUR);
        assert!(
            popular.score(now, RankingMode::Frecency) > recent.score(now, RankingMode::Frecency)
        );
    }

    #[test]
    fn recency_mode_prefers_recent_access() {
        let now = 10 * HOUR;
        let popular_but_old = dir("/popular", 50.0, now - 5 * HOUR);
        let unpopular_but_recent = dir("/recent", 1.0, now - HOUR);
        assert!(
            unpopular_but_recent.score(now, RankingMode::Recency)
                > popular_but_old.score(now, RankingMode::Recency)
        );
    }

    #[test]
    fn recency_mode_ignores_rank() {
        let now = 10 * HOUR;
        let a = dir("/a", 1.0, now - HOUR);
        let b = dir("/b", 9999.0, now - HOUR);
        assert_eq!(a.score(now, RankingMode::Recency), b.score(now, RankingMode::Recency));
    }
}
