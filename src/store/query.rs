use std::path::Path;

pub struct Query(Vec<String>);

impl Query {
    pub fn new<I, S>(keywords: I) -> Query
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Query(
            keywords
                .into_iter()
                .map(|s: S| s.as_ref().to_lowercase())
                .collect(),
        )
    }

    pub fn keywords(&self) -> &[String] {
        &self.0
    }

    pub fn matches<S: AsRef<str>>(&self, path: S) -> bool {
        let path = path.as_ref().to_lowercase();
        let keywords = self.keywords();

        let get_filenames = || {
            let query_name = Path::new(keywords.last()?).file_name()?.to_str().unwrap();
            let dir_name = Path::new(&path).file_name()?.to_str().unwrap();
            Some((query_name, dir_name))
        };

        if let Some((query_name, dir_name)) = get_filenames() {
            if !dir_name.contains(query_name) {
                return false;
            }
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
