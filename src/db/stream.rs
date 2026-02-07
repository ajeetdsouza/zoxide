use std::iter::Rev;
use std::ops::Range;
use std::path::Path;
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

    pub fn next(&mut self) -> Option<&Dir<'_>> {
        while let Some(idx) = self.idxs.next() {
            let dir = &self.db.dirs()[idx];

            if !self.filter_by_keywords(&dir.path) {
                continue;
            }

            if !self.filter_by_base_dir(&dir.path) {
                continue;
            }

            if !self.filter_by_exclude(&dir.path) {
                self.db.swap_remove(idx);
                continue;
            }

            // Exists queries are slow, this should always be checked last.
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

    fn filter_by_base_dir(&self, path: &str) -> bool {
        match &self.options.base_dir {
            Some(base_dir) => Path::new(path).starts_with(base_dir),
            None => true,
        }
    }

    fn filter_by_exclude(&self, path: &str) -> bool {
        !self.options.exclude.iter().any(|pattern| pattern.matches(path))
    }

    fn filter_by_exists(&self, path: &str) -> bool {
        if !self.options.exists {
            return true;
        }

        // The logic here is reversed - if we resolve symlinks when adding entries to
        // the database, we should not return symlinks when querying from
        // the database.
        let resolver =
            if self.options.resolve_symlinks { fs::symlink_metadata } else { fs::metadata };
        resolver(path).map(|metadata| metadata.is_dir()).unwrap_or_default()
    }

    fn filter_by_keywords(&self, path: &str) -> bool {
        let (keywords_last, keywords) = match self.options.keywords.split_last() {
            Some(split) => split,
            None => return true,
        };

        let path = util::to_lowercase(path);
        let mut path = path.as_str();

        let (idx, _end) = match rfind_component_match(path, keywords_last) {
            Some((idx, end)) => {
                if path[end..].contains(path::is_separator) {
                    return false;
                }
                (idx, end)
            }
            None => return false,
        };
        path = &path[..idx];

        for keyword in keywords.iter().rev() {
            match rfind_component_match(path, keyword) {
                Some((idx, _)) => path = &path[..idx],
                None => return false,
            }
        }

        true
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

    /// Only return directories within this parent directory
    /// Does not check if the path exists
    base_dir: Option<String>,
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
            base_dir: None,
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

    pub fn with_base_dir(mut self, base_dir: Option<String>) -> Self {
        self.base_dir = base_dir;
        self
    }
}

fn rfind_component_match(path: &str, keyword: &str) -> Option<(usize, usize)> {
    if keyword.is_empty() {
        return None;
    }

    if keyword.contains(path::is_separator) {
        return path.rfind(keyword).map(|idx| (idx, idx + keyword.len()));
    }

    if let Some(idx) = path.rfind(keyword) {
        return Some((idx, idx + keyword.len()));
    }

    let keyword_len = keyword.chars().count();

    // Fuzzy: rightmost component where keyword is a subsequence, or edit
    // distance 1 (equal-length typo) within a single component.
    for (component_start, component) in rsplit_components_with_indices(path) {
        if let Some((start, end)) = subsequence_bounds(component, keyword) {
            return Some((component_start + start, component_start + end));
        }

        if keyword_len == component.chars().count() && edit_distance_leq1(component, keyword) {
            return Some((component_start, component_start + component.len()));
        }
    }

    None
}

fn rsplit_components_with_indices(path: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut components = Vec::new();
    let mut end = path.len();

    for (idx, ch) in path.char_indices().rev() {
        if path::is_separator(ch) {
            if idx + ch.len_utf8() < end {
                components.push((idx + ch.len_utf8(), &path[idx + ch.len_utf8()..end]));
            }
            end = idx;
        }
    }

    if end > 0 {
        components.push((0, &path[..end]));
    }

    components.into_iter()
}

fn subsequence_bounds(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    if needle.is_empty() {
        return None;
    }

    let mut start = None;
    let mut needle_chars = needle.chars();
    let mut next_needed = needle_chars.next()?;

    for (idx, ch) in haystack.char_indices() {
        if ch == next_needed {
            start.get_or_insert(idx);
            if let Some(n) = needle_chars.next() {
                next_needed = n;
            } else {
                return Some((start.unwrap(), idx + ch.len_utf8()));
            }
        }
    }

    None
}

fn edit_distance_leq1(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }

    let a_chars: Vec<_> = a.chars().collect();
    let b_chars: Vec<_> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if (a_len as isize - b_len as isize).abs() > 1 {
        return false;
    }

    if a_len == b_len {
        // Single substitution?
        let mut diffs = 0;
        for (ac, bc) in a_chars.iter().zip(b_chars.iter()) {
            if ac != bc {
                diffs += 1;
                if diffs > 1 {
                    break;
                }
            }
        }
        if diffs == 1 {
            return true;
        }

        // Single adjacent transposition?
        for i in 0..a_len - 1 {
            if a_chars[i] != b_chars[i] {
                return i + 1 < a_len
                    && a_chars[i] == b_chars[i + 1]
                    && a_chars[i + 1] == b_chars[i]
                    && a_chars[i + 2..] == b_chars[i + 2..]
                    && a_chars[..i] == b_chars[..i];
            }
        }

        return false;
    }

    // Lengths differ by exactly 1: check single insertion/deletion.
    let (short, long) = if a_len < b_len { (&a_chars, &b_chars) } else { (&b_chars, &a_chars) };
    let mut i = 0;
    let mut j = 0;
    let mut edits = 0;

    while i < short.len() && j < long.len() {
        if short[i] == long[j] {
            i += 1;
            j += 1;
        } else {
            edits += 1;
            if edits > 1 {
                return false;
            }
            j += 1; // skip one char in longer string
        }
    }

    true
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
    // Fuzzy subsequence within component
    #[case(&["docs"], "/home/Documents", true)]
    #[case(&["dcmts"], "/home/Documents", true)]
    // Typo tolerance (edit distance 1)
    #[case(&["doucments"], "/home/Documents", true)]
    fn query(#[case] keywords: &[&str], #[case] path: &str, #[case] is_match: bool) {
        let db = &mut Database::new(PathBuf::new(), Vec::new(), |_| Vec::new(), false);
        let options = StreamOptions::new(0).with_keywords(keywords.iter());
        let stream = Stream::new(db, options);
        assert_eq!(is_match, stream.filter_by_keywords(path));
    }
}
