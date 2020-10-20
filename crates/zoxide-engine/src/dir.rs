use crate::query::Query;

use serde::{Deserialize, Serialize};

use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir {
    pub path: String,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir {
    pub fn is_dir(&self) -> bool {
        Path::new(&self.path).is_dir()
    }

    pub fn is_match(&self, query: &Query) -> bool {
        query.matches(&self.path)
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
}

pub type Rank = f64;
pub type Epoch = u64;
