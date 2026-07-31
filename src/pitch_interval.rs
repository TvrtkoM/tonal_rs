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
            Quality::Diminished(if ty == IntervalType::Perfectable {
                alt as u8
            } else {
                alt as u8 + 1
            })
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
