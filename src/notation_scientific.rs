use std::borrow::Cow;
use std::str::FromStr;

use crate::error::TonalParseError;
use crate::pitch::{Named, alt_to_acc};
use crate::regexes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScientificPitch {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScientificPitchParts {
    pub step: usize,
    pub alt: i32,
    pub oct: Option<i32>,
}

impl ScientificPitch {
    pub fn parts(&self) -> ScientificPitchParts {
        ScientificPitchParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
        }
    }
}

pub(crate) fn tokenize(s: &str) -> (String, String, String, String) {
    match regexes::NOTE.captures(s) {
        Some(c) => (
            c[1].to_uppercase(),
            c[2].replace("x", "##"),
            c[3].to_string(),
            c[4].to_string(),
        ),
        None => (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ),
    }
}

pub fn parse(note_name: &str) -> Option<ScientificPitch> {
    let (letter, acc, oct_str, rest) = tokenize(note_name);

    if letter.is_empty() || !rest.is_empty() {
        return None;
    }

    let step = ((letter.chars().next().unwrap() as usize) + 3).rem_euclid(7);
    let alt = if acc.starts_with('b') {
        -(acc.len() as i32)
    } else {
        acc.len() as i32
    };

    let oct = if oct_str.is_empty() {
        None
    } else {
        oct_str.parse::<i32>().ok()
    };

    Some(ScientificPitch { step, alt, oct })
}

impl TryFrom<&str> for ScientificPitch {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parse(value) {
            Some(parsed) => Ok(parsed),
            _ => Err(TonalParseError {
                entity: String::from("scientific pitch"),
                input: value.to_string(),
            }),
        }
    }
}

impl FromStr for ScientificPitch {
    type Err = TonalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ScientificPitch::try_from(s)
    }
}

pub fn name(pitch: &ScientificPitch) -> String {
    let ScientificPitch { step, alt, oct } = pitch;
    let Some(letter) = "CDEFGAB".chars().nth(*step) else {
        return String::new();
    };

    let pc = format!("{letter}{}", alt_to_acc(*alt));

    match oct {
        Some(o) => format!("{pc}{o}"),
        None => pc,
    }
}

impl Named for ScientificPitch {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(name(self))
    }
}

impl std::fmt::Display for ScientificPitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(step: usize, alt: i32, oct: Option<i32>) -> ScientificPitch {
        ScientificPitch { step, alt, oct }
    }

    fn toks(s: &str) -> [String; 4] {
        let (a, b, c, d) = tokenize(s);
        [a, b, c, d]
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(toks("Cbb5 major"), ["C", "bb", "5", "major"]);
        assert_eq!(toks("Ax"), ["A", "##", "", ""]);
        assert_eq!(toks("CM"), ["C", "", "", "M"]);
        assert_eq!(toks("maj7"), ["", "", "", "maj7"]);
        assert_eq!(toks(""), ["", "", "", ""]);
        assert_eq!(toks("bb"), ["B", "b", "", ""]);
        assert_eq!(toks("##"), ["", "##", "", ""]);
    }

    #[test]
    fn test_parse() {
        assert_eq!(parse("C3"), Some(sp(0, 0, Some(3))));
        assert_eq!(parse("A4"), Some(sp(5, 0, Some(4))));
        assert_eq!(parse("D#"), Some(sp(1, 1, None)));
        assert_eq!(parse("D##"), Some(sp(1, 2, None)));
        assert_eq!(parse("D###"), Some(sp(1, 3, None)));
        assert_eq!(parse("eb"), Some(sp(2, -1, None)));
        assert_eq!(parse("ebbb"), Some(sp(2, -3, None)));
        assert_eq!(parse("ebbb3"), Some(sp(2, -3, Some(3))));
        assert_eq!(parse("ebbb-3"), Some(sp(2, -3, Some(-3))));
        assert_eq!(parse("ebbbbbb-3"), Some(sp(2, -6, Some(-3))));
        assert_eq!(parse("bb"), parse("Bb"));
    }

    #[test]
    fn test_name() {
        assert_eq!(name(&sp(1, -1, None)), "Db");
        assert_eq!(name(&sp(2, 1, None)), "E#");
        assert_eq!(name(&sp(2, 1, Some(4))), "E#4");
        assert_eq!(name(&sp(5, 0, None)), "A");
        assert_eq!(name(&sp(5, 0, Some(2))), "A2");
        assert_eq!(name(&sp(8, 0, None)), "");
    }
}
