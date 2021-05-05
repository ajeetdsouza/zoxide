use crate::util;

use std::path;

pub struct Query(Vec<String>);

impl Query {
    pub fn new<I, S>(keywords: I) -> Query
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Query(keywords.into_iter().map(util::to_lowercase).collect())
    }

    pub fn matches<S: AsRef<str>>(&self, path: S) -> bool {
        let keywords = &self.0;
        let (keywords_last, keywords) = match keywords.split_last() {
            Some(split) => split,
            None => return true,
        };

        let path = util::to_lowercase(path);
        let mut subpath = path.as_str();
        match subpath.rfind(keywords_last) {
            Some(idx) => {
                if subpath[idx + keywords_last.len()..].contains(path::is_separator) {
                    return false;
                }
                subpath = &subpath[..idx];
            }
            None => return false,
        }

        for keyword in keywords.iter().rev() {
            match subpath.rfind(keyword) {
                Some(idx) => subpath = &subpath[..idx],
                None => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

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
            assert_eq!(is_match, Query::new(keywords).matches(path))
        }
    }
}
