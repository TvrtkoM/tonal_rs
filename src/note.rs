//! Get properties of notes and transpose them.
//!
//! A note is described by a scientific-notation name such as `"C4"` or `"Bb"`.
//! Use [`get`] to obtain a [`Note`] with all of its
//! properties, or the shorthand accessors ([`name`], [`midi`], [`freq`], …) to
//! read a single property directly from a note name.

use std::cmp::Ordering;

use crate::{
    midi::{ToNoteNameOptions, freq_to_midi, midi_to_note_name},
    pitch_distance::transpose_with_coord,
    pitch_note::{IntoNote, Note},
};

const NAMES: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];

/// Get a [`Note`] from a note name.
///
/// Returns `None` if the name is not a valid note.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::get("Bb4").unwrap().parts().midi, Some(70));
/// assert!(note::get("x").is_none());
/// ```
pub use crate::pitch_note::note as get;

/// Get the full note name (pitch class + octave).
///
/// Returns an empty string for invalid input.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::name("db"), "Db");
/// assert_eq!(note::name("x"), "");
/// ```
pub fn name<T: IntoNote>(note: T) -> String {
    note.into_note().map(|n| n.name).unwrap_or_default()
}

/// Get the pitch class (note name without octave).
///
/// Returns an empty string for invalid input.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::pitch_class("Ax4"), "A##");
/// assert_eq!(note::pitch_class("x"), "");
/// ```
pub fn pitch_class<T: IntoNote>(note: T) -> String {
    note.into_note().map(|n| n.pc).unwrap_or_default()
}

/// Get the note accidentals (`"#"`, `"b"`, `"##"`, …).
///
/// Returns an empty string for a natural note or invalid input.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::accidentals("c#2"), "#");
/// assert_eq!(note::accidentals("C"), "");
/// ```
pub fn accidentals<T: IntoNote>(note: T) -> String {
    note.into_note().map(|n| n.acc).unwrap_or_default()
}

/// Get the note octave.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::octave("C4"), Some(4));
/// assert_eq!(note::octave("C"), None);
/// ```
pub fn octave<T: IntoNote>(note: T) -> Option<i32> {
    note.into_note()?.oct
}

/// Get the note MIDI number.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::midi("db4"), Some(61));
/// assert_eq!(note::midi("C"), None);
/// ```
pub fn midi<T: IntoNote>(note: T) -> Option<i32> {
    note.into_note()?.midi
}

/// Get the note frequency in Hz (using A4 = 440 Hz).
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::freq("A4"), Some(440.0));
/// assert_eq!(note::freq("x"), None);
/// ```
pub fn freq<T: IntoNote>(note: T) -> Option<f32> {
    note.into_note()?.freq
}

/// Get the note chroma — the pitch class as a number from 0 (C) to 11 (B).
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::chroma("db4"), Some(1));
/// assert_eq!(note::chroma("x"), None);
/// ```
pub fn chroma<T: IntoNote>(note: T) -> Option<i32> {
    note.into_note().map(|n| n.chroma)
}

/// Get a note name from a MIDI number, preferring flats.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::from_midi(70), "Bb4");
/// assert_eq!(note::from_midi(61), "Db4");
/// ```
pub fn from_midi(midi: i32) -> String {
    midi_to_note_name(midi, None)
}

/// Get a note name from a frequency in Hz, preferring flats.
///
/// The nearest chromatic note is returned; invalid frequencies yield an empty
/// string.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::from_freq(440.0), "A4");
/// assert_eq!(note::from_freq(470.0), "Bb4");
/// assert_eq!(note::from_freq(0.0), "");
/// ```
pub fn from_freq(freq: f32) -> String {
    let midi = freq_to_midi(freq);
    if !midi.is_finite() {
        return String::new();
    }
    midi_to_note_name(midi.round() as i32, None)
}

/// Get a note name from a frequency in Hz, preferring sharps.
///
/// Invalid frequencies yield an empty string.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::from_freq_sharps(470.0), "A#4");
/// assert_eq!(note::from_freq_sharps(0.0), "");
/// ```
pub fn from_freq_sharps(freq: f32) -> String {
    let midi = freq_to_midi(freq);
    if !midi.is_finite() {
        return String::new();
    }
    midi_to_note_name(
        midi.round() as i32,
        Some(ToNoteNameOptions {
            sharps: Some(true),
            pitch_class: None,
        }),
    )
}

/// Get a note name from a MIDI number, preferring sharps.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::from_midi_sharps(61), "C#4");
/// assert_eq!(note::from_midi_sharps(70), "A#4");
/// ```
pub fn from_midi_sharps(midi: i32) -> String {
    midi_to_note_name(
        midi,
        Some(ToNoteNameOptions {
            sharps: Some(true),
            pitch_class: None,
        }),
    )
}

/// Get the interval distance between two notes.
///
/// Returns an empty string if either note is invalid.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::distance("C4", "G4"), "5P");
/// assert_eq!(note::distance("one", "two"), "");
/// ```
pub use crate::pitch_distance::distance;

/// Transpose a note by an interval.
///
/// Returns an empty string if the note or interval is invalid.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::transpose("C4", "5P"), "G4");
/// assert_eq!(note::transpose("D", "3M"), "F#");
/// ```
pub use crate::pitch_distance::transpose;
/// Alias of [`transpose`].
pub use crate::pitch_distance::transpose as tr;

/// Partially apply [`transpose`] to a note, returning a function that
/// transposes that note by any interval.
///
/// The returned function yields an empty string for an invalid note or
/// interval.
///
/// ```rust
/// use tonal_rs::note;
/// let from_c = note::transpose_from("C4");
/// assert_eq!(from_c("5P"), "G4");
/// assert_eq!(from_c("3M"), "E4");
/// ```
pub fn transpose_from(note: &str) -> impl Fn(&str) -> String {
    move |interval: &str| -> String { transpose(note, interval) }
}

/// Alias of [`transpose_from`].
pub use self::transpose_from as tr_from;

/// Partially apply [`transpose`] to an interval, returning a function that
/// transposes any note by that interval.
///
/// The returned function yields an empty string for an invalid note or
/// interval.
///
/// ```rust
/// use tonal_rs::note;
/// let up_fifth = note::transpose_by("5P");
/// assert_eq!(up_fifth("C4"), "G4");
/// assert_eq!(up_fifth("D"), "A");
/// ```
pub fn transpose_by(interval: &str) -> impl Fn(&str) -> String {
    move |note: &str| -> String { transpose(note, interval) }
}

/// Alias of [`transpose_by`].
pub use self::transpose_by as tr_by;

/// Transpose a note by a number of perfect fifths.
///
/// Returns an empty string for an invalid note.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::transpose_fifths("G4", 3), "E6");
/// assert_eq!(note::transpose_fifths("C2", 1), "G2");
/// ```
pub fn transpose_fifths(note_name: &str, fifths: i32) -> String {
    transpose_with_coord(note_name, (fifths, 0))
}

/// Alias of [`transpose_fifths`].
pub use self::transpose_fifths as tr_fifths;

/// Transpose a note by a number of octaves.
///
/// Returns an empty string for an invalid note.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::transpose_octaves("C4", 2), "C6");
/// assert_eq!(note::transpose_octaves("C4", -1), "C3");
/// ```
pub fn transpose_octaves(note_name: &str, octaves: i32) -> String {
    transpose_with_coord(note_name, (0, octaves))
}

/// Alias of [`transpose_octaves`].
pub use self::transpose_octaves as tr_octaves;

/// A function comparing two notes, used to sort collections of notes.
pub type NoteComparator = fn(&Note, &Note) -> Ordering;

/// Comparator ordering notes from lowest to highest pitch.
///
/// ```rust
/// use tonal_rs::note;
/// use std::cmp::Ordering;
/// let c4 = note::get("C4").unwrap();
/// let g4 = note::get("G4").unwrap();
/// assert_eq!(note::ascending(&c4, &g4), Ordering::Less);
/// ```
pub fn ascending(a: &Note, b: &Note) -> Ordering {
    a.height.cmp(&b.height)
}

/// Comparator ordering notes from highest to lowest pitch.
///
/// ```rust
/// use tonal_rs::note;
/// use std::cmp::Ordering;
/// let c4 = note::get("C4").unwrap();
/// let g4 = note::get("G4").unwrap();
/// assert_eq!(note::descending(&c4, &g4), Ordering::Greater);
/// ```
pub fn descending(a: &Note, b: &Note) -> Ordering {
    b.height.cmp(&a.height)
}

fn only_notes<T: IntoNote + Clone>(vec: &[T]) -> Vec<Note> {
    vec.iter().filter_map(|n| n.clone().into_note()).collect()
}

/// Get note names from a collection, filtering out invalid ones.
///
/// With `None`, returns the seven natural note names. Invalid names are dropped.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::names::<&str>(None), ["C", "D", "E", "F", "G", "A", "B"]);
/// assert_eq!(note::names(Some(&["fx", "bb", "x"][..])), ["F##", "Bb"]);
/// ```
pub fn names<T: IntoNote + Clone>(vec: Option<&[T]>) -> Vec<String> {
    let Some(vec) = vec else {
        return NAMES.iter().map(|&c| String::from(c)).collect();
    };
    only_notes(vec).into_iter().map(|n| n.name).collect()
}

/// Get sorted note names, filtering out invalid ones.
///
/// Sorts with the given comparator, defaulting to [`ascending`]. Invalid notes
/// are removed, so the result may be shorter (or empty) than the input.
///
/// ```rust
/// use tonal_rs::note;
/// let notes = ["c2", "c5", "c1", "c0", "c"];
/// assert_eq!(note::sorted_names(&notes, None), ["C", "C0", "C1", "C2", "C5"]);
/// ```
pub fn sorted_names<T: IntoNote + Clone>(
    notes: &[T],
    comparator: Option<NoteComparator>,
) -> Vec<String> {
    let comparator = comparator.unwrap_or(ascending);
    let mut notes = only_notes(notes);
    notes.sort_by(comparator);
    notes.into_iter().map(|n| n.name).collect()
}

/// Get sorted note names with adjacent duplicates removed.
///
/// Like [`sorted_names`] with the [`ascending`] comparator, but consecutive
/// equal names are collapsed.
///
/// ```rust
/// use tonal_rs::note;
/// let notes = ["c2", "c2", "b", "c", "c3"];
/// assert_eq!(note::sorted_uniq_names(&notes), ["C", "B", "C2", "C3"]);
/// ```
pub fn sorted_uniq_names<T: IntoNote + Clone>(notes: &[T]) -> Vec<String> {
    let mut names = sorted_names(notes, Some(ascending));
    names.dedup();
    names
}

/// Simplify a note, removing redundant accidentals (double sharps/flats).
///
/// Returns an empty string for invalid input.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::simplify("C##"), "D");
/// assert_eq!(note::simplify("B#4"), "C5");
/// assert_eq!(note::simplify("x"), "");
/// ```
pub fn simplify<T: IntoNote>(note: T) -> String {
    let Some(note) = note.into_note() else {
        return String::new();
    };

    let midi = note.midi.unwrap_or(note.chroma);

    midi_to_note_name(
        midi,
        Some(ToNoteNameOptions {
            sharps: Some(note.alt > 0),
            pitch_class: Some(note.midi.is_none()),
        }),
    )
}

/// Get the enharmonic equivalent of a note.
///
/// With `dest = None`, returns the simplest enharmonic spelling. With a
/// destination pitch class, respells the note as that pitch class if they are
/// enharmonically equal, keeping the octave.
///
/// Returns an empty string if the notes are not enharmonically equal.
///
/// ```rust
/// use tonal_rs::note;
/// assert_eq!(note::enharmonic("C#", None), "Db");
/// assert_eq!(note::enharmonic("B2", Some("Cb")), "Cb3");
/// assert_eq!(note::enharmonic("F2", Some("Eb")), "");
/// ```
pub fn enharmonic(note: &str, dest: Option<&str>) -> String {
    let Some(src) = note.into_note() else {
        return String::new();
    };

    let dest = match dest {
        Some(d) => d.into_note(),
        None => midi_to_note_name(
            src.midi.unwrap_or(src.chroma),
            Some(ToNoteNameOptions {
                sharps: Some(src.alt < 0),
                pitch_class: Some(true),
            }),
        )
        .into_note(),
    };

    let Some(dest) = dest else {
        return String::new();
    };

    if dest.chroma != src.chroma {
        return String::new();
    }

    let Some(src_oct) = src.oct else {
        return dest.pc;
    };

    let src_chroma = src.chroma - src.alt;
    let dest_chroma = dest.chroma - dest.alt;
    let dest_oct_offset = if src_chroma > 11 || dest_chroma < 0 {
        -1
    } else if src_chroma < 0 || dest_chroma > 11 {
        1
    } else {
        0
    };
    let dest_oct = src_oct + dest_oct_offset;
    format!("{}{}", dest.pc, dest_oct)
}

#[cfg(test)]
mod tests {
    use super::*;

    // split a space-separated string into words — mirrors the TS `$` helper.
    fn words(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    // apply f to each space-separated name, collecting results.
    fn map_words<F: Fn(&str) -> String>(s: &str, f: F) -> Vec<String> {
        s.split(' ').map(f).collect()
    }

    #[test]
    fn test_get() {
        let n = get("C4").unwrap();
        assert_eq!(n.parts().name, "C4");
        assert_eq!(n.parts().pc, "C");
        assert_eq!(n.parts().letter, 'C');
        assert_eq!(n.parts().acc, "");
        assert_eq!(n.parts().alt, 0);
        assert_eq!(n.parts().oct, Some(4));
        assert_eq!(n.parts().chroma, 0);
        assert_eq!(n.parts().height, 60);
        assert_eq!(n.parts().midi, Some(60));
        assert_eq!(n.parts().step, 0);
        assert!(get("x").is_none());
    }

    #[test]
    fn test_property_shorthands() {
        assert_eq!(name("db"), "Db");
        assert_eq!(pitch_class("Ax4"), "A##");
        assert_eq!(chroma("db4"), Some(1));
        assert_eq!(midi("db4"), Some(61));
        assert_eq!(freq("A4"), Some(440.0));
    }

    #[test]
    fn test_simplify() {
        assert_eq!(simplify("C#"), "C#");
        assert_eq!(simplify("C##"), "D");
        assert_eq!(simplify("C###"), "D#");
        assert_eq!(simplify("B#4"), "C5");
        assert_eq!(
            map_words("C## C### F##4 Gbbb5 B#4 Cbb4", |n| simplify(n)),
            words("D D# G4 E5 C5 Bb3")
        );
        assert_eq!(simplify("x"), "");
    }

    #[test]
    fn test_from_midi() {
        assert_eq!(from_midi(70), "Bb4");
        assert_eq!(
            [60, 61, 62].map(from_midi),
            ["C4", "Db4", "D4"].map(String::from)
        );
        assert_eq!(
            [60, 61, 62].map(from_midi_sharps),
            ["C4", "C#4", "D4"].map(String::from)
        );
    }

    #[test]
    fn test_names() {
        assert_eq!(names::<&str>(None), words("C D E F G A B"));
        // TS passes a mixed array with numbers/objects/null; here just note-ish
        // strings — the invalid ones are filtered out.
        assert_eq!(
            names(Some(&["fx", "bb", "nothing", "x"][..])),
            words("F## Bb")
        );
    }

    #[test]
    fn test_sorted_names() {
        assert_eq!(sorted_names(&words("c f g a b h j"), None), words("C F G A B"));
        assert_eq!(
            sorted_names(&words("c f g a b h j j h b a g f c"), None),
            words("C C F F G G A A B B")
        );
        assert_eq!(
            sorted_names(&words("c2 c5 c1 c0 c6 c"), None),
            words("C C0 C1 C2 C5 C6")
        );
        assert_eq!(
            sorted_names(&words("c2 c5 c1 c0 c6 c"), Some(descending)),
            words("C6 C5 C2 C1 C0 C")
        );
    }

    #[test]
    fn test_sorted_uniq_names() {
        assert_eq!(
            sorted_uniq_names(&words("a b c2 1p p2 c2 b c c3")),
            words("C A B C2 C3")
        );
    }

    #[test]
    fn test_transpose() {
        assert_eq!(transpose("A4", "3M"), "C#5");
        assert_eq!(tr("A4", "3M"), "C#5");
    }

    #[test]
    fn test_transpose_from() {
        assert_eq!(transpose_from("C4")("5P"), "G4");
        let f = transpose_from("C");
        assert_eq!(
            ["1P", "3M", "5P"].map(&f),
            ["C", "E", "G"].map(String::from)
        );
    }

    #[test]
    fn test_transpose_by() {
        assert_eq!(transpose_by("5P")("C4"), "G4");
        let f = transpose_by("5P");
        assert_eq!(
            ["C", "D", "E"].map(&f),
            ["G", "A", "B"].map(String::from)
        );
    }

    #[test]
    fn test_enharmonic() {
        assert_eq!(enharmonic("C#", None), "Db");
        assert_eq!(enharmonic("C##", None), "D");
        assert_eq!(enharmonic("C###", None), "Eb");
        assert_eq!(enharmonic("B#4", None), "C5");
        assert_eq!(
            map_words("C## C### F##4 Gbbb5 B#4 Cbb4", |n| enharmonic(n, None)),
            words("D Eb G4 E5 C5 A#3")
        );
        assert_eq!(enharmonic("x", None), "");
        assert_eq!(enharmonic("F2", Some("E#")), "E#2");
        assert_eq!(enharmonic("B2", Some("Cb")), "Cb3");
        assert_eq!(enharmonic("C2", Some("B#")), "B#1");
        assert_eq!(enharmonic("F2", Some("Eb")), "");
    }

    #[test]
    fn test_transpose_fifths() {
        assert_eq!(transpose_fifths("G4", 3), "E6");
        assert_eq!(transpose_fifths("G", 3), "E");
        let ns: Vec<String> = (0..=5).map(|n| transpose_fifths("C2", n)).collect();
        assert_eq!(ns, words("C2 G2 D3 A3 E4 B4"));
        let sharps: Vec<String> = (0..=6).map(|n| transpose_fifths("F#", n)).collect();
        assert_eq!(sharps, words("F# C# G# D# A# E# B#"));
        let flats: Vec<String> = [0, -1, -2, -3, -4, -5, -6]
            .iter()
            .map(|&n| transpose_fifths("Bb", n))
            .collect();
        assert_eq!(flats, words("Bb Eb Ab Db Gb Cb Fb"));
    }

    #[test]
    fn test_transpose_octaves() {
        let up: Vec<String> = (0..=4).map(|o| transpose_octaves("C4", o)).collect();
        assert_eq!(up.join(" "), "C4 C5 C6 C7 C8");
        let down: Vec<String> = [-1, -2, -3, -4, -5]
            .iter()
            .map(|&o| transpose_octaves("C4", o))
            .collect();
        assert_eq!(down.join(" "), "C3 C2 C1 C0 C-1");
    }

    #[test]
    fn test_from_freq() {
        assert_eq!(from_freq(440.0), "A4");
        assert_eq!(from_freq(444.0), "A4");
        assert_eq!(from_freq(470.0), "Bb4");
        assert_eq!(from_freq_sharps(470.0), "A#4");
        assert_eq!(from_freq(0.0), "");
        assert_eq!(from_freq(f32::NAN), "");
    }
}
