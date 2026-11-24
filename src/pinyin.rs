use std::collections::HashSet;

use pinyin::{Pinyin as PinyinReading, ToPinyin, ToPinyinMulti};

use crate::romanize::Romanizer;

/// Pinyin romanizer for Chinese (CJK Unified Ideographs) directory names.
pub struct Pinyin;

/// Upper bound on the number of transliteration variants generated for a
/// single path. Polyphonic characters can multiply the variant count; once
/// this limit would be exceeded, only the default reading of each remaining
/// character is used, keeping the search tractable.
const MAX_VARIANTS: usize = 64;

fn contains_cjk(s: &str) -> bool {
    s.chars().any(is_cjk)
}

// Covers the CJK Unified Ideographs and their Extension A, plus the CJK
// Compatibility Ideographs. Extension B and later (U+20000+, rare historical
// characters) are not covered, which has no practical impact for directory
// names.
fn is_cjk(c: char) -> bool {
    matches!(
        u32::from(c),
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

/// All distinct readings of a Chinese character, using the order provided by
/// the crate (the first reading is the character's default reading). Empty for
/// characters with no pinyin (e.g. separators, ASCII, compat ideographs).
fn readings(ch: char) -> Vec<&'static str> {
    match ch.to_pinyin_multi() {
        Some(multi) => {
            let mut seen = HashSet::new();
            let mut out = Vec::new();
            for reading in multi.into_iter().map(PinyinReading::plain) {
                if seen.insert(reading) {
                    out.push(reading);
                }
            }
            out
        }
        None => ch.to_pinyin().map_or_else(Vec::new, |p| vec![p.plain()]),
    }
}

/// Generates every consistent pinyin transliteration of `s`. Each Chinese
/// character is expanded to all its possible readings, while non-Chinese
/// characters (including path separators) are preserved verbatim. The first
/// variant uses the default reading of each character.
///
/// To keep the search tractable, at most [`MAX_VARIANTS`] variants are
/// generated; if this limit would be exceeded, only the default reading of
/// each character is used.
///
/// Returns `None` if `s` contains no Chinese characters.
fn variants(s: &str) -> Option<Vec<String>> {
    if !contains_cjk(s) {
        return None;
    }

    let mut out: Vec<String> = vec![String::new()];
    let mut primary_only = false;

    for ch in s.chars() {
        let mut readings = readings(ch).into_iter();

        // Some characters (separators, ASCII, unknown ideographs) have no
        // reading and are preserved as-is.
        let Some(first) = readings.next() else {
            for variant in &mut out {
                variant.push(ch);
            }
            continue;
        };

        // `readings` had its first item consumed into `first`, so the total
        // number of readings for this character is `readings.len() + 1`. Fall
        // back to the default reading if there would be too many variants, so
        // that pathological inputs stay O(n).
        if primary_only || out.len().saturating_mul(readings.len() + 1) > MAX_VARIANTS {
            primary_only = true;
            for variant in &mut out {
                variant.push_str(first);
            }
            continue;
        }

        let rest: Vec<&'static str> = readings.collect();
        let mut next = Vec::with_capacity(out.len() * (rest.len() + 1));
        for base in &out {
            let mut variant = base.clone();
            variant.push_str(first);
            next.push(variant);
            for reading in &rest {
                let mut variant = base.clone();
                variant.push_str(reading);
                next.push(variant);
            }
        }
        out = next;
    }

    Some(out)
}

impl Romanizer for Pinyin {
    fn variants(&self, s: &str) -> Option<Vec<String>> {
        variants(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cjk() {
        assert!(contains_cjk("市场"));
        assert!(contains_cjk("/foo/市场"));
        assert!(contains_cjk("\u{3400}"));
        assert!(contains_cjk("\u{4E00}"));
        assert!(contains_cjk("\u{9FFF}"));
        assert!(contains_cjk("\u{F900}"));
        assert!(!contains_cjk("/foo/bar"));
        assert!(!contains_cjk(""));
    }

    #[test]
    fn no_variants_without_cjk() {
        assert_eq!(variants("/foo/bar"), None);
        assert_eq!(variants(""), None);
    }

    #[test]
    fn primary_reading() {
        // The first variant uses the default reading of each character.
        assert_eq!(variants("市场").unwrap().first().unwrap(), "shichang");
        assert_eq!(variants("/foo/市场").unwrap().first().unwrap(), "/foo/shichang");
    }

    #[test]
    fn preserves_separators() {
        let variants = variants("/foo/市场/资料").unwrap();
        assert!(variants.iter().all(|v| v.contains('/')));
        assert!(variants.iter().any(|v| v == "/foo/shichang/ziliao"));
    }

    #[test]
    fn polyphone_variants() {
        // 时长 (duration) is read shícháng; the default reading of 长 is zhǎng, so
        // generating all readings is required to match "shichang".
        let variants = variants("时长").unwrap();
        assert!(variants.contains(&"shichang".to_owned()));
        assert!(variants.contains(&"shizhang".to_owned()));
    }

    #[test]
    fn mixed_content() {
        let variants = variants("/foo/mixed市场dir").unwrap();
        assert!(variants.iter().any(|v| v == "/foo/mixedshichangdir"));
    }

    #[test]
    fn consecutive_polyphones() {
        // 行长: both characters are polyphonic.
        // 行: hang/xing, 长: zhang/chang
        // All 4 combinations should be generated.
        let v = variants("行长").unwrap();
        assert!(v.contains(&"hangzhang".to_owned()), "missing hangzhang in {v:?}");
        assert!(v.contains(&"hangchang".to_owned()), "missing hangchang in {v:?}");
        assert!(v.contains(&"xingzhang".to_owned()), "missing xingzhang in {v:?}");
        assert!(v.contains(&"xingchang".to_owned()), "missing xingchang in {v:?}");
    }
}
