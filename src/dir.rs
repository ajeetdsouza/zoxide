use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub use f64 as Rank;
pub use i64 as Epoch; // use a signed integer so subtraction can be performed on it

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir {
    pub path: PathBuf,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir {
    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    #[cfg(unix)]
    pub fn is_match(&self, query: &[String]) -> bool {
        use bstr::ByteSlice;

        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path_lower = self.path.as_os_str().as_bytes().to_lowercase();

        if let Some(query_name) = query
            .last()
            .and_then(|query_last| Path::new(query_last).file_name())
        {
            if let Some(dir_name) = Path::new(OsStr::from_bytes(&path_lower)).file_name() {
                let dir_name_bytes = dir_name.as_bytes();
                let query_name_bytes = query_name.as_bytes();

                if !dir_name_bytes.contains_str(query_name_bytes) {
                    return false;
                }
            }
        }

        let mut subpath = path_lower.as_slice();

        for subquery in query.iter() {
            let subquery_bytes = subquery.as_bytes();
            match subpath.find(subquery_bytes) {
                Some(idx) => subpath = &subpath[idx + subquery_bytes.len()..],
                None => return false,
            }
        }

        true
    }

    #[cfg(not(unix))]
    pub fn is_match(&self, query: &[String]) -> bool {
        let path_lower = match self.path.to_str() {
            Some(path_str) => path_str.to_lowercase(),
            None => return false, // silently ignore invalid UTF-8
        };

        let mut subpath = path_lower.as_str();

        if let Some(query_name) = query
            .last()
            .and_then(|query_last| Path::new(query_last).file_name())
        {
            if let Some(dir_name) = Path::new(&path_lower).file_name() {
                // unwrap is safe here because we've already handled invalid UTF-8
                let dir_name_str = dir_name.to_str().unwrap();
                let query_name_str = query_name.to_str().unwrap();

                if !dir_name_str.contains(query_name_str) {
                    return false;
                }
            }
        }

        for subquery in query.iter() {
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
