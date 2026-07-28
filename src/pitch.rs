#[cfg(test)]
#[path = "pitch_tests.rs"]
mod pitch_tests;

pub trait NamedPitch {
    fn name(&self) -> &str;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Direction {
    Up = 1,
    Down = -1,
}

type Fifths = i32;
type Octaves = i32;

#[derive(Debug, PartialEq, Eq)]
pub enum PitchCoordinates {
    PitchClass(Fifths),
    Note(Fifths, Octaves),
    Interval(Fifths, Octaves, Direction),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pitch {
    step: usize,
    alt: i32,
    oct: Option<i32>,
    dir: Option<Direction>,
}

const STEPS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

pub fn chroma(pitch: &Pitch) -> i32 {
    (((STEPS[pitch.step] + pitch.alt) % 12) + 12) % 12
}

pub fn height(pitch: &Pitch) -> i32 {
    let dir_val = pitch.dir.map_or(1, |d| d as i32);
    dir_val * (STEPS[pitch.step] + pitch.alt + 12 * pitch.oct.unwrap_or(-100))
}

pub fn midi(pitch: &Pitch) -> Option<i32> {
    let h = height(pitch);
    if let Some(_) = pitch.oct
        && (-12..=115).contains(&h)
    {
        return Some(h + 12);
    }
    None
}

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

pub fn coordinates(pitch: &Pitch) -> PitchCoordinates {
    let Pitch {
        step,
        alt,
        oct,
        dir,
    } = *pitch;
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
            PitchCoordinates::Note(dir_val * f, dir_val * o)
        }
        None => PitchCoordinates::PitchClass(dir_val * f),
    }
}

// return index of fifth in unaltered array FIFTHS_TO_STEPS
fn unaltered(f: i32) -> usize {
    let i = (f + 1) % 7;
    (if i < 0 { 7 + i } else { i }) as usize
}

pub fn pitch(coord: PitchCoordinates) -> Pitch {
    let (fifths, oct, dir) = match coord {
        PitchCoordinates::PitchClass(f) => (f, None, None),
        PitchCoordinates::Note(f, o) => (f, Some(o), None),
        PitchCoordinates::Interval(f, o, d) => (f, Some(o), Some(d)),
    };
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
