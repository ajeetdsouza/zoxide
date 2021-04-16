use std::path::Path;

pub struct Query(Vec<String>);

impl Query {
    pub fn new<I, S>(keywords: I) -> Query
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Query(keywords.into_iter().map(|s: S| to_lowercase(s)).collect())
    }

    pub fn matches<S: AsRef<str>>(&self, path: S) -> bool {
        let keywords = &self.0;
        let keywords_last = match keywords.last() {
            Some(keyword) => keyword,
            None => return true,
        };

        let path = to_lowercase(path);

        let query_name = get_filename(keywords_last);
        let dir_name = get_filename(&path);
        if !dir_name.contains(query_name) {
            return false;
        }

        let mut subpath = path.as_str();
        for keyword in keywords.iter() {
            match subpath.find(keyword) {
                Some(idx) => subpath = &subpath[idx + keyword.len()..],
                None => return false,
            }
        }

        true
    }
}

fn get_filename(mut path: &str) -> &str {
    if cfg!(windows) {
        Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    } else {
        if path.ends_with('/') {
            path = &path[..path.len() - 1];
        }
        match path.rfind('/') {
            Some(idx) => &path[idx + 1..],
            None => path,
        }
    }
}

fn to_lowercase<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    if s.is_ascii() {
        s.to_ascii_lowercase()
    } else {
        s.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn query_normalization() {
        assert!(Query::new(&["fOo", "bAr"]).matches("/foo/bar"));
    }

    #[test]
    fn query_filename() {
        assert!(Query::new(&["ba"]).matches("/foo/bar"));
    }

    #[test]
    fn query_not_filename() {
        assert!(!Query::new(&["fo"]).matches("/foo/bar"));
    }

    #[test]
    fn query_not_filename_slash() {
        assert!(!Query::new(&["foo/"]).matches("/foo/bar"));
    }

    #[test]
    fn query_path_separator() {
        assert!(Query::new(&["/", "fo", "/", "ar"]).matches("/foo/bar"));
    }

    #[test]
    fn query_path_separator_between() {
        assert!(Query::new(&["oo/ba"]).matches("/foo/bar"));
    }

    #[test]
    fn query_overlap_text() {
        assert!(!Query::new(&["foo", "o", "bar"]).matches("/foo/bar"));
    }

    #[test]
    fn query_overlap_slash() {
        assert!(!Query::new(&["/foo/", "/bar"]).matches("/foo/bar"));
    }

    #[test]
    fn query_consecutive_slash() {
        assert!(Query::new(&["/foo/", "/baz"]).matches("/foo/bar/baz"));
    }
}
