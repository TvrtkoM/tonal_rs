//! Shared, lazily-compiled regexes.
use std::sync::LazyLock;

use regex::Regex;

/// Tokenizes a note name into `(letter, accidental, octave, rest)`.
///
/// Shared by [`crate::pitch_note`] and [`crate::notation_scientific`].
pub(crate) static NOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-gA-G]?)(#+|b+|x+|)(-?\d*)\s*(.*)$").unwrap());

/// Matches a bare accidental string (only flats or only sharps).
pub(crate) static ACCIDENTAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(b+|#+)$").unwrap());

/// Splits a duration shorthand into `(base, dots)`.
pub(crate) static DURATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^.]+)(\.*)$").unwrap());

/// Matches a time signature `numerator/denominator`, allowing additive
/// numerators (e.g. `3+2/8`).
pub(crate) static TIME_SIGNATURE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(?:\+\d)*)/(\d+)$").unwrap());

/// Tokenizes an interval in either tonal (number then quality) or shorthand
/// (quality then number) notation.
///
/// - group 1-2: tonal notation (number then quality)
/// - group 3-4: shorthand notation (quality then number)
pub(crate) static INTERVAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:([-+]?\d+)(d{1,4}|m|M|P|A{1,4})|(AA|A|P|M|m|d|dd)([-+]?\d+))$").unwrap()
});

/// Tokenizes an ABC note into `(accidental, letter, octave)`.
pub(crate) static ABC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(_+|=|\^+|)([a-gA-G])([,']*)$").unwrap());

/// Tokenizes a roman numeral into `(accidental, numeral, rest)`.
pub(crate) static ROMAN_NUMERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(#+|b+|x+|)(IV|I{1,3}|VI{0,2}|iv|i{1,3}|vi{0,2})([^IViv]*)$").unwrap()
});

/// Matches a 12-bit chroma string.
pub(crate) static CHROMA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[01]{12}$").unwrap());
