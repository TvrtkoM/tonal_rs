//! Parse time signatures.
//!
//! Parse a time signature from a string (`"4/4"`, `"3+2/8"`) or a numeric tuple
//! into a [`TimeSignature`] with [`get`], reading its `upper`, `lower`, `kind`
//! and any `additive` grouping.

use std::{borrow::Cow, str::FromStr};

use crate::{error::TonalParseError, pitch::Named, regexes};

/// The classification of a time signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeSignatureType {
    /// Simple meter (e.g. 4/4, 3/4).
    Simple,
    /// Compound meter (upper divisible by 3 above 3, e.g. 6/8, 9/8).
    Compound,
    /// Irregular / additive meter (e.g. 3+2/8).
    Irregular,
    /// Irrational meter (lower not a power of two).
    Irrational,
}

/// A parsed time signature.
///
/// Fields are private; read them through [`TimeSignature::parts`].
#[derive(Clone, Debug)]
pub struct TimeSignature {
    pub(crate) name: String,
    pub(crate) upper: u32,
    pub(crate) lower: u32,
    pub(crate) kind: TimeSignatureType,
    pub(crate) additive: Vec<u32>,
}

/// A borrowing, fully public view of a [`TimeSignature`], returned by
/// [`TimeSignature::parts`].
#[derive(Clone, Copy, Debug)]
pub struct TimeSignatureParts<'a> {
    /// The signature name, e.g. `"4/4"`.
    pub name: &'a str,
    /// The upper number (beats per measure).
    pub upper: u32,
    /// The lower number (beat unit).
    pub lower: u32,
    /// The classification.
    pub kind: TimeSignatureType,
    /// The additive grouping of the upper number, if any (e.g. `[3, 2]`).
    pub additive: &'a [u32],
}

impl TimeSignature {
    /// A borrowing view of all fields.
    pub fn parts(&self) -> TimeSignatureParts<'_> {
        TimeSignatureParts {
            name: &self.name,
            upper: self.upper,
            lower: self.lower,
            kind: self.kind,
            additive: &self.additive,
        }
    }
}

impl Named for TimeSignature {
    fn name(&self) -> Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Conversion into a [`TimeSignature`], implemented for a `&str` (`"4/4"`), a
/// `(u32, u32)` tuple and a `(&str, &str)` tuple.
pub trait IntoTimeSignature {
    /// Parse or build a [`TimeSignature`].
    fn into_time_signature(self) -> Option<TimeSignature>;
}

impl IntoTimeSignature for &str {
    fn into_time_signature(self) -> Option<TimeSignature> {
        let p = parse_from_str(self)?;
        build(&p)
    }
}

impl IntoTimeSignature for (u32, u32) {
    fn into_time_signature(self) -> Option<TimeSignature> {
        let p = parse_from_int_tuple(self);
        build(&p)
    }
}

impl IntoTimeSignature for (&str, &str) {
    fn into_time_signature(self) -> Option<TimeSignature> {
        let p = parse_from_str_tuple(self)?;
        build(&p)
    }
}

impl TryFrom<&str> for TimeSignature {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.into_time_signature() {
            Some(parsed) => Ok(parsed),
            _ => Err(TonalParseError {
                entity: String::from("time signature"),
                input: value.to_string(),
            }),
        }
    }
}

impl FromStr for TimeSignature {
    type Err = TonalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TimeSignature::try_from(s)
    }
}

/// Get a [`TimeSignature`] from a string or numeric tuple.
///
/// Returns `None` for an invalid time signature.
///
/// ```rust
/// use tonal_rs::time_signature::{self, TimeSignatureType};
/// let ts = time_signature::get("3/4").unwrap();
/// assert_eq!(ts.parts().upper, 3);
/// assert_eq!(ts.parts().lower, 4);
/// assert_eq!(time_signature::get((6, 8)).unwrap().parts().kind, TimeSignatureType::Compound);
/// assert!(time_signature::get("x").is_none());
/// ```
pub fn get<T: IntoTimeSignature>(literal: T) -> Option<TimeSignature> {
    literal.into_time_signature()
}

/// An intermediate parse result: either a simple `upper/lower` or an additive
/// grouping with a lower number.
#[derive(Clone, Debug)]
pub enum ParsedTimeSignature {
    /// Additive upper numbers with a single lower number, e.g. `3+2/8`.
    Additive(Vec<u32>, u32),
    /// A simple `upper/lower` signature.
    Simple(u32, u32),
}

fn parse_from_int_tuple(literal: (u32, u32)) -> ParsedTimeSignature {
    ParsedTimeSignature::Simple(literal.0, literal.1)
}

fn parse_from_str_tuple(literal: (&str, &str)) -> Option<ParsedTimeSignature> {
    let (up, down) = literal;

    let up_parts: Vec<_> = up.split("+").collect();

    let down_num = down.parse::<u32>().ok()?;

    if up_parts.len() == 1 {
        let up = *up_parts.first().unwrap();
        let up_num = up.parse::<u32>().ok()?;
        Some(ParsedTimeSignature::Simple(up_num, down_num))
    } else {
        let up_nums: Vec<_> = up_parts
            .iter()
            .flat_map(|p| p.parse::<u32>().ok())
            .collect();
        if up_nums.len() != up_parts.len() {
            None
        } else {
            Some(ParsedTimeSignature::Additive(up_nums, down_num))
        }
    }
}

fn parse_from_str(literal: &str) -> Option<ParsedTimeSignature> {
    match regexes::TIME_SIGNATURE.captures(literal) {
        Some(c) => parse_from_str_tuple((c[1].as_ref(), c[2].as_ref())),
        _ => None,
    }
}

impl From<(u32, u32)> for ParsedTimeSignature {
    fn from(value: (u32, u32)) -> Self {
        parse_from_int_tuple(value)
    }
}

impl TryFrom<(&str, &str)> for ParsedTimeSignature {
    type Error = TonalParseError;
    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        match parse_from_str_tuple(value) {
            Some(parsed) => Ok(parsed),
            _ => Err(TonalParseError {
                entity: String::from("parsed time signature"),
                input: format!("{}/{}", value.0, value.1),
            }),
        }
    }
}

impl TryFrom<&str> for ParsedTimeSignature {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parse_from_str(value) {
            Some(parsed) => Ok(parsed),
            _ => Err(TonalParseError {
                entity: String::from("parsed time signature"),
                input: value.to_string(),
            }),
        }
    }
}

impl FromStr for ParsedTimeSignature {
    type Err = TonalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedTimeSignature::try_from(s)
    }
}

static NAMES: &[&str] = &["4/4", "3/4", "2/4", "2/2", "12/8", "9/8", "6/8", "3/8"];

/// The common time signature names
/// (`["4/4", "3/4", "2/4", "2/2", "12/8", "9/8", "6/8", "3/8"]`).
pub fn names() -> &'static [&'static str] {
    NAMES
}

fn build(pts: &ParsedTimeSignature) -> Option<TimeSignature> {
    let (additive, name, upper, lower) = match pts {
        ParsedTimeSignature::Simple(u, l) => (Vec::new(), format!("{u}/{l}"), *u, *l),
        ParsedTimeSignature::Additive(u, l) => (
            Vec::from(&u[..]),
            format!(
                "{}/{l}",
                u.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            u.iter().sum(),
            *l,
        ),
    };

    if upper == 0 || lower == 0 {
        return None;
    }

    let kind = if lower == 4 || lower == 2 {
        TimeSignatureType::Simple
    } else if lower == 8 && upper % 3 == 0 {
        TimeSignatureType::Compound
    } else if lower.is_power_of_two() {
        TimeSignatureType::Irregular
    } else {
        TimeSignatureType::Irrational
    };

    Some(TimeSignature {
        name,
        kind,
        upper,
        lower,
        additive,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(literal: &str) -> TimeSignatureType {
        get(literal).unwrap().kind
    }

    #[test]
    fn test_get() {
        let ts = get("4/4").unwrap();
        assert_eq!(ts.name, "4/4");
        assert_eq!(ts.kind, TimeSignatureType::Simple);
        assert_eq!(ts.upper, 4);
        assert_eq!(ts.lower, 4);
        assert_eq!(ts.additive, [] as [u32; 0]);
    }

    #[test]
    fn test_get_invalid() {
        assert!(get("0/0").is_none());
    }

    #[test]
    fn test_simple() {
        assert_eq!(kind("4/4"), TimeSignatureType::Simple);
        assert_eq!(kind("3/4"), TimeSignatureType::Simple);
        assert_eq!(kind("2/4"), TimeSignatureType::Simple);
        assert_eq!(kind("2/2"), TimeSignatureType::Simple);
    }

    #[test]
    fn test_compound() {
        assert_eq!(kind("3/8"), TimeSignatureType::Compound);
        assert_eq!(kind("6/8"), TimeSignatureType::Compound);
        assert_eq!(kind("9/8"), TimeSignatureType::Compound);
        assert_eq!(kind("12/8"), TimeSignatureType::Compound);
    }

    #[test]
    fn test_irregular() {
        assert_eq!(kind("2+3+3/8"), TimeSignatureType::Irregular);
        assert_eq!(kind("3+2+2/8"), TimeSignatureType::Irregular);
    }

    #[test]
    fn test_irrational() {
        assert_eq!(kind("12/10"), TimeSignatureType::Irrational);
        assert_eq!(kind("12/19"), TimeSignatureType::Irrational);
    }

    #[test]
    fn test_names() {
        assert_eq!(
            names(),
            ["4/4", "3/4", "2/4", "2/2", "12/8", "9/8", "6/8", "3/8"]
        );
    }
}
