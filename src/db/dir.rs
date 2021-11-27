use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use bincode::Options as _;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DirList<'a>(#[serde(borrow)] pub Vec<Dir<'a>>);

impl DirList<'_> {
    const VERSION: u32 = 3;

    pub fn new() -> DirList<'static> {
        DirList(Vec::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DirList> {
        // Assume a maximum size for the database. This prevents bincode from throwing strange
        // errors when it encounters invalid data.
        const MAX_SIZE: u64 = 32 << 20; // 32 MiB
        let deserializer = &mut bincode::options().with_fixint_encoding().with_limit(MAX_SIZE);

        // Split bytes into sections.
        let version_size = deserializer.serialized_size(&Self::VERSION).unwrap() as _;
        if bytes.len() < version_size {
            bail!("could not deserialize database: corrupted data");
        }
        let (bytes_version, bytes_dirs) = bytes.split_at(version_size);

        // Deserialize sections.
        (|| {
            let version = deserializer.deserialize(bytes_version)?;
            match version {
                Self::VERSION => Ok(deserializer.deserialize(bytes_dirs)?),
                version => {
                    bail!("unsupported version (got {}, supports {})", version, Self::VERSION,)
                }
            }
        })()
        .context("could not deserialize database")
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        (|| -> bincode::Result<_> {
            // Preallocate buffer with combined size of sections.
            let version_size = bincode::serialized_size(&Self::VERSION)?;
            let dirs_size = bincode::serialized_size(&self)?;
            let buffer_size = version_size + dirs_size;
            let mut buffer = Vec::with_capacity(buffer_size as _);

            // Serialize sections into buffer.
            bincode::serialize_into(&mut buffer, &Self::VERSION)?;
            bincode::serialize_into(&mut buffer, &self)?;
            Ok(buffer)
        })()
        .context("could not serialize database")
    }
}

impl<'a> Deref for DirList<'a> {
    type Target = Vec<Dir<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for DirList<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<Vec<Dir<'a>>> for DirList<'a> {
    fn from(dirs: Vec<Dir<'a>>) -> Self {
        DirList(dirs)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dir<'a> {
    #[serde(borrow)]
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir<'_> {
    pub fn score(&self, now: Epoch, keywords: &Vec<String>) -> Score {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

        // The older the entry, the lesser its importance.
        let duration = now.saturating_sub(self.last_accessed);
        let adjusted_rank = if duration < HOUR {
            self.rank * 4.0
        } else if duration < DAY {
            self.rank * 2.0
        } else if duration < WEEK {
            self.rank * 0.5
        } else {
            self.rank * 0.25
        };

        for keyword in keywords {
            debug_assert!(self.path.to_lowercase().contains(&keyword.to_lowercase()));
        }

        let mut kw_score_sum = 0;

        // Split the path into components, then words, so the "M" can be a better match
        // for "folk music" than for "tom", and the best match for "music".
        // And even more so if it's the last path component.
        let path = PathBuf::from_str(&self.path).unwrap(); // safe because error is Infallible
        let path_components = path.components();
        let mut is_last_component = true;
        for component in path_components.rev() {
            let component = component.as_os_str().to_str().unwrap(); // safe because the path came from a string
            let left_word_boundaries = left_word_boundaries(&component);
            for keyword in keywords {
                kw_score_sum += Self::compute_kw_score(&component, keyword, &left_word_boundaries, is_last_component);
            }
            is_last_component = false;
        }

        (kw_score_sum, adjusted_rank)
    }

    pub fn compute_kw_score(
        path_component: &str,
        keyword: &str,
        left_word_boundaries: &Vec<usize>,
        is_last_component: bool,
    ) -> u64 {
        let keyword_lower = &keyword.to_lowercase();
        let path_lower = path_component.to_lowercase();

        // more than one boundary can match
        let mut best_boundary_score = 0;
        for idx in left_word_boundaries {
            // TODO: think carefully about these rules. Should the case of the match
            // be allowed to influence the score? What if it's all lowercase, so
            // a smart case match is impossible?
            let word = &path_component[*idx..];
            let word_lower = &path_lower[*idx..];
            if word.starts_with(keyword) {
                // exact match, but even better if it's at the leftmost position in the component,
                // like "D" matching $HOME/Documents
                let score = if *idx == 0 { 105 } else { 100 };

                // TODO: think about checking the right word boundary, and give extra points if it matches.
                //       Imagine two directories, src_3 and src. If src_3 is more frequently used, "sr" will
                //       match src_3. But "src" will match src.
                best_boundary_score = best_boundary_score.max(score);
            } else if word_lower.starts_with(keyword) {
                // smart case match
                best_boundary_score = best_boundary_score.max(90);
            } else if word_lower.starts_with(keyword_lower) {
                // wrong case but it's a match otherwise
                best_boundary_score = best_boundary_score.max(20);
            } else {
                // No score. We don't need to give any score for a keyword that matches but not on a word boundary--
                // All paths being checked should at least match in that way.
                // But note that though the path will match the keyword, this path component may not match.
            }
        }

        if best_boundary_score > 0 && is_last_component {
            // matches in the last path component should be considered a little better
            best_boundary_score += 5;
        }

        best_boundary_score
    }

    pub fn display(&self) -> DirDisplay {
        DirDisplay { dir: self }
    }

    pub fn display_score(&self, now: Epoch, keywords: Option<&Vec<String>>) -> DirDisplayScore {
        DirDisplayScore { dir: self, now, keywords: keywords.map(|vec| vec.iter().cloned().collect()) }
    }
}

/// Returns byte indices that correspond to the leftmost position of each word.
/// For input "hi there", the result will contain 0 and 3.
///
/// The result may also contain extraneous indices.
fn left_word_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = Vec::new();

    #[derive(PartialEq, Clone, Copy, PartialOrd)]
    enum Case {
        None,
        LowerCase,
        UpperCase,
    }

    // We won't need the words themselves because we want to do multi-word match.
    // We need the whole string for that.
    for (word_idx, word) in text.unicode_word_indices() {
        boundaries.push(word_idx);

        // Also search for case changes, and non-text characters:
        // MyDocuments
        // my_documents
        // TODO: should "clap3b4" count as 4 words or 1?
        let mut prev_case = None;
        for (grapheme_idx, grapheme) in word.grapheme_indices(true) {
            let lower = grapheme.to_lowercase();
            let upper = grapheme.to_uppercase();
            let case = if lower == grapheme && upper == grapheme {
                Case::None
            } else if lower == grapheme {
                Case::LowerCase
            } else {
                // Assume the other cases are upper case, because there might be more than
                // one way to represent upper case
                Case::UpperCase
            };

            if let Some(prev_case) = &prev_case {
                if case > *prev_case {
                    // Consider this a word start if going from no case to any case,
                    // or lower case to upper case.
                    boundaries.push(word_idx + grapheme_idx);
                }
            }
            let _ = prev_case.replace(case);
        }
    }

    boundaries
}

pub struct DirDisplay<'a> {
    dir: &'a Dir<'a>,
}

impl Display for DirDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dir.path)
    }
}

pub struct DirDisplayScore<'a> {
    dir: &'a Dir<'a>,
    now: Epoch,
    keywords: Option<Vec<String>>,
}

impl Display for DirDisplayScore<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let no_keywords = Vec::default();
        let keywords = self.keywords.as_ref().unwrap_or(&no_keywords);

        let (kw_score, score) = self.dir.score(self.now, keywords);
        let score = if score > 9999.0 {
            9999
        } else if score > 0.0 {
            score as u32
        } else {
            0
        };
        write!(f, "{:>4},{:>4} {}", kw_score, score, self.dir.path)
    }
}

pub type Rank = f64;
pub type Score = (u64, Rank);
pub type Epoch = u64;

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::HashSet};

    use super::{left_word_boundaries, Dir, DirList};

    #[test]
    fn zero_copy() {
        let dirs = DirList(vec![Dir { path: "/".into(), rank: 0.0, last_accessed: 0 }]);

        let bytes = dirs.to_bytes().unwrap();
        let dirs = DirList::from_bytes(&bytes).unwrap();

        for dir in dirs.iter() {
            assert!(matches!(dir.path, Cow::Borrowed(_)))
        }
    }

    #[test]
    fn test_left_word_boundaries() {
        assert!(left_word_boundaries("") == vec![]);
        assert!(left_word_boundaries("Hi") == vec![0]);

        assert!(vec![0, 3]
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&left_word_boundaries("hi there").into_iter().collect()));
        assert!(vec![0, 3]
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&left_word_boundaries("hi_there").into_iter().collect()));

        assert!(vec![0, 4] == left_word_boundaries("FürElise"));
        assert!(vec![0, 1] == left_word_boundaries("uTorrent"));
        assert!(vec![0, 2] == left_word_boundaries("µTorrent"));

        assert!(vec![1, 6, 11]
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&left_word_boundaries("/path/file.ext").into_iter().collect()));
        assert!(vec![0, 3, 8, 13]
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&left_word_boundaries(r"C:\path\file.ext").into_iter().collect()));
    }
}
