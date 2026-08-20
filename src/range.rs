//! Build ranges of notes between given notes.
//!
//! [`numeric`] returns the MIDI numbers spanning a list of notes (filling in
//! every semitone between successive anchors); [`chromatic`] returns those as
//! note names.

use crate::{
    collection::range,
    midi::{ToMidi, ToNoteNameOptions, midi_to_note_name},
};

/// Get the MIDI numbers spanning a list of notes.
///
/// Fills in every chromatic step between successive notes, ascending or
/// descending. Returns an empty vector if the input is empty or contains an
/// invalid note.
///
/// ```rust
/// use tonal_rs::range;
/// assert_eq!(range::numeric(&["C4", "E4"][..]), [60, 61, 62, 63, 64]);
/// assert_eq!(range::numeric(&["E4", "C4"][..]), [64, 63, 62, 61, 60]);
/// ```
pub fn numeric<T: ToMidi>(notes: &[T]) -> Vec<i32> {
    let midi: Vec<_> = notes.iter().filter_map(|n| n.to_midi()).collect();

    if notes.is_empty() || midi.len() != notes.len() {
        return vec![];
    }

    let mut result = vec![midi[0]];

    for &n in &midi[1..] {
        let last = *result.last().unwrap();
        result.extend(range(last, n).into_iter().skip(1));
    }
    result
}

/// Get the note names of a chromatic range spanning a list of notes.
///
/// Pass [`ToNoteNameOptions`] to prefer sharps or pitch classes. Returns an
/// empty vector if the input is empty or contains an invalid note.
///
/// ```rust
/// use tonal_rs::range;
/// assert_eq!(range::chromatic(&["C4", "E4"][..], None), ["C4", "Db4", "D4", "Eb4", "E4"]);
/// ```
pub fn chromatic<T: ToMidi>(notes: &[T], options: Option<ToNoteNameOptions>) -> Vec<String> {
    numeric(notes)
        .into_iter()
        .map(|m| midi_to_note_name(m, options))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // split a space-separated string into words — mirrors the TS `$` helper.
    fn words(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    #[test]
    fn test_numeric_special_cases() {
        let empty: &[i32] = &[];
        assert_eq!(numeric(empty), Vec::<i32>::new());
        assert_eq!(numeric(&["C4"][..]), [60]);
    }

    #[test]
    fn test_numeric_midi_numbers() {
        // NOTE: tonal treats raw numbers as midi without a 0..=127 clamp, so it
        // also supports negatives (`numeric([-5, 5])`). This port routes i32
        // through `ToMidi`, which clamps to 0..=127, so those out-of-range cases
        // are intentionally unsupported — nothing in tonal uses them outside its
        // own tests, and `collection::range` covers raw numeric ranges directly.
        assert_eq!(numeric(&[0, 10][..]), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(numeric(&[10, 0][..]), [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        assert_eq!(numeric(&[5, 0][..]), [5, 4, 3, 2, 1, 0]);
        assert_eq!(numeric(&[10, 5][..]), [10, 9, 8, 7, 6, 5]);
    }

    #[test]
    fn test_numeric_notes_with_names() {
        assert_eq!(
            numeric(&["C4", "C5"][..]),
            [60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72]
        );
        assert_eq!(
            numeric(&["C5", "C4"][..]),
            [72, 71, 70, 69, 68, 67, 66, 65, 64, 63, 62, 61, 60]
        );
    }

    #[test]
    fn test_numeric_multiple_notes_in_a_string() {
        assert_eq!(
            numeric(&words("C2 F2 Bb1 C2")[..]),
            [36, 37, 38, 39, 40, 41, 40, 39, 38, 37, 36, 35, 34, 35, 36]
        );
    }

    #[test]
    fn test_chromatic_note_names() {
        assert_eq!(
            chromatic(&["A3", "A4"][..], None),
            words("A3 Bb3 B3 C4 Db4 D4 Eb4 E4 F4 Gb4 G4 Ab4 A4")
        );
        assert_eq!(
            chromatic(&["A4", "A3"][..], None),
            words("A4 Ab4 G4 Gb4 F4 E4 Eb4 D4 Db4 C4 B3 Bb3 A3")
        );
        assert_eq!(
            chromatic(&words("C3 Eb3 A2")[..], None),
            words("C3 Db3 D3 Eb3 D3 Db3 C3 B2 Bb2 A2")
        );
    }

    #[test]
    fn test_chromatic_use_sharps() {
        assert_eq!(
            chromatic(
                &["C2", "C3"][..],
                Some(ToNoteNameOptions {
                    sharps: Some(true),
                    pitch_class: None,
                })
            ),
            words("C2 C#2 D2 D#2 E2 F2 F#2 G2 G#2 A2 A#2 B2 C3")
        );
        assert_eq!(
            chromatic(
                &["C2", "C3"][..],
                Some(ToNoteNameOptions {
                    sharps: Some(true),
                    pitch_class: Some(true),
                })
            ),
            words("C C# D D# E F F# G G# A A# B C")
        );
    }
}
