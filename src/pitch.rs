pub const STEPS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

// the number of fifths of [C, D, E, F, G, A, B] (from C)
const FIFTHS: [i32; 7] = [0, 2, 4, -1, 1, 3, 5];

// fifths to steps ordered like [F, C, G, D, A, E, B]
// notice those represent corresponding index in FIFTHS array
const FIFTHS_TO_STEPS: [usize; 7] = [3, 0, 4, 1, 5, 2, 6];

const fn make_steps_to_octs() -> [i32; 7] {
    let mut result = [0; 7];
    let mut i = 0;
    while i < 7 {
        result[i] = (FIFTHS[i] * 7).div_euclid(12);
        i += 1;
    }
    result
}

const STEPS_TO_OCTS: [i32; 7] = make_steps_to_octs();

pub trait Named {
    fn name(&self) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Direction {
    Up = 1,
    Down = -1,
}

pub type Fifths = i32;
pub type Octaves = i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PitchClassCoordinates(pub Fifths);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteCoordinates(pub Fifths, pub Octaves);

impl From<NoteCoordinates> for (Fifths, Octaves) {
    fn from(value: NoteCoordinates) -> Self {
        let NoteCoordinates(fifths, octaves) = value;
        (fifths, octaves)
    }
}

// only used as input to conversion e.g. PitchCoordinates to Pitch
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntervalCoordinates(pub Fifths, pub Octaves, pub Direction);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchCoordinates {
    PitchClass(PitchClassCoordinates),
    Note(NoteCoordinates),
    Interval(IntervalCoordinates),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pitch {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: Option<i32>,
    pub(crate) dir: Option<Direction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PitchParts {
    pub step: usize,
    pub alt: i32,
    pub oct: Option<i32>,
    pub dir: Option<Direction>,
}

pub fn semitones(step: usize, alt: i32, oct: i32, dir: Option<Direction>) -> i32 {
    let dir_val = dir.map_or(1, |d| d as i32);
    dir_val * (STEPS[step] + alt + 12 * oct)
}

pub fn chroma(step: usize, alt: i32, dir: Option<Direction>) -> i32 {
    let dir_val = dir.map_or(1, |d| d as i32);
    (dir_val * (STEPS[step] + alt)).rem_euclid(12)
}

pub fn coordinates(
    step: usize,
    alt: i32,
    oct: Option<i32>,
    dir: Option<Direction>,
) -> PitchCoordinates {
    let dir_val = dir.map_or(1, |d| d as i32);
    // adding 7 fifths to a note makes it a sharp
    let f = FIFTHS[step] + 7 * alt;

    match oct {
        Some(o) => {
            // correct octave from octaves added by fifths
            // STEPS_TO_OCTS[step] - expressing letter as fifths of C overshoots into other octaves
            // 4 * alt - e.g. sharp adds 7 fifths = 49 semitones = 4 octaves + 1 semitone and we only
            //   wanted a semitone
            let o = o - STEPS_TO_OCTS[step] - 4 * alt;
            PitchCoordinates::Note(NoteCoordinates(dir_val * f, dir_val * o))
        }
        None => PitchCoordinates::PitchClass(PitchClassCoordinates(dir_val * f)),
    }
}

impl Pitch {
    pub fn chroma(&self) -> i32 {
        chroma(self.step, self.alt, None)
    }

    pub fn height(&self) -> i32 {
        semitones(self.step, self.alt, self.oct.unwrap_or(-100), self.dir)
    }

    pub fn midi(&self) -> Option<i32> {
        let h = self.height();
        (self.oct.is_some() && (-12..=115).contains(&h)).then_some(h + 12)
    }

    pub fn coordinates(&self) -> PitchCoordinates {
        let Pitch {
            step,
            alt,
            oct,
            dir,
        } = *self;
        coordinates(step, alt, oct, dir)
    }

    pub fn step(&self) -> usize {
        self.step
    }

    pub fn alt(&self) -> i32 {
        self.alt
    }

    pub fn oct(&self) -> Option<i32> {
        self.oct
    }

    pub fn dir(&self) -> Option<Direction> {
        self.dir
    }

    pub fn parts(&self) -> PitchParts {
        PitchParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
            dir: self.dir,
        }
    }
}

pub fn alt_to_acc(alt: i32) -> String {
    if alt < 0 {
        "b".repeat(alt.unsigned_abs() as usize)
    } else {
        "#".repeat(alt as usize)
    }
}

pub fn step_to_letter(step: usize) -> Option<char> {
    "CDEFGAB".chars().nth(step)
}

impl Named for Pitch {
    fn name(&self) -> String {
        let Pitch { step, alt, oct, .. } = *self;
        let Some(letter) = step_to_letter(step) else {
            return String::new();
        };
        let oct = oct.map(|o| o.to_string()).unwrap_or_default();
        format!("{}{}{}", letter, alt_to_acc(alt), oct)
    }
}

// return index of fifth in unaltered array FIFTHS_TO_STEPS
fn unaltered(f: i32) -> usize {
    (f + 1).rem_euclid(7) as usize
}

impl From<&PitchCoordinates> for (i32, Option<i32>, Option<Direction>) {
    fn from(coord: &PitchCoordinates) -> Self {
        match *coord {
            PitchCoordinates::PitchClass(PitchClassCoordinates(f)) => (f, None, None),
            PitchCoordinates::Note(NoteCoordinates(f, o)) => (f, Some(o), None),
            PitchCoordinates::Interval(IntervalCoordinates(f, o, d)) => (f, Some(o), Some(d)),
        }
    }
}

impl From<(i32, Option<i32>, Option<Direction>)> for PitchCoordinates {
    fn from((f, oct, dir): (i32, Option<i32>, Option<Direction>)) -> Self {
        match (oct, dir) {
            (Some(o), Some(d)) => PitchCoordinates::Interval(IntervalCoordinates(f, o, d)),
            (Some(o), None) => PitchCoordinates::Note(NoteCoordinates(f, o)),
            (None, _) => PitchCoordinates::PitchClass(PitchClassCoordinates(f)),
        }
    }
}

impl From<&PitchCoordinates> for Pitch {
    fn from(coord: &PitchCoordinates) -> Self {
        let (fifths, oct, dir) = coord.into();
        let step = FIFTHS_TO_STEPS[unaltered(fifths)];
        let alt = (fifths + 1).div_euclid(7);
        if let Some(o) = oct {
            let o = o + 4 * alt + STEPS_TO_OCTS[step];
            return Pitch {
                step,
                alt,
                oct: Some(o),
                dir,
            };
        }
        Pitch {
            step,
            alt,
            oct,
            dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pitch classes
    fn c() -> Pitch {
        Pitch {
            step: 0,
            alt: 0,
            oct: None,
            dir: None,
        }
    }
    fn cs() -> Pitch {
        Pitch {
            step: 0,
            alt: 1,
            oct: None,
            dir: None,
        }
    }
    fn cb() -> Pitch {
        Pitch {
            step: 0,
            alt: -1,
            oct: None,
            dir: None,
        }
    }
    fn a() -> Pitch {
        Pitch {
            step: 5,
            alt: 0,
            oct: None,
            dir: None,
        }
    }

    // Notes
    fn c4() -> Pitch {
        Pitch {
            step: 0,
            alt: 0,
            oct: Some(4),
            dir: None,
        }
    }
    fn a4() -> Pitch {
        Pitch {
            step: 5,
            alt: 0,
            oct: Some(4),
            dir: None,
        }
    }
    fn gs6() -> Pitch {
        Pitch {
            step: 4,
            alt: 1,
            oct: Some(6),
            dir: None,
        }
    }

    // Intervals
    fn p5() -> Pitch {
        Pitch {
            step: 4,
            alt: 0,
            oct: Some(0),
            dir: Some(Direction::Up),
        }
    }
    fn p_5() -> Pitch {
        Pitch {
            step: 4,
            alt: 0,
            oct: Some(0),
            dir: Some(Direction::Down),
        }
    }

    #[test]
    fn test_height() {
        let pcs: Vec<i32> = [c(), cs(), cb(), a()].iter().map(Pitch::height).collect();
        assert_eq!(pcs, [-1200, -1199, -1201, -1191]);

        let notes: Vec<i32> = [c4(), a4(), gs6()].iter().map(Pitch::height).collect();
        assert_eq!(notes, [48, 57, 80]);

        let intervals: Vec<i32> = [p5(), p_5()].iter().map(Pitch::height).collect();
        assert_eq!(intervals, [7, -7]);
    }

    #[test]
    fn test_midi() {
        let pcs: Vec<Option<i32>> = [c(), cs(), cb(), a()].iter().map(Pitch::midi).collect();
        assert_eq!(pcs, [None, None, None, None]);

        let notes: Vec<Option<i32>> = [c4(), a4(), gs6()].iter().map(Pitch::midi).collect();
        assert_eq!(notes, [Some(60), Some(69), Some(92)]);
    }

    #[test]
    fn test_chroma() {
        let pcs: Vec<i32> = [c(), cs(), cb(), a()].iter().map(Pitch::chroma).collect();
        assert_eq!(pcs, [0, 1, 11, 9]);

        let notes: Vec<i32> = [c4(), a4(), gs6()].iter().map(Pitch::chroma).collect();
        assert_eq!(notes, [0, 9, 8]);

        let intervals: Vec<i32> = [p5(), p_5()].iter().map(Pitch::chroma).collect();
        assert_eq!(intervals, [7, 7]);
    }

    #[test]
    fn test_coordinates() {
        // pitch classes
        assert_eq!(
            c().coordinates(),
            PitchCoordinates::PitchClass(PitchClassCoordinates(0))
        );
        assert_eq!(
            a().coordinates(),
            PitchCoordinates::PitchClass(PitchClassCoordinates(3))
        );
        assert_eq!(
            cs().coordinates(),
            PitchCoordinates::PitchClass(PitchClassCoordinates(7))
        );
        assert_eq!(
            cb().coordinates(),
            PitchCoordinates::PitchClass(PitchClassCoordinates(-7))
        );
        // notes
        assert_eq!(
            c4().coordinates(),
            PitchCoordinates::Note(NoteCoordinates(0, 4))
        );
        assert_eq!(
            a4().coordinates(),
            PitchCoordinates::Note(NoteCoordinates(3, 3))
        );
        // intervals (direction is folded into the sign, like the TS version)
        assert_eq!(
            p5().coordinates(),
            PitchCoordinates::Note(NoteCoordinates(1, 0))
        );
        // TS expects [-1, -0]; Rust integers have no negative zero, so this is 0.
        assert_eq!(
            p_5().coordinates(),
            PitchCoordinates::Note(NoteCoordinates(-1, 0))
        );
    }

    #[test]
    fn test_from_coordinates() {
        assert_eq!(
            Pitch::from(&PitchCoordinates::PitchClass(PitchClassCoordinates(0))),
            c()
        );
        assert_eq!(
            Pitch::from(&PitchCoordinates::PitchClass(PitchClassCoordinates(7))),
            cs()
        );
    }

    #[test]
    fn test_name() {
        // Named for Pitch (moved into this module)
        assert_eq!(c().name(), "C");
        assert_eq!(cs().name(), "C#");
        assert_eq!(cb().name(), "Cb");
        assert_eq!(a().name(), "A");
        assert_eq!(c4().name(), "C4");
        assert_eq!(a4().name(), "A4");
        assert_eq!(gs6().name(), "G#6");
        // out-of-range step yields an empty name
        let bad = Pitch {
            step: 8,
            alt: 0,
            oct: None,
            dir: None,
        };
        assert_eq!(bad.name(), "");
    }

    #[test]
    fn test_accessors() {
        let p = gs6();
        assert_eq!(p.step(), 4);
        assert_eq!(p.alt(), 1);
        assert_eq!(p.oct(), Some(6));
        assert_eq!(p.dir(), None);
        assert_eq!(
            p.parts(),
            PitchParts {
                step: 4,
                alt: 1,
                oct: Some(6),
                dir: None
            }
        );

        assert_eq!(p_5().dir(), Some(Direction::Down));
    }
}
