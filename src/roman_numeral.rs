//! Parse roman numeral chord symbols.
//!
//! A roman numeral such as `"bIImaj7"` describes a scale degree with an
//! accidental and a chord type. Use [`get`] to parse one into a
//! [`RomanNumeral`], reading its `interval`, `roman` and `chord_type`; a roman
//! numeral converts to an interval, so it can be transposed against a tonic.

use std::{borrow::Cow, str::FromStr};

use crate::{
    error::TonalParseError,
    pitch::{Direction, Named, Pitch, alt_to_acc},
    pitch_interval::{Interval, IntoInterval},
    pitch_note::acc_to_alt,
    regexes,
};

/// A parsed roman numeral with its degree, accidental and chord type.
///
/// Fields are private; read them through [`RomanNumeral::parts`].
#[derive(Clone, Debug, PartialEq)]
pub struct RomanNumeral {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: i32,
    pub(crate) dir: Direction,
    pub(crate) name: String,
    pub(crate) roman: String,
    pub(crate) interval: String,
    pub(crate) acc: String,
    pub(crate) chord_type: String,
    pub(crate) major: bool,
}

/// A borrowing, fully public view of a [`RomanNumeral`]'s fields, returned by
/// [`RomanNumeral::parts`].
#[derive(Clone, Copy, Debug)]
pub struct RomanNumeralParts<'a> {
    /// Diatonic step, `0` (I) to `6` (VII).
    pub step: usize,
    /// Alteration from the accidental.
    pub alt: i32,
    /// Octave (always `0`).
    pub oct: i32,
    /// Direction (always ascending).
    pub dir: Direction,
    /// The full name, e.g. `"#VIIb5"`.
    pub name: &'a str,
    /// The roman part, uppercased, e.g. `"VII"`.
    pub roman: &'a str,
    /// The interval this degree represents, e.g. `"7A"`.
    pub interval: &'a str,
    /// The accidental, e.g. `"#"`.
    pub acc: &'a str,
    /// The trailing chord type, e.g. `"b5"`.
    pub chord_type: &'a str,
    /// Whether the numeral is uppercase (major).
    pub major: bool,
}

impl RomanNumeral {
    /// A borrowing view of all fields.
    pub fn parts(&self) -> RomanNumeralParts<'_> {
        RomanNumeralParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
            dir: self.dir,
            name: &self.name,
            roman: &self.roman,
            interval: &self.interval,
            acc: &self.acc,
            chord_type: &self.chord_type,
            major: self.major,
        }
    }
}

impl Named for RomanNumeral {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for RomanNumeral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

static NAMES: &[&str] = &["I", "II", "III", "IV", "V", "VI", "VII"];
static NAMES_MINOR: &[&str] = &["i", "ii", "iii", "iv", "v", "vi", "vii"];

/// The seven roman numeral names, uppercase (`major = true`) or lowercase.
///
/// ```rust
/// use tonal_rs::roman_numeral;
/// assert_eq!(roman_numeral::names(true)[0], "I");
/// assert_eq!(roman_numeral::names(false)[0], "i");
/// ```
pub fn names(major: bool) -> &'static [&'static str] {
    if major { NAMES } else { NAMES_MINOR }
}

/// Split a roman numeral string into `(name, accidental, roman, chord_type)`.
///
/// ```rust
/// use tonal_rs::roman_numeral;
/// let (_name, acc, roman, chord_type) = roman_numeral::tokenize("#VIIb5");
/// assert_eq!(acc, "#");
/// assert_eq!(roman, "VII");
/// assert_eq!(chord_type, "b5");
/// ```
pub fn tokenize(s: &str) -> (String, String, String, String) {
    let [a, b, c, d] = regexes::ROMAN_NUMERAL
        .captures(s)
        .map_or(["", "", "", ""], |c| {
            [0, 1, 2, 3].map(|i| c.get(i).map_or("", |m| m.as_str()))
        });
    (a.into(), b.into(), c.into(), d.into())
}

fn parse(src: &str) -> Option<RomanNumeral> {
    let (name, acc, roman, chord_type) = tokenize(src);
    if roman.is_empty() {
        return None;
    }

    let upper_roman = roman.to_uppercase();
    let major = upper_roman == roman;
    let step = NAMES.iter().position(|&n| n == upper_roman)?;
    let alt = acc_to_alt(&acc);

    let p = Pitch {
        step,
        alt,
        oct: Some(0),
        dir: Some(Direction::Up),
    };

    let interval = (&p).into_interval()?;

    Some(RomanNumeral {
        step,
        alt,
        oct: 0,
        dir: Direction::Up,
        name,
        major,
        roman,
        interval: interval.name,
        acc,
        chord_type,
    })
}

fn from_pitch(pitch: &Pitch) -> Option<RomanNumeral> {
    let src = format!("{}{}", alt_to_acc(pitch.alt), NAMES[pitch.step]);
    parse(&src)
}

fn from_size(n: usize) -> Option<RomanNumeral> {
    let src = *NAMES.get(n)?;
    parse(src)
}

/// Conversion into a [`RomanNumeral`], implemented for a name (`&str`), a
/// degree index (`usize`), and [`Pitch`]/[`Interval`] references.
pub trait IntoRomanNumeral {
    /// Parse or convert `self` into a [`RomanNumeral`].
    fn into_roman_numeral(self) -> Option<RomanNumeral>;
}

impl IntoRomanNumeral for &str {
    fn into_roman_numeral(self) -> Option<RomanNumeral> {
        parse(self)
    }
}

impl IntoRomanNumeral for usize {
    fn into_roman_numeral(self) -> Option<RomanNumeral> {
        from_size(self)
    }
}

impl IntoRomanNumeral for &Pitch {
    fn into_roman_numeral(self) -> Option<RomanNumeral> {
        from_pitch(self)
    }
}

impl IntoRomanNumeral for &Interval {
    fn into_roman_numeral(self) -> Option<RomanNumeral> {
        let p = Pitch {
            step: self.step,
            alt: self.alt,
            oct: Some(self.oct),
            dir: Some(self.dir),
        };
        from_pitch(&p)
    }
}

impl TryFrom<&str> for RomanNumeral {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.into_roman_numeral() {
            Some(parsed) => Ok(parsed),
            _ => Err(TonalParseError {
                entity: String::from("roman numeral"),
                input: value.to_string(),
            }),
        }
    }
}

impl FromStr for RomanNumeral {
    type Err = TonalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RomanNumeral::try_from(s)
    }
}

/// Get a [`RomanNumeral`] from a name, degree index, pitch or interval.
///
/// Returns `None` for an invalid roman numeral.
///
/// ```rust
/// use tonal_rs::roman_numeral;
/// assert_eq!(roman_numeral::get("bIImaj7").unwrap().parts().interval, "2m");
/// assert_eq!(roman_numeral::get(0usize).unwrap().parts().name, "I");
/// assert!(roman_numeral::get("nope").is_none());
/// ```
pub fn get<T: IntoRomanNumeral>(src: T) -> Option<RomanNumeral> {
    src.into_roman_numeral()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch_interval::interval;

    fn split(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    #[test]
    fn test_names() {
        assert_eq!(
            names(true).to_vec(),
            ["I", "II", "III", "IV", "V", "VI", "VII"]
        );
        assert_eq!(
            names(false).to_vec(),
            ["i", "ii", "iii", "iv", "v", "vi", "vii"]
        );
    }

    #[test]
    fn test_properties() {
        let r = get("#VIIb5").unwrap();
        assert_eq!(r.name, "#VIIb5");
        assert_eq!(r.roman, "VII");
        assert_eq!(r.interval, "7A");
        assert_eq!(r.acc, "#");
        assert_eq!(r.chord_type, "b5");
        assert!(r.major);
        assert_eq!(r.step, 6);
        assert_eq!(r.alt, 1);
        assert_eq!(r.oct, 0);
        assert_eq!(r.dir, Direction::Up);
    }

    #[test]
    fn test_compatible_with_pitch() {
        let roman_names = |ivls: &str| -> Vec<String> {
            ivls.split(' ')
                .map(|i| get(&interval(i).unwrap()).unwrap().name)
                .collect()
        };
        assert_eq!(
            roman_names("1P 2M 3M 4P 5P 6M 7M"),
            split("I II III IV V VI VII")
        );
        assert_eq!(
            roman_names("1d 2m 3m 4d 5d 6m 7m"),
            split("bI bII bIII bIV bV bVI bVII")
        );
        assert_eq!(
            roman_names("1A 2A 3A 4A 5A 6A 7A"),
            split("#I #II #III #IV #V #VI #VII")
        );
    }

    #[test]
    fn test_can_convert_to_intervals() {
        assert_eq!(get("I").unwrap().interval, "1P");
        assert_eq!(get("bIIImaj4").unwrap().interval, "3m");
        assert_eq!(get("#IV7").unwrap().interval, "4A");
    }

    #[test]
    fn test_step() {
        let steps: Vec<usize> = names(true).iter().map(|x| get(*x).unwrap().step).collect();
        assert_eq!(steps, [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_invalid() {
        assert!(get("nothing").is_none());
        assert!(get("iI").is_none());
    }

    #[test]
    fn test_roman() {
        assert_eq!(get("IIIMaj7").unwrap().roman, "III");
        let got: Vec<String> = names(true).iter().map(|x| get(*x).unwrap().name).collect();
        assert_eq!(got, names(true).to_vec());
    }

    #[test]
    fn test_create_from_degrees() {
        let got: Vec<String> = (0usize..7).map(|i| get(i).unwrap().name).collect();
        assert_eq!(got, names(true).to_vec());
    }
}
