use std::borrow::Cow;

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

pub type Rank = f64;
pub type Epoch = u64;
