use crate::util;

use std::fs;
use std::path;

#[derive(Debug, Default)]
pub struct Matcher {
    keywords: Vec<String>,
    check_exists: bool,
    resolve_symlinks: bool,
}

impl Matcher {
    pub fn new() -> Matcher {
        Matcher::default()
    }

    pub fn with_exists(mut self, resolve_symlinks: bool) -> Matcher {
        self.check_exists = true;
        self.resolve_symlinks = resolve_symlinks;
        self
    }

    pub fn with_keywords<S: AsRef<str>>(mut self, keywords: &[S]) -> Matcher {
        self.keywords = keywords.iter().map(util::to_lowercase).collect();
        self
    }

    pub fn matches<S: AsRef<str>>(&self, path: S) -> bool {
        self.matches_keywords(&path) && self.matches_exists(path)
    }

    fn matches_exists<S: AsRef<str>>(&self, path: S) -> bool {
        if !self.check_exists {
            return true;
        }

        let resolver = if self.resolve_symlinks {
            fs::symlink_metadata
        } else {
            fs::metadata
        };

        resolver(path.as_ref())
            .map(|m| m.is_dir())
            .unwrap_or_default()
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
    use super::Matcher;

    #[test]
    fn query() {
        const CASES: &[(&[&str], &str, bool)] = &[
            // Case normalization
            (&["fOo", "bAr"], "/foo/bar", true),
            // Last component
            (&["ba"], "/foo/bar", true),
            (&["fo"], "/foo/bar", false),
            // Slash as suffix
            (&["foo/"], "/foo", false),
            (&["foo/"], "/foo/bar", true),
            (&["foo/"], "/foo/bar/baz", false),
            (&["foo", "/"], "/foo", false),
            (&["foo", "/"], "/foo/bar", true),
            (&["foo", "/"], "/foo/bar/baz", true),
            // Split components
            (&["/", "fo", "/", "ar"], "/foo/bar", true),
            (&["oo/ba"], "/foo/bar", true),
            // Overlap
            (&["foo", "o", "bar"], "/foo/bar", false),
            (&["/foo/", "/bar"], "/foo/bar", false),
            (&["/foo/", "/bar"], "/foo/baz/bar", true),
        ];

        for &(keywords, path, is_match) in CASES {
            let matcher = Matcher::new().with_keywords(keywords);
            assert_eq!(is_match, matcher.matches(path))
        }
    }
}
