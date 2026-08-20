//! MIDI conversions and pitch-class-set helpers.
//!
//! Convert between MIDI numbers, frequencies and note names ([`to_midi`],
//! [`midi_to_freq`], [`freq_to_midi`], [`midi_to_note_name`]), and build
//! pitch-class sets and scale/degree mappers from MIDI numbers or chroma
//! strings ([`pcset`], [`pcset_nearest`], [`pcset_steps`], [`pcset_degrees`]).

use std::f32::consts::LN_2;

use crate::pitch_note::IntoNote;

/// Conversion into a MIDI number, implemented for `i32` (validated to 0–127)
/// and `&str` (a numeric string or a note name).
pub trait ToMidi {
    /// The MIDI number, or `None` if out of range or unparseable.
    fn to_midi(&self) -> Option<i32>;
}

impl ToMidi for i32 {
    fn to_midi(&self) -> Option<i32> {
        (0..=127).contains(self).then_some(*self)
    }
}

impl ToMidi for &str {
    fn to_midi(&self) -> Option<i32> {
        match self.trim().parse::<f64>() {
            Ok(n) => (0.0..=127.0).contains(&n).then_some(n as i32),
            _ => self.into_note().and_then(|n| n.midi),
        }
    }
}

/// Get the MIDI number of a note name or number.
///
/// ```rust
/// use tonal_rs::midi;
/// assert_eq!(midi::to_midi("C4"), Some(60));
/// assert_eq!(midi::to_midi(60), Some(60));
/// assert_eq!(midi::to_midi(128), None);
/// ```
pub fn to_midi<T: ToMidi>(note: T) -> Option<i32> {
    note.to_midi()
}

/// Get the frequency in Hz of a MIDI number (A4 = 69 = 440 Hz).
///
/// ```rust
/// use tonal_rs::midi;
/// assert_eq!(midi::midi_to_freq(69), 440.0);
/// ```
pub fn midi_to_freq(midi: i32) -> f32 {
    ((midi as f32 - 69.) / 12.).exp2() * 440.
}

/// Get the (fractional) MIDI number of a frequency in Hz.
///
/// Rounded to two decimal places.
///
/// ```rust
/// use tonal_rs::midi;
/// assert_eq!(midi::freq_to_midi(440.0), 69.0);
/// assert_eq!(midi::freq_to_midi(220.0), 57.0);
/// ```
pub fn freq_to_midi(freq: f32) -> f32 {
    let v = (12. * (freq.ln() - 440f32.ln())) / LN_2 + 69.;
    (v * 100.).round() / 100.
}

const SHARPS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

const FLATS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Options for [`midi_to_note_name`].
#[derive(Clone, Copy, Default)]
pub struct ToNoteNameOptions {
    /// If `Some(true)`, return the pitch class only (no octave).
    pub pitch_class: Option<bool>,
    /// If `Some(true)`, prefer sharps; otherwise prefer flats.
    pub sharps: Option<bool>,
}

/// Get a note name from a MIDI number.
///
/// Pass [`ToNoteNameOptions`] to prefer sharps or return the pitch class only.
///
/// ```rust
/// use tonal_rs::midi::{self, ToNoteNameOptions};
/// assert_eq!(midi::midi_to_note_name(61, None), "Db4");
/// let sharps = ToNoteNameOptions { sharps: Some(true), pitch_class: None };
/// assert_eq!(midi::midi_to_note_name(61, Some(sharps)), "C#4");
/// ```
pub fn midi_to_note_name(midi: i32, options: Option<ToNoteNameOptions>) -> String {
    let ToNoteNameOptions {
        pitch_class,
        sharps,
    } = options.unwrap_or_default();
    let pitch_class = pitch_class.unwrap_or(false);
    let sharps = sharps.unwrap_or(false);
    let pcs = if sharps { &SHARPS } else { &FLATS };
    let pc = pcs[midi.rem_euclid(12) as usize];
    if pitch_class {
        return pc.to_string();
    }
    let o = midi.div_euclid(12) - 1;
    format!("{}{}", pc, o)
}

/// Get the chroma (pitch class 0–11) of a MIDI number.
///
/// ```rust
/// use tonal_rs::midi;
/// assert_eq!(midi::chroma(60), 0);
/// assert_eq!(midi::chroma(61), 1);
/// ```
pub fn chroma(midi: i32) -> i32 {
    midi.rem_euclid(12)
}

fn pcset_from_chroma(chroma: &str) -> Vec<i32> {
    chroma
        .chars()
        .take(12)
        .enumerate()
        .filter(|(_, c)| *c == '1')
        .map(|(i, _)| i as i32)
        .collect()
}

fn pcset_from_midi(midi: &[i32]) -> Vec<i32> {
    let mut set: Vec<i32> = midi.iter().map(|&m| chroma(m)).collect();
    set.sort_unstable();
    set.dedup();
    set
}

/// Conversion into a pitch-class set (a sorted, deduplicated list of chroma
/// 0–11), implemented for chroma strings (`"101011010101"`) and MIDI slices.
pub trait ToPitchClassSet {
    /// The pitch classes present, as a sorted unique list of chroma.
    fn to_pitch_class_set(self) -> Vec<i32>;
}

impl ToPitchClassSet for &str {
    fn to_pitch_class_set(self) -> Vec<i32> {
        pcset_from_chroma(self)
    }
}

impl ToPitchClassSet for &[i32] {
    fn to_pitch_class_set(self) -> Vec<i32> {
        pcset_from_midi(self)
    }
}

/// Get the pitch-class set of a collection of MIDI numbers, or a chroma string.
///
/// Returns an empty vector for an empty input.
///
/// ```rust
/// use tonal_rs::midi;
/// assert_eq!(midi::pcset(&[62, 63, 60, 65, 70, 72][..]), [0, 2, 3, 5, 10]);
/// assert_eq!(midi::pcset("100100100101"), [0, 3, 6, 9, 11]);
/// ```
pub fn pcset<T: ToPitchClassSet>(notes: T) -> Vec<i32> {
    notes.to_pitch_class_set()
}

/// Build a function mapping a MIDI number to the nearest MIDI number whose
/// chroma is in the set.
///
/// The returned function yields `None` for every input when the set is empty.
///
/// ```rust
/// use tonal_rs::midi;
/// let nearest = midi::pcset_nearest(&[0, 5, 7][..]);
/// assert_eq!(nearest(1), Some(0));
/// assert_eq!(nearest(3), Some(5));
/// ```
pub fn pcset_nearest<T: ToPitchClassSet>(notes: T) -> impl Fn(i32) -> Option<i32> {
    let set = notes.to_pitch_class_set();
    move |midi: i32| -> Option<i32> {
        let ch = chroma(midi);
        for i in 0..12 {
            let ch_up = (ch + i) % 12;
            let ch_down = (ch - i).rem_euclid(12);
            if set.contains(&ch_up) {
                return Some(midi + i);
            }
            if set.contains(&ch_down) {
                return Some(midi - i);
            }
        }
        None
    }
}

/// Build a function mapping a step index to a MIDI number, given a set and a
/// tonic MIDI number.
///
/// Step `0` is the tonic; steps wrap around the set, adding an octave each time.
/// Negative steps descend.
///
/// ```rust
/// use tonal_rs::midi;
/// let scale = midi::pcset_steps("101010", 60);
/// assert_eq!(scale(0), 60);
/// assert_eq!(scale(3), 72);
/// ```
pub fn pcset_steps<T: ToPitchClassSet>(notes: T, tonic: i32) -> impl Fn(i32) -> i32 {
    let set = notes.to_pitch_class_set();
    let len = set.len() as i32;
    move |step: i32| -> i32 {
        let index = step.rem_euclid(len);
        let octaves = step.div_euclid(len);
        set[index as usize] + octaves * 12 + tonic
    }
}

/// Build a function mapping a 1-based degree to a MIDI number, given a set and a
/// tonic MIDI number.
///
/// Like [`pcset_steps`] but 1-indexed: degree `1` is the tonic. Degree `0`
/// returns `None`; negative degrees descend.
///
/// ```rust
/// use tonal_rs::midi;
/// let scale = midi::pcset_degrees("101010", 60);
/// assert_eq!(scale(1), Some(60));
/// assert_eq!(scale(0), None);
/// ```
pub fn pcset_degrees<T: ToPitchClassSet>(notes: T, tonic: i32) -> impl Fn(i32) -> Option<i32> {
    let steps = pcset_steps(notes, tonic);
    move |degree: i32| -> Option<i32> {
        if degree == 0 {
            return None;
        }
        Some(steps(if degree > 0 { degree - 1 } else { degree }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // freq is f32 here; tonal uses f64. Relative tolerance, matching pitch_note.
    fn assert_close(a: f32, b: f32) {
        assert!(
            (a - b).abs() <= 1e-4 * b.abs().max(1.0),
            "expected {a} to be close to {b}"
        );
    }

    // midi_to_note_name over a list of midis, joined — reads like the TS `.map(...).join(" ")`.
    fn joined(midis: &[i32], options: Option<ToNoteNameOptions>) -> String {
        midis
            .iter()
            .map(|&m| midi_to_note_name(m, options))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn test_to_midi() {
        assert_eq!(to_midi(100), Some(100));
        assert_eq!(to_midi("C4"), Some(60));
        assert_eq!(to_midi("60"), Some(60));
        assert_eq!(to_midi(0), Some(0));
        assert_eq!(to_midi("0"), Some(0));
        assert_eq!(to_midi(-1), None);
        assert_eq!(to_midi(128), None);
        assert_eq!(to_midi("blah"), None);
        // TS: toMidi(NaN) === null — no NaN for i32, so unrepresentable here.
    }

    #[test]
    fn test_freq_to_midi() {
        assert_close(freq_to_midi(220.0), 57.0);
        assert_close(freq_to_midi(261.62), 60.0);
        assert_close(freq_to_midi(261.0), 59.96);
    }

    #[test]
    fn test_midi_to_freq() {
        assert_close(midi_to_freq(60), 261.625_57);
        assert_close(midi_to_freq(69), 440.0);
        // TS: midiToFreq(69, 443) === 443 — tuning is hardcoded to 440 here.
    }

    #[test]
    fn test_midi_to_note_name() {
        let notes = [60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72];
        assert_eq!(
            joined(&notes, None),
            "C4 Db4 D4 Eb4 E4 F4 Gb4 G4 Ab4 A4 Bb4 B4 C5"
        );
        assert_eq!(
            joined(
                &notes,
                Some(ToNoteNameOptions {
                    sharps: Some(true),
                    pitch_class: None,
                })
            ),
            "C4 C#4 D4 D#4 E4 F4 F#4 G4 G#4 A4 A#4 B4 C5"
        );
        assert_eq!(
            joined(
                &notes,
                Some(ToNoteNameOptions {
                    pitch_class: Some(true),
                    sharps: None,
                })
            ),
            "C Db D Eb E F Gb G Ab A Bb B C"
        );
        // TS: NaN / ±Infinity => "" — unrepresentable for i32.
    }

    #[test]
    fn test_pcset_from_chroma() {
        assert_eq!(pcset("100100100101"), [0, 3, 6, 9, 11]);
    }

    #[test]
    fn test_pcset_from_midi() {
        assert_eq!(pcset(&[62, 63, 60, 65, 70, 72][..]), [0, 2, 3, 5, 10]);
    }

    #[test]
    fn test_pcset_nearest_upwards() {
        let nearest = pcset_nearest(&[0, 5, 7][..]);
        let out: Vec<Option<i32>> = (0..=12).map(&nearest).collect();
        assert_eq!(out, [0, 0, 0, 5, 5, 5, 7, 7, 7, 7, 12, 12, 12].map(Some));
    }

    #[test]
    fn test_pcset_nearest_c_minor_pentatonic() {
        let nearest = pcset_nearest("100101010010");
        let out: Vec<Option<i32>> = (36..=47).map(&nearest).collect();
        assert_eq!(
            out,
            [36, 36, 39, 39, 41, 41, 43, 43, 43, 46, 46, 48].map(Some)
        );
    }

    #[test]
    fn test_pcset_nearest_half_octave() {
        let nearest = pcset_nearest("100000100000");
        let out: Vec<Option<i32>> = (36..=47).map(&nearest).collect();
        assert_eq!(
            out,
            [36, 36, 36, 42, 42, 42, 42, 42, 42, 48, 48, 48].map(Some)
        );
    }

    #[test]
    fn test_pcset_nearest_empty() {
        let nearest = pcset_nearest(&[][..]);
        let out: Vec<Option<i32>> = [10, 30, 40].iter().map(|&m| nearest(m)).collect();
        assert_eq!(out, [None, None, None]);
    }

    #[test]
    fn test_pcset_steps() {
        let scale = pcset_steps("101010", 60);
        let up: Vec<i32> = (0..=9).map(&scale).collect();
        assert_eq!(up, [60, 62, 64, 72, 74, 76, 84, 86, 88, 96]);
        let down: Vec<i32> = [0, -1, -2, -3, -4, -5, -6, -7, -8, -9]
            .iter()
            .map(|&s| scale(s))
            .collect();
        assert_eq!(down, [60, 52, 50, 48, 40, 38, 36, 28, 26, 24]);
    }

    #[test]
    fn test_pcset_degrees() {
        let scale = pcset_degrees("101010", 60);
        let pos: Vec<Option<i32>> = [1, 2, 3, 4, 5].iter().map(|&d| scale(d)).collect();
        assert_eq!(pos, [60, 62, 64, 72, 74].map(Some));
        let mixed: Vec<Option<i32>> = [-1, -2, -3, 4, 5].iter().map(|&d| scale(d)).collect();
        assert_eq!(mixed, [52, 50, 48, 72, 74].map(Some));
        assert_eq!(scale(0), None);
    }
}
