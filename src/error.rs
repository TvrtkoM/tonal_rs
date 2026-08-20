//! Error types for the crate.
//!
//! The only error type is [`TonalParseError`], returned by the fallible
//! `TryFrom`/`FromStr` conversions on parsed types (notes, intervals, chords,
//! …). The plain `get`/parse functions return `Option` instead.

/// The error returned when a string cannot be parsed into a musical entity.
///
/// For example, `Note::try_from("x")` yields a `TonalParseError` with
/// `entity: "note"` and `input: "x"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TonalParseError {
    /// The kind of entity that failed to parse, e.g. `"note"` or `"interval"`.
    pub entity: String,
    /// The input string that could not be parsed.
    pub input: String,
}

impl std::fmt::Display for TonalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid {}: {}", self.entity, self.input)
    }
}

impl std::error::Error for TonalParseError {}
