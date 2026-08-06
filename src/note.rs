use std::cmp::Ordering;

use crate::{
    midi::{ToNoteNameOptions, freq_to_midi, midi_to_note_name},
    pitch_distance::transpose_with_coord,
    pitch_note::{AsNote, Note},
};

const NAMES: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];

pub use crate::pitch_note::note as get;

pub fn name<T: AsNote>(note: T) -> &'static str {
    note.note().map(|n| n.name.as_str()).unwrap_or("")
}

pub fn pitch_class<T: AsNote>(note: T) -> &'static str {
    note.note().map(|n| n.pc.as_str()).unwrap_or("")
}

pub fn accidentals<T: AsNote>(note: T) -> &'static str {
    note.note().map(|n| n.acc.as_str()).unwrap_or("")
}

pub fn octave<T: AsNote>(note: T) -> Option<i32> {
    note.note()?.oct
}

pub fn midi<T: AsNote>(note: T) -> Option<i32> {
    note.note()?.midi
}

pub fn freq<T: AsNote>(note: T) -> Option<f32> {
    note.note()?.freq
}

pub fn chroma<T: AsNote>(note: T) -> Option<i32> {
    note.note().map(|n| n.chroma)
}

pub fn from_midi(midi: i32) -> String {
    midi_to_note_name(midi, None)
}

pub fn from_freq(freq: f32) -> String {
    let midi = freq_to_midi(freq);
    if !midi.is_finite() {
        return String::new();
    }
    midi_to_note_name(midi.round() as i32, None)
}

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

pub fn from_midi_sharps(midi: i32) -> String {
    midi_to_note_name(
        midi,
        Some(ToNoteNameOptions {
            sharps: Some(true),
            pitch_class: None,
        }),
    )
}

pub use crate::pitch_distance::distance;

pub use crate::pitch_distance::transpose;
pub use crate::pitch_distance::transpose as tr;

pub fn transpose_from(note: &str) -> impl Fn(&str) -> String {
    move |interval: &str| -> String { transpose(note, interval) }
}

pub use self::transpose_from as tr_from;

pub fn transpose_by(interval: &str) -> impl Fn(&str) -> String {
    move |note: &str| -> String { transpose(note, interval) }
}

pub use self::transpose_by as tr_by;

pub fn transpose_fifths(note_name: &str, fifths: i32) -> String {
    transpose_with_coord(note_name, (fifths, 0))
}

pub use self::transpose_fifths as tr_fifths;

pub fn transpose_octaves(note_name: &str, octaves: i32) -> String {
    transpose_with_coord(note_name, (0, octaves))
}

pub use self::transpose_octaves as tr_octaves;

pub type NoteComparator = fn(&Note, &Note) -> Ordering;

pub fn ascending(a: &Note, b: &Note) -> Ordering {
    a.height.cmp(&b.height)
}

pub fn descending(a: &Note, b: &Note) -> Ordering {
    b.height.cmp(&a.height)
}

fn only_notes<T: AsNote + Clone>(vec: &[T]) -> Vec<&'static Note> {
    vec.iter().filter_map(|n| n.clone().note()).collect()
}

pub fn names<T: AsNote + Clone>(vec: Option<&[T]>) -> Vec<&'static str> {
    let Some(vec) = vec else {
        return NAMES.to_vec();
    };
    only_notes(vec)
        .into_iter()
        .map(|n| n.name.as_str())
        .collect()
}

pub fn sorted_names<T: AsNote + Clone>(
    notes: &[T],
    comparator: Option<NoteComparator>,
) -> Vec<&'static str> {
    let comparator = comparator.unwrap_or(ascending);
    let mut notes = only_notes(notes);
    notes.sort_by(|a, b| comparator(a, b));
    notes.into_iter().map(|n| n.name.as_str()).collect()
}

pub fn sorted_uniq_names<T: AsNote + Clone>(notes: &[T]) -> Vec<&'static str> {
    let mut names = sorted_names(notes, Some(ascending));
    names.dedup();
    names
}

pub fn simplify<T: AsNote>(note: T) -> String {
    let Some(note) = note.note() else {
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

pub fn enharmonic(note: &str, dest: Option<&str>) -> String {
    let Some(src) = note.note() else {
        return String::new();
    };

    let dest = match dest {
        Some(d) => d.note(),
        None => midi_to_note_name(
            src.midi.unwrap_or(src.chroma),
            Some(ToNoteNameOptions {
                sharps: Some(src.alt < 0),
                pitch_class: Some(true),
            }),
        )
        .note(),
    };

    let Some(dest) = dest else {
        return String::new();
    };

    if dest.chroma != src.chroma {
        return String::new();
    }

    let Some(src_oct) = src.oct else {
        return dest.pc.clone();
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
        assert_eq!(
            names(Some(&["fx", "bb", "nothing", "x"][..])),
            words("F## Bb")
        );
    }

    #[test]
    fn test_sorted_names() {
        assert_eq!(
            sorted_names(&words("c f g a b h j"), None),
            words("C F G A B")
        );
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
        assert_eq!(["C", "D", "E"].map(&f), ["G", "A", "B"].map(String::from));
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
