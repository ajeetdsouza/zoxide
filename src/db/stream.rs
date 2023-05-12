use std::iter::Rev;
use std::ops::Range;
use std::path::Path;
use std::{fs, path};

use crate::db::{Database, Dir, Epoch};
use crate::util::{self, MONTH};

pub struct Stream<'a> {
    // State
    db: &'a mut Database,
    idxs: Rev<Range<usize>>,
    did_exclude: bool,

    // Configuration
    keywords: Vec<String>,
    check_exists: bool,
    expire_below: Epoch,
    resolve_symlinks: bool,
    exclude_path: Option<String>,
}

impl<'a> Stream<'a> {
    pub fn new(db: &'a mut Database, now: Epoch) -> Self {
        db.sort_by_score(now);
        let idxs = (0..db.dirs().len()).rev();

        // If a directory is deleted and hasn't been used for 3 months, delete
        // it from the database.
        let expire_below = now.saturating_sub(3 * MONTH);

        Stream {
            db,
            idxs,
            did_exclude: false,
            keywords: Vec::new(),
            check_exists: false,
            expire_below,
            resolve_symlinks: false,
            exclude_path: None,
        }
    }

    pub fn with_exclude(mut self, path: impl Into<String>) -> Self {
        self.exclude_path = Some(path.into());
        self
    }

    pub fn with_exists(mut self, resolve_symlinks: bool) -> Self {
        self.check_exists = true;
        self.resolve_symlinks = resolve_symlinks;
        self
    }

    pub fn with_keywords(mut self, keywords: &[impl AsRef<str>]) -> Self {
        self.keywords = keywords.iter().map(util::to_lowercase).collect();
        self
    }

    pub fn next(&mut self) -> Option<&Dir> {
        while let Some(idx) = self.idxs.next() {
            let dir = &self.db.dirs()[idx];

            if !self.matches_keywords(&dir.path) {
                continue;
            }

            if !self.matches_exists(&dir.path) {
                if dir.last_accessed < self.expire_below {
                    self.db.swap_remove(idx);
                }
                continue;
            }

            if self
                .exclude_path
                .as_deref()
                .map(|a| dir.path.to_str().map(|x| x == a))
                .flatten()
                .unwrap_or_default()
            {
                self.did_exclude = true;
                continue;
            }

            let dir = &self.db.dirs()[idx];
            return Some(dir);
        }

        None
    }

    pub fn did_exclude(&self) -> bool {
        self.did_exclude
    }

    fn matches_exists(&self, path: &Path) -> bool {
        if !self.check_exists {
            return true;
        }
        let resolver = if self.resolve_symlinks { fs::symlink_metadata } else { fs::metadata };
        resolver(path).map(|m| m.is_dir()).unwrap_or_default()
    }

    fn matches_keywords(&self, path: &Path) -> bool {
        let (keywords_last, keywords) = match self.keywords.split_last() {
            Some(split) => split,
            None => return true,
        };
        let Some(path) = path.to_str() else {
            return false;
        };
        let path = util::to_lowercase(path);
        let mut path = path.as_str();
        match path.rfind(keywords_last) {
            Some(idx) => {
                if path[idx + keywords_last.len()..].contains(path::is_separator) {
                    return false;
                }
                path = &path[..idx];
            }
            None => return false,
        }

        for keyword in keywords.iter().rev() {
            match path.rfind(keyword) {
                Some(idx) => path = &path[..idx],
                None => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    #[rstest]
    // Case normalization
    #[case(&["fOo", "bAr"], "/foo/bar", true)]
    // Last component
    #[case(&["ba"], "/foo/bar", true)]
    #[case(&["fo"], "/foo/bar", false)]
    // Slash as suffix
    #[case(&["foo/"], "/foo", false)]
    #[case(&["foo/"], "/foo/bar", true)]
    #[case(&["foo/"], "/foo/bar/baz", false)]
    #[case(&["foo", "/"], "/foo", false)]
    #[case(&["foo", "/"], "/foo/bar", true)]
    #[case(&["foo", "/"], "/foo/bar/baz", true)]
    // Split components
    #[case(&["/", "fo", "/", "ar"], "/foo/bar", true)]
    #[case(&["oo/ba"], "/foo/bar", true)]
    // Overlap
    #[case(&["foo", "o", "bar"], "/foo/bar", false)]
    #[case(&["/foo/", "/bar"], "/foo/bar", false)]
    #[case(&["/foo/", "/bar"], "/foo/baz/bar", true)]
    fn query(#[case] keywords: &[&str], #[case] path: &str, #[case] is_match: bool) {
        let db = &mut Database::new(PathBuf::new(), Vec::new(), |_| Vec::new(), false);
        let stream = Stream::new(db, 0).with_keywords(keywords);
        assert_eq!(is_match, stream.matches_keywords(Path::new(path)));
    }
}
