use std::sync::LazyLock;

use regex::Regex;

use crate::pitch_distance::distance as dist;
use crate::pitch_distance::transpose as tr;
use crate::pitch_note::AsNote;

static REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(_+|=|\^+|)([a-gA-G])([,']*)$").unwrap());

type AbcTokens = (String, String, String);

pub fn tokenize(str: &str) -> AbcTokens {
    match REGEX.captures(str) {
        Some(c) => (c[1].to_string(), c[2].to_string(), c[3].to_string()),
        None => (String::new(), String::new(), String::new()),
    }
}

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

pub fn scientific_to_abc_notation(s: &str) -> String {
    let note = s.note();
    let Some(note) = note else {
        return String::new();
    };
    let Some(oct) = note.oct else {
        return String::new();
    };

    let letter = note.letter;
    let acc = &note.acc;

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

pub fn transpose(note: &str, interval: &str) -> String {
    scientific_to_abc_notation(&tr(abc_to_scientific_notation(note).as_str(), interval))
}

pub fn distance(from: &str, to: &str) -> &'static str {
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
        let scientific = ["Abb2", "Bb3", "C4", "D5", "E#6", "F##7", "G#2", "Gb7", ""];
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
            [
                "A,,,,", "_B,,,,", "C,,,,", "D,,,,", "^E,,,,", "^^F,,,,", "^G,,,,"
            ]
        );
    }
}
