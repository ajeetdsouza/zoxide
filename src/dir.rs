use crate::types::{Rank, Epoch};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Dir {
    pub path: String,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir {
    pub fn is_dir(&self) -> bool {
        Path::new(&self.path).is_dir()
    }

    pub fn is_match(&self, query: &[String]) -> bool {
        let path = self.path.to_ascii_lowercase();

        if let Some(query_name) = query.last().and_then(|word| Path::new(word).file_name()) {
            if let Some(path_name) = Path::new(&path).file_name() {
                // `unwrap()` here should be safe because the values are already encoded as UTF-8
                let query_name = query_name.to_str().unwrap();
                let path_name = path_name.to_str().unwrap();

                if !path_name.contains(&query_name) {
                    return false;
                }
            }
        }

        let mut subpath = path.as_str();
        for subquery in query {
            match subpath.find(subquery) {
                Some(idx) => subpath = &subpath[idx + subquery.len()..],
                None => return false,
            }
        }

        true
    }

    pub fn get_frecency(&self, now: Epoch) -> Rank {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

        let duration = now - self.last_accessed;
        if duration < HOUR {
            self.rank * 4.0
        } else if duration < DAY {
            self.rank * 2.0
        } else if duration < WEEK {
            self.rank / 2.0
        } else {
            self.rank / 4.0
        }
    }
}
