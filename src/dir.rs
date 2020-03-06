use crate::types::{Rank, Timestamp};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Dir {
    pub path: String,
    pub rank: Rank,
    pub last_accessed: Timestamp,
}

impl Dir {
    pub fn is_dir(&self) -> bool {
        Path::new(&self.path).is_dir()
    }

    pub fn is_match(&self, query: &[String]) -> bool {
        let path = self.path.to_ascii_lowercase();

        if let Some(dir_name) = Path::new(&path).file_name() {
            if let Some(query_last) = query.last() {
                if let Some(query_dir_name) = Path::new(query_last).file_name() {
                    // `unwrap()` here should be safe because the values are already encoded as UTF-8
                    let dir_name_str = dir_name.to_str().unwrap().to_ascii_lowercase();
                    let query_dir_name_str = query_dir_name.to_str().unwrap().to_ascii_lowercase();

                    if !dir_name_str.contains(&query_dir_name_str) {
                        return false;
                    }
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

    pub fn get_frecency(&self, now: Timestamp) -> Rank {
        const HOUR: Timestamp = 60 * 60;
        const DAY: Timestamp = 24 * HOUR;
        const WEEK: Timestamp = 7 * DAY;

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
