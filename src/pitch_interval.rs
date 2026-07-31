use regex::Regex;
use std::sync::LazyLock;

use crate::{
    error::TonalParseError,
    pitch::{
        Direction, Named, NoteCoordinates, Pitch, PitchCoordinates, chroma, coordinates, semitones,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalType {
    Perfectable,
    Majorable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Diminished(u8),
    Minor,
    Major,
    Perfect,
    Augmented(u8),
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Quality::Diminished(n) => write!(f, "{}", "d".repeat(*n as usize)),
            Quality::Minor => f.write_str("m"),
            Quality::Major => f.write_str("M"),
            Quality::Perfect => f.write_str("P"),
            Quality::Augmented(n) => write!(f, "{}", "A".repeat(*n as usize)),
        }
    }
}

impl TryFrom<&str> for Quality {
    type Error = TonalParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "M" => Quality::Major,
            "m" => Quality::Minor,
            "P" => Quality::Perfect,
            s if s.chars().all(|c| c == 'd') && !s.is_empty() => Quality::Diminished(s.len() as u8),
            s if s.chars().all(|c| c == 'A') && !s.is_empty() => Quality::Augmented(s.len() as u8),
            _ => {
                return Err(TonalParseError {
                    entity: String::from("quality"),
                    input: s.to_string(),
                });
            }
        })
    }
}

impl Quality {
    pub fn to_alt(self, ty: IntervalType) -> i32 {
        use IntervalType::*;
        use Quality::*;
        match (self, ty) {
            (Perfect, Perfectable) | (Major, Majorable) => 0,
            (Minor, Majorable) => -1,
            (Augmented(n), _) => n as i32,
            (Diminished(n), Perfectable) => -(n as i32),
            (Diminished(n), Majorable) => -(n as i32) - 1, // below minor
            _ => 0,
        }
    }

    pub fn from_alt(ty: IntervalType, alt: i32) -> Quality {
        if alt == 0 {
            if ty == IntervalType::Majorable {
                Quality::Major
            } else {
                Quality::Perfect
            }
        } else if alt == -1 && ty == IntervalType::Majorable {
            Quality::Minor
        } else if alt > 0 {
            Quality::Augmented(alt as u8)
        } else {
            // alt is negative here; the number of `d`s is its magnitude
            // (one less for majorable, since diminished sits below minor).
            let count = if ty == IntervalType::Perfectable {
                -alt
            } else {
                -alt - 1
            };
            Quality::Diminished(count as u8)
        }
    }
}

type IntervalTokens = (String, String);

// group 1-2: tonal notation (number then quality)
// group 3-4: shorthand notation (quality then number)
static INTERVAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:([-+]?\d+)(d{1,4}|m|M|P|A{1,4})|(AA|A|P|M|m|d|dd)([-+]?\d+))$").unwrap()
});

pub(crate) fn tokenize_interval(s: &str) -> IntervalTokens {
    match INTERVAL_REGEX.captures(s) {
        // group 1 present => tonal alternative matched (number = 1, quality = 2)
        Some(c) if c.get(1).is_some() => (c[1].to_string(), c[2].to_string()),
        // otherwise the shorthand alternative matched (quality = 3, number = 4)
        Some(c) => (c[4].to_string(), c[3].to_string()),
        None => (String::new(), String::new()),
    }
}

const TYPES: [IntervalType; 7] = [
    IntervalType::Perfectable, // 1  unison
    IntervalType::Majorable,   // 2  second
    IntervalType::Majorable,   // 3  third
    IntervalType::Perfectable, // 4  fourth
    IntervalType::Perfectable, // 5  fifth
    IntervalType::Majorable,   // 6  sixth
    IntervalType::Majorable,   // 7  seventh
];

#[derive(Debug, PartialEq)]
pub struct Interval {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: i32,
    pub(crate) dir: Direction,
    pub(crate) name: String,
    pub(crate) num: i32,
    pub(crate) q: Quality,
    pub(crate) typ: IntervalType,
    pub(crate) simple: i32,
    pub(crate) semitones: i32,
    pub(crate) chroma: i32,
    // intentionally using NoteCoordinates here for ease of transposition
    // direction is encoded struct itself and sign in NoteCoordinates
    pub(crate) coord: NoteCoordinates,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntervalParts<'a> {
    pub step: usize,
    pub alt: i32,
    pub oct: i32,
    pub dir: Direction,
    pub name: &'a str,
    pub num: i32,
    pub q: Quality,
    pub typ: IntervalType,
    pub simple: i32,
    pub semitones: i32,
    pub chroma: i32,
    // intentionally using NoteCoordinates here for ease of transposition
    // direction is encoded struct itself and sign in NoteCoordinates
    pub coord: NoteCoordinates,
}

impl Interval {
    pub fn parts(&self) -> IntervalParts<'_> {
        IntervalParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
            dir: self.dir,
            name: &self.name,
            num: self.num,
            q: self.q,
            typ: self.typ,
            simple: self.simple,
            semitones: self.semitones,
            chroma: self.chroma,
            coord: self.coord,
        }
    }
}

impl Named for Interval {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub fn interval(src: &str) -> Option<Interval> {
    parse(src)
}

impl TryFrom<&Pitch> for Interval {
    type Error = TonalParseError;
    fn try_from(p: &Pitch) -> Result<Self, Self::Error> {
        let name = pitch_name(p);
        interval(&name).ok_or(TonalParseError {
            entity: String::from("interval"),
            input: name,
        })
    }
}

impl TryFrom<&str> for Interval {
    type Error = TonalParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        interval(s).ok_or_else(|| TonalParseError {
            entity: String::from("interval"),
            input: s.to_string(),
        })
    }
}

fn parse(str: &str) -> Option<Interval> {
    let (tok1, tok2) = tokenize_interval(str);
    if tok1.is_empty() || tok2.is_empty() {
        return None;
    }
    // unwrap safe to use - strings are validated by regex already
    let num = tok1.parse::<i32>().unwrap();
    let q: Quality = tok2.as_str().try_into().unwrap();
    let step = ((num.abs() - 1) % 7) as usize;
    let typ = TYPES[step];
    if typ == IntervalType::Majorable && q == Quality::Perfect {
        return None;
    }

    let name = format!("{}{}", num, q);
    let dir = if num < 0 {
        Direction::Down
    } else {
        Direction::Up
    };

    let simple = if num == 8 || num == -8 {
        num
    } else {
        (dir as i32) * (step as i32 + 1)
    };

    let alt = q.to_alt(typ);
    let oct = (num.abs() - 1).div_euclid(7);
    let semitones = semitones(step, alt, oct, Some(dir));
    let chroma = chroma(step, alt, Some(dir));
    let PitchCoordinates::Note(coord) = coordinates(step, alt, Some(oct), Some(dir)) else {
        unreachable!("coordinates() always yield NoteCoordinates here");
    };

    Some(Interval {
        step,
        alt,
        oct,
        dir,
        name,
        num,
        q,
        typ,
        simple,
        semitones,
        chroma,
        coord,
    })
}

// interval name from pitch
fn pitch_name(p: &Pitch) -> String {
    let Pitch {
        step,
        alt,
        oct,
        dir,
    } = *p;
    if dir.is_none() {
        return String::new();
    }
    let oct = oct.unwrap_or(0);
    let calc_num = step as i32 + 1 + 7 * oct;
    let num = if calc_num == 0 {
        step as i32 + 1
    } else {
        calc_num
    };
    let dir = if let Some(d) = dir
        && (d as i8) < 0
    {
        "-"
    } else {
        ""
    };
    let ty = TYPES[step];
    format!("{}{}{}", dir, num, Quality::from_alt(ty, alt))
}

#[cfg(test)]
mod tests {
    use super::*;

    // interval(i).name for each space-separated interval, rejoined (all valid).
    fn names(s: &str) -> String {
        s.split(' ')
            .map(|i| interval(i).unwrap().name)
            .collect::<Vec<_>>()
            .join(" ")
    }

    // interval(i).q rendered to string, for each space-separated interval.
    fn qs(s: &str) -> Vec<String> {
        s.split(' ')
            .map(|i| interval(i).unwrap().q.to_string())
            .collect()
    }

    fn alts(s: &str) -> Vec<i32> {
        s.split(' ').map(|i| interval(i).unwrap().alt).collect()
    }

    fn simples(s: &str) -> Vec<i32> {
        s.split(' ').map(|i| interval(i).unwrap().simple).collect()
    }

    // interval from pitch props, returning the name (or None if invalid).
    fn from_pitch(step: usize, alt: i32, oct: Option<i32>, dir: Option<Direction>) -> Option<String> {
        Interval::try_from(&Pitch { step, alt, oct, dir })
            .ok()
            .map(|i| i.name)
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize_interval("-2M"), ("-2".into(), "M".into()));
        assert_eq!(tokenize_interval("M-3"), ("-3".into(), "M".into()));
    }

    #[test]
    fn test_properties() {
        let i = interval("4d").unwrap();
        assert_eq!(i.name, "4d");
        assert_eq!(i.num, 4);
        assert_eq!(i.q, Quality::Diminished(1));
        assert_eq!(i.q.to_string(), "d");
        assert_eq!(i.typ, IntervalType::Perfectable);
        assert_eq!(i.alt, -1);
        assert_eq!(i.chroma, 4);
        assert_eq!(i.dir, Direction::Up);
        assert_eq!(i.coord, NoteCoordinates(-8, 5));
        assert_eq!(i.oct, 0);
        assert_eq!(i.semitones, 4);
        assert_eq!(i.simple, 4);
        assert_eq!(i.step, 3);
    }

    #[test]
    fn test_accepts_interval_as_parameter() {
        // TS: interval(interval("5P")) === interval("5P"). Re-parse its own name.
        let p5 = interval("5P").unwrap();
        assert_eq!(interval(&p5.name), interval("5P"));
    }

    #[test]
    fn test_name() {
        assert_eq!(names("1P 2M 3M 4P 5P 6M 7M"), "1P 2M 3M 4P 5P 6M 7M");
        assert_eq!(names("P1 M2 M3 P4 P5 M6 M7"), "1P 2M 3M 4P 5P 6M 7M");
        assert_eq!(names("-1P -2M -3M -4P -5P -6M -7M"), "-1P -2M -3M -4P -5P -6M -7M");
        assert_eq!(names("P-1 M-2 M-3 P-4 P-5 M-6 M-7"), "-1P -2M -3M -4P -5P -6M -7M");
        assert!(interval("not-an-interval").is_none());
        assert!(interval("2P").is_none());
    }

    #[test]
    fn test_q() {
        assert_eq!(qs("1dd 1d 1P 1A 1AA"), ["dd", "d", "P", "A", "AA"]);
        assert_eq!(qs("2dd 2d 2m 2M 2A 2AA"), ["dd", "d", "m", "M", "A", "AA"]);
    }

    #[test]
    fn test_alt() {
        assert_eq!(alts("1dd 2dd 3dd 4dd"), [-2, -3, -3, -2]);
    }

    #[test]
    fn test_simple() {
        assert_eq!(simples("1P 2M 3M 4P"), [1, 2, 3, 4]);
        assert_eq!(simples("8P 9M 10M 11P"), [8, 2, 3, 4]);
        assert_eq!(simples("-8P -9M -10M -11P"), [-8, -2, -3, -4]);
    }

    #[test]
    fn test_from_pitch_props() {
        use Direction::*;
        assert_eq!(from_pitch(0, 0, None, Some(Up)).as_deref(), Some("1P"));
        assert_eq!(from_pitch(0, -2, None, Some(Up)).as_deref(), Some("1dd"));
        assert_eq!(from_pitch(1, 1, None, Some(Up)).as_deref(), Some("2A"));
        assert_eq!(from_pitch(2, -2, None, Some(Up)).as_deref(), Some("3d"));
        assert_eq!(from_pitch(1, 1, None, Some(Down)).as_deref(), Some("-2A"));
        // no dir -> not an interval
        assert!(from_pitch(1000, 0, None, None).is_none());
    }

    #[test]
    fn test_from_pitch_props_with_octave() {
        use Direction::*;
        assert_eq!(from_pitch(0, 0, Some(0), Some(Up)).as_deref(), Some("1P"));
        assert_eq!(from_pitch(0, -1, Some(1), Some(Down)).as_deref(), Some("-8d"));
        assert_eq!(from_pitch(0, 1, Some(2), Some(Down)).as_deref(), Some("-15A"));
        assert_eq!(from_pitch(1, -1, Some(1), Some(Down)).as_deref(), Some("-9m"));
    }
}
