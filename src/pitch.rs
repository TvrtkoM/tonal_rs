//! The low-level pitch representation shared by notes and intervals.
//!
//! A [`Pitch`] captures the abstract components common to notes and intervals —
//! a diatonic `step`, an alteration `alt` (accidentals), an optional octave and
//! an optional direction. Notes ([`Note`](crate::pitch_note::Note)) and
//! intervals ([`Interval`](crate::pitch_interval::Interval)) are built on top of
//! it. Pitches can also be expressed as [`PitchCoordinates`] — a position in
//! the array of fifths — which is the basis for transposition and distance.

/// Semitones from C for each diatonic step `[C, D, E, F, G, A, B]`.
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

/// Anything that has a musical name (a note, interval or pitch).
pub trait Named {
    /// The name of this entity, e.g. `"C#4"` or `"5P"`.
    fn name(&self) -> Cow<'_, str>;
}

/// The direction of an interval: ascending or descending.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Direction {
    /// Ascending (`+1`).
    Up = 1,
    /// Descending (`-1`).
    Down = -1,
}

/// A number of fifths along the line of fifths.
pub type Fifths = i32;
/// A number of octaves.
pub type Octaves = i32;

/// Coordinates of a pitch class: a position on the line of fifths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PitchClassCoordinates(pub Fifths);

/// Coordinates of a note: fifths plus octaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NoteCoordinates(pub Fifths, pub Octaves);

impl From<NoteCoordinates> for (Fifths, Octaves) {
    fn from(value: NoteCoordinates) -> Self {
        let NoteCoordinates(fifths, octaves) = value;
        (fifths, octaves)
    }
}

/// Coordinates of an interval: fifths, octaves and a direction.
// only used as input to conversion e.g. PitchCoordinates to Pitch
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IntervalCoordinates(pub Fifths, pub Octaves, pub Direction);

/// A pitch expressed as coordinates on the line of fifths.
///
/// The variant reflects whether the pitch is a pitch class, a note (with
/// octave), or an interval (with octave and direction).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PitchCoordinates {
    /// A pitch class (no octave).
    PitchClass(PitchClassCoordinates),
    /// A note (pitch class with octave).
    Note(NoteCoordinates),
    /// An interval (with octave span and direction).
    Interval(IntervalCoordinates),
}

/// The abstract representation of a pitch, shared by notes and intervals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pitch {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: Option<i32>,
    pub(crate) dir: Option<Direction>,
}

/// A copyable, fully public view of a [`Pitch`]'s fields, returned by
/// [`Pitch::parts`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PitchParts {
    /// Diatonic step, `0` (C) to `6` (B).
    pub step: usize,
    /// Alteration in semitones (positive = sharps, negative = flats).
    pub alt: i32,
    /// Octave, if any.
    pub oct: Option<i32>,
    /// Direction, for intervals.
    pub dir: Option<Direction>,
}

/// The number of semitones for the given step, alteration, octave and
/// direction.
///
/// ```rust
/// use tonal_rs::pitch;
/// // C4, measured relative to C0
/// assert_eq!(pitch::semitones(0, 0, 4, None), 48);
/// ```
pub fn semitones(step: usize, alt: i32, oct: i32, dir: Option<Direction>) -> i32 {
    let dir_val = dir.map_or(1, |d| d as i32);
    dir_val * (STEPS[step] + alt + 12 * oct)
}

/// The chroma (0–11) for the given step, alteration and direction.
///
/// ```rust
/// use tonal_rs::pitch;
/// assert_eq!(pitch::chroma(0, 0, None), 0); // C
/// assert_eq!(pitch::chroma(0, 1, None), 1); // C#
/// ```
pub fn chroma(step: usize, alt: i32, dir: Option<Direction>) -> i32 {
    let dir_val = dir.map_or(1, |d| d as i32);
    (dir_val * (STEPS[step] + alt)).rem_euclid(12)
}

/// Convert step/alteration/octave/direction into [`PitchCoordinates`].
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
    /// The chroma (pitch class as a number 0–11).
    pub fn chroma(&self) -> i32 {
        chroma(self.step, self.alt, None)
    }

    /// The height in semitones. For pitch classes without octave this is a
    /// large negative value used only for ordering.
    pub fn height(&self) -> i32 {
        semitones(self.step, self.alt, self.oct.unwrap_or(-100), self.dir)
    }

    /// The MIDI number, or `None` if the pitch has no octave or falls outside
    /// the MIDI range.
    pub fn midi(&self) -> Option<i32> {
        let h = self.height();
        (self.oct.is_some() && (-12..=115).contains(&h)).then_some(h + 12)
    }

    /// The pitch as [`PitchCoordinates`].
    pub fn coordinates(&self) -> PitchCoordinates {
        let Pitch {
            step,
            alt,
            oct,
            dir,
        } = *self;
        coordinates(step, alt, oct, dir)
    }

    /// The diatonic step, `0` (C) to `6` (B).
    pub fn step(&self) -> usize {
        self.step
    }

    /// The alteration in semitones (positive = sharps, negative = flats).
    pub fn alt(&self) -> i32 {
        self.alt
    }

    /// The octave, if any.
    pub fn oct(&self) -> Option<i32> {
        self.oct
    }

    /// The direction, for intervals.
    pub fn dir(&self) -> Option<Direction> {
        self.dir
    }

    /// A copyable view of all fields.
    pub fn parts(&self) -> PitchParts {
        PitchParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
            dir: self.dir,
        }
    }
}

use std::borrow::Cow;

/// Convert an alteration number into an accidental string
/// (`1` => `"#"`, `-2` => `"bb"`).
///
/// ```rust
/// use tonal_rs::pitch;
/// assert_eq!(pitch::alt_to_acc(1), "#");
/// assert_eq!(pitch::alt_to_acc(-2), "bb");
/// ```
pub use crate::pitch_note::alt_to_acc;

/// The natural letter name for a diatonic step (`0` => `'C'`, …).
///
/// Returns `None` if the step is out of range.
///
/// ```rust
/// use tonal_rs::pitch;
/// assert_eq!(pitch::step_to_letter(0), Some('C'));
/// assert_eq!(pitch::step_to_letter(9), None);
/// ```
pub fn step_to_letter(step: usize) -> Option<char> {
    "CDEFGAB".chars().nth(step)
}

impl Named for Pitch {
    fn name(&self) -> Cow<'_, str> {
        let Pitch { step, alt, oct, .. } = *self;
        let Some(letter) = step_to_letter(step) else {
            return Cow::from(String::new());
        };
        let oct = oct.map(|o| o.to_string()).unwrap_or_default();
        Cow::from(format!("{}{}{}", letter, alt_to_acc(alt), oct))
    }
}

impl std::fmt::Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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
