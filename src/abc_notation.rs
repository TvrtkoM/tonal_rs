//! Convert between ABC notation and scientific notation.
//!
//! ABC notation writes notes like `C`, `c`, `^C`, `_B,`. This module converts
//! to and from scientific notation ([`abc_to_scientific_notation`],
//! [`scientific_to_abc_notation`]) and offers [`transpose`] and [`distance`]
//! that work directly on ABC note strings.

use crate::pitch_distance::distance as dist;
use crate::pitch_distance::transpose as tr;
use crate::pitch_note::IntoNote;
use crate::pitch_note::Note;
use crate::regexes;

type AbcTokens = (String, String, String);

/// Split an ABC note into `(accidental, letter, octave)`.
///
/// ```rust
/// use tonal_rs::abc_notation;
/// let (acc, letter, oct) = abc_notation::tokenize("^C,");
/// assert_eq!(acc, "^");
/// assert_eq!(letter, "C");
/// assert_eq!(oct, ",");
/// ```
pub fn tokenize(str: &str) -> AbcTokens {
    match regexes::ABC.captures(str) {
        Some(c) => (c[1].to_string(), c[2].to_string(), c[3].to_string()),
        None => (String::new(), String::new(), String::new()),
    }
}

/// Convert an ABC note to scientific notation.
///
/// Returns an empty string for an invalid ABC note.
///
/// ```rust
/// use tonal_rs::abc_notation;
/// assert_eq!(abc_notation::abc_to_scientific_notation("C"), "C4");
/// assert_eq!(abc_notation::abc_to_scientific_notation("^c"), "C#5");
/// assert_eq!(abc_notation::abc_to_scientific_notation("nope"), "");
/// ```
pub fn abc_to_scientific_notation(s: &str) -> String {
    let (acc, letter, oct) = tokenize(s);
    let Some(first) = letter.chars().next() else {
        return String::new();
    };

    let o = 4 + oct
        .chars()
        .map(|c| if c == ',' { -1 } else { 1 })
        .sum::<i32>();

    let a = if acc.starts_with('_') {
        acc.replace('_', "b")
    } else if acc.starts_with('^') {
        acc.replace('^', "#")
    } else {
        String::new()
    };

    if first.is_ascii_lowercase() {
        return format!("{}{}{}", first.to_ascii_uppercase(), a, o + 1);
    }
    format!("{}{}{}", first, a, o)
}

/// Convert a scientific-notation note to ABC notation.
///
/// Returns an empty string if the note has no octave or is invalid.
///
/// ```rust
/// use tonal_rs::abc_notation;
/// assert_eq!(abc_notation::scientific_to_abc_notation("C#5"), "^c");
/// assert_eq!(abc_notation::scientific_to_abc_notation("C"), "");
/// ```
pub fn scientific_to_abc_notation(s: &str) -> String {
    let note = s.into_note();
    let Some(note) = note else {
        return String::new();
    };
    let Some(oct) = note.oct else {
        return String::new();
    };

    let Note { letter, acc, .. } = note;

    let a = if acc.starts_with('b') {
        acc.replace('b', "_")
    } else {
        acc.replace('#', "^")
    };

    let l = if oct > 4 {
        letter.to_ascii_lowercase()
    } else {
        letter
    };

    let o = if oct == 5 {
        String::new()
    } else if oct > 4 {
        "'".repeat((oct - 5) as usize)
    } else {
        ",".repeat((4 - oct) as usize)
    };

    format!("{}{}{}", a, l, o)
}

/// Transpose an ABC note by an interval, returning an ABC note.
///
/// Returns an empty string if the note or interval is invalid.
///
/// ```rust
/// use tonal_rs::abc_notation;
/// assert_eq!(abc_notation::transpose("C", "3M"), "E");
/// ```
pub fn transpose(note: &str, interval: &str) -> String {
    scientific_to_abc_notation(&tr(abc_to_scientific_notation(note).as_str(), interval))
}

/// Get the interval between two ABC notes.
///
/// Returns an empty string if either note is invalid.
///
/// ```rust
/// use tonal_rs::abc_notation;
/// assert_eq!(abc_notation::distance("C", "G"), "5P");
/// ```
pub fn distance(from: &str, to: &str) -> String {
    dist(
        abc_to_scientific_notation(from).as_str(),
        abc_to_scientific_notation(to).as_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build an AbcTokens tuple from &str so the tokenize asserts read like the TS.
    fn tok(a: &str, b: &str, c: &str) -> AbcTokens {
        (a.into(), b.into(), c.into())
    }

    fn to_scientific(abc: &[&str]) -> Vec<String> {
        abc.iter().map(|&s| abc_to_scientific_notation(s)).collect()
    }

    fn to_abc(scientific: &[&str]) -> Vec<String> {
        scientific
            .iter()
            .map(|&s| scientific_to_abc_notation(s))
            .collect()
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("C,',"), tok("", "C", ",',"));
        assert_eq!(tokenize("g,,'"), tok("", "g", ",,'"));
        assert_eq!(tokenize(""), tok("", "", ""));
        assert_eq!(tokenize("m"), tok("", "", ""));
        assert_eq!(tokenize("c#"), tok("", "", ""));
    }

    #[test]
    fn test_transpose() {
        assert_eq!(transpose("=C", "P19"), "g'");
    }

    #[test]
    fn test_distance() {
        assert_eq!(distance("=C", "g"), "12P");
    }

    #[test]
    fn test_to_note() {
        let abc = [
            "__A,,", "_B,", "=C", "d", "^e'", "^^f''", "G,,''", "g,,,'''", "",
        ];
        assert_eq!(
            to_scientific(&abc),
            ["Abb2", "Bb3", "C4", "D5", "E#6", "F##7", "G4", "G5", ""]
        );
    }

    #[test]
    fn test_to_abc() {
        let scientific = [
            "Abb2", "Bb3", "C4", "D5", "E#6", "F##7", "G#2", "Gb7", "",
        ];
        assert_eq!(
            to_abc(&scientific),
            ["__A,,", "_B,", "C", "d", "^e'", "^^f''", "^G,,", "_g''", ""]
        );
    }

    #[test]
    fn test_to_abc_octave_0() {
        let scientific = ["A0", "Bb0", "C0", "D0", "E#0", "F##0", "G#0"];
        assert_eq!(
            to_abc(&scientific),
            ["A,,,,", "_B,,,,", "C,,,,", "D,,,,", "^E,,,,", "^^F,,,,", "^G,,,,"]
        );
    }
}
