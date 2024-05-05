use std::iter::Rev;
use std::ops::Range;
use std::{fs, path};

use glob::Pattern;

use crate::db::{Database, Dir, Epoch};
use crate::util::{self, MONTH};

pub struct Stream<'a> {
    db: &'a mut Database,
    idxs: Rev<Range<usize>>,
    options: StreamOptions,
}

impl<'a> Stream<'a> {
    pub fn new(db: &'a mut Database, options: StreamOptions) -> Self {
        db.sort_by_score(options.now);
        let idxs = (0..db.dirs().len()).rev();
        Stream { db, idxs, options }
    }

    pub fn next(&mut self) -> Option<&Dir> {
        while let Some(idx) = self.idxs.next() {
            let dir = &self.db.dirs()[idx];

            if !self.filter_by_keywords(&dir.path) {
                continue;
            }

            if !self.filter_by_exclude(&dir.path) {
                self.db.swap_remove(idx);
                continue;
            }

            if !self.filter_by_exists(&dir.path) {
                if dir.last_accessed < self.options.ttl {
                    self.db.swap_remove(idx);
                }
                continue;
            }

            let dir = &self.db.dirs()[idx];
            return Some(dir);
        }

        None
    }

    fn filter_by_keywords(&self, path: &str) -> bool {
        let (keywords_last, keywords) = match self.options.keywords.split_last() {
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

    fn filter_by_exclude(&self, path: &str) -> bool {
        !self.options.exclude.iter().any(|pattern| pattern.matches(path))
    }

    fn filter_by_exists(&self, path: &str) -> bool {
        if !self.options.exists {
            return true;
        }
        let resolver =
            if self.options.resolve_symlinks { fs::metadata } else { fs::symlink_metadata };
        resolver(path).map(|metadata| metadata.is_dir()).unwrap_or_default()
    }
}

pub struct StreamOptions {
    /// The current time.
    now: Epoch,

    /// Only directories matching these keywords will be returned.
    keywords: Vec<String>,

    /// Directories that match any of these globs will be lazily removed.
    exclude: Vec<Pattern>,

    /// Directories will only be returned if they exist on the filesystem.
    exists: bool,

    /// Whether to resolve symlinks when checking if a directory exists.
    resolve_symlinks: bool,

    /// Directories that do not exist and haven't been accessed since TTL will
    /// be lazily removed.
    ttl: Epoch,
}

impl StreamOptions {
    pub fn new(now: Epoch) -> Self {
        StreamOptions {
            now,
            keywords: Vec::new(),
            exclude: Vec::new(),
            exists: false,
            resolve_symlinks: false,
            ttl: now.saturating_sub(3 * MONTH),
        }
    }

    pub fn with_keywords<I>(mut self, keywords: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.keywords = keywords.into_iter().map(util::to_lowercase).collect();
        self
    }

    pub fn with_exclude(mut self, exclude: Vec<Pattern>) -> Self {
        self.exclude = exclude;
        self
    }

    pub fn with_exists(mut self, exists: bool) -> Self {
        self.exists = exists;
        self
    }

    pub fn with_resolve_symlinks(mut self, resolve_symlinks: bool) -> Self {
        self.resolve_symlinks = resolve_symlinks;
        self
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
        let options = StreamOptions::new(0).with_keywords(keywords.iter());
        let stream = Stream::new(db, options);
        assert_eq!(is_match, stream.filter_by_keywords(path));
    }
}
