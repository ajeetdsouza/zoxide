//! Pluggable transliteration support for keyword matching.
//!
//! A [`Romanizer`] converts non-ASCII path components into an ASCII
//! romanization, so that ASCII keywords can match native-script directory
//! names (e.g. `z shichang` matching `市场`). Each romanizer is an optional
//! Cargo feature so the default build stays lean; downstream packagers opt
//! into the transliterations relevant to their locale.
//!
//! To add a new transliteration:
//! 1. Add an optional dependency and matching feature in `Cargo.toml`.
//! 2. Implement [`Romanizer`] in a new module, gated on the feature.
//! 3. Register an instance in [`romanizers`] below.

/// Converts a path into one or more ASCII romanizations, enabling ASCII
/// keywords to match non-ASCII directory names.
pub trait Romanizer: Send + Sync {
    /// Returns every distinct romanization of `s`, or `None` if `s` contains
    /// no characters this romanizer recognizes. Non-ASCII characters with no
    /// reading are preserved verbatim; path separators are always preserved.
    /// The first returned variant should be the most likely reading.
    fn variants(&self, s: &str) -> Option<Vec<String>>;
}

/// All romanizers enabled by the active Cargo features, in priority order.
/// Returns an empty slice when no transliteration features are compiled in,
/// so keyword matching is a no-op for the default build.
pub fn romanizers() -> &'static [&'static dyn Romanizer] {
    ROMANIZERS
}

#[cfg(feature = "pinyin")]
static ROMANIZERS: &[&dyn Romanizer] = &[&crate::pinyin::Pinyin];

#[cfg(not(feature = "pinyin"))]
static ROMANIZERS: &[&dyn Romanizer] = &[];
