use std::{fs, path};

use ordered_float::OrderedFloat;

use super::{Database, Dir, Epoch};
use crate::util;

pub struct Stream<'db, 'file> {
    db: &'db mut Database<'file>,

    keywords: Vec<String>,

    check_exists: bool,
    expire_below: Epoch,
    resolve_symlinks: bool,

    exclude_path: Option<String>,
    now: Epoch,
}

impl<'db, 'file> Stream<'db, 'file> {
    pub fn new(db: &'db mut Database<'file>, now: Epoch) -> Self {
        // If a directory is deleted and hasn't been used for 90 days, delete it from the database.
        let expire_below = now.saturating_sub(90 * 24 * 60 * 60);

        Stream {
            db,
            keywords: Vec::new(),
            check_exists: false,
            expire_below,
            resolve_symlinks: false,
            exclude_path: None,
            now,
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

    pub fn into_iter(self) -> StreamIterator<'db, 'file> {
        let mut idxs: Vec<_> = self.db.dirs.iter()
            .enumerate() // store the original indices before filtering
            .filter(|(_idx, dir)|
                self.matches_keywords(&dir.path) &&
                    Some(dir.path.as_ref()) != self.exclude_path.as_deref())
            .collect();

        // Iterate in descending order of score.
        idxs.sort_by_cached_key(|(_idx, dir)| {
            let (kw_score, frequency_score) = dir.score(self.now, &self.keywords);
            (kw_score, OrderedFloat(frequency_score))
        });
        let idxs = idxs.into_iter().map(|(idx, _)| idx).rev().collect::<Vec<_>>().into_iter(); // copy the indices to avoid lifetime issues

        StreamIterator { stream: self, idxs: Box::new(idxs) }
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

pub struct StreamIterator<'db, 'file> {
    stream: Stream<'db, 'file>,
    idxs: Box<dyn Iterator<Item = usize>>,
}

impl<'db, 'file> StreamIterator<'db, 'file> {
    pub fn next(&mut self) -> Option<&Dir<'file>> {
        while let Some(idx) = self.idxs.next() {
            let dir = &self.stream.db.dirs[idx];

            if !self.stream.matches_exists(&dir.path) {
                if dir.last_accessed < self.stream.expire_below {
                    self.stream.db.dirs.swap_remove(idx);
                    self.stream.db.modified = true;
                }
                continue;
            }

            let dir = &self.stream.db.dirs[idx];
            return Some(dir);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::Database;

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
