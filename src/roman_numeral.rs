use std::sync::LazyLock;

use regex::Regex;

use crate::{
    pitch::{Direction, Pitch, alt_to_acc},
    pitch_interval::{IntoInterval, Interval},
    pitch_note::acc_to_alt,
};

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct RomanNumeralParts<'a> {
    pub step: usize,
    pub alt: i32,
    pub oct: i32,
    pub dir: Direction,
    pub name: &'a str,
    pub roman: &'a str,
    pub interval: &'a str,
    pub acc: &'a str,
    pub chord_type: &'a str,
    pub major: bool,
}

impl RomanNumeral {
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

static NAMES: &[&str] = &["I", "II", "III", "IV", "V", "VI", "VII"];
static NAMES_MINOR: &[&str] = &["i", "ii", "iii", "iv", "v", "vi", "vii"];

pub fn names(major: bool) -> &'static [&'static str] {
    if major { NAMES } else { NAMES_MINOR }
}

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(#+|b+|x+|)(IV|I{1,3}|VI{0,2}|iv|i{1,3}|vi{0,2})([^IViv]*)$").unwrap()
});

pub fn tokenize(s: &str) -> (String, String, String, String) {
    let [a, b, c, d] = REGEX.captures(s).map_or(["", "", "", ""], |c| {
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

pub trait IntoRomanNumeral {
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
        assert_eq!(names(true).to_vec(), ["I", "II", "III", "IV", "V", "VI", "VII"]);
        assert_eq!(names(false).to_vec(), ["i", "ii", "iii", "iv", "v", "vi", "vii"]);
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
