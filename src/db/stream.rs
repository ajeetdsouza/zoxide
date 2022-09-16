use std::iter::Rev;
use std::ops::Range;
use std::{fs, path};

use crate::db::{Database, Dir, Epoch};
use crate::util;

pub struct Stream<'db, 'file> {
    db: &'db mut Database<'file>,
    idxs: Rev<Range<usize>>,

    keywords: Vec<String>,

    check_exists: bool,
    expire_below: Epoch,
    resolve_symlinks: bool,

    exclude_path: Option<String>,
}

impl<'db, 'file> Stream<'db, 'file> {
    pub fn new(db: &'db mut Database<'file>, now: Epoch) -> Self {
        // Iterate in descending order of score.
        db.dirs.sort_unstable_by(|dir1, dir2| dir1.score(now).total_cmp(&dir2.score(now)));
        let idxs = (0..db.dirs.len()).rev();

        // If a directory is deleted and hasn't been used for 90 days, delete it from the database.
        let expire_below = now.saturating_sub(90 * 24 * 60 * 60);

        Stream {
            db,
            idxs,
            keywords: Vec::new(),
            check_exists: false,
            expire_below,
            resolve_symlinks: false,
            exclude_path: None,
        }
    }

    pub fn with_exclude<S: Into<String>>(mut self, path: S) -> Self {
        self.exclude_path = Some(path.into());
        self
    }

    pub fn with_exists(mut self, resolve_symlinks: bool) -> Self {
        self.check_exists = true;
        self.resolve_symlinks = resolve_symlinks;
        self
    }

    pub fn with_keywords<S: AsRef<str>>(mut self, keywords: &[S]) -> Self {
        self.keywords = keywords.iter().map(util::to_lowercase).collect();
        self
    }

    pub fn next(&mut self) -> Option<&Dir<'file>> {
        while let Some(idx) = self.idxs.next() {
            let dir = &self.db.dirs[idx];

            if !self.matches_keywords(&dir.path) {
                continue;
            }

            if !self.matches_exists(&dir.path) {
                if dir.last_accessed < self.expire_below {
                    self.db.dirs.swap_remove(idx);
                    self.db.modified = true;
                }
                continue;
            }

            if Some(dir.path.as_ref()) == self.exclude_path.as_deref() {
                continue;
            }

            let dir = &self.db.dirs[idx];
            return Some(dir);
        }

        None
    }

    fn matches_exists<S: AsRef<str>>(&self, path: S) -> bool {
        if !self.check_exists {
            return true;
        }
        let resolver = if self.resolve_symlinks { fs::symlink_metadata } else { fs::metadata };
        resolver(path.as_ref()).map(|m| m.is_dir()).unwrap_or_default()
    }

    fn matches_keywords<S: AsRef<str>>(&self, path: S) -> bool {
        let (keywords_last, keywords) = match self.keywords.split_last() {
            Some(split) => split,
            None => return true,
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
        let mut db = Database { dirs: Vec::new().into(), modified: false, data_dir: &PathBuf::new() };
        let stream = db.stream(0).with_keywords(keywords);
        assert_eq!(is_match, stream.matches_keywords(path));
    }
}
