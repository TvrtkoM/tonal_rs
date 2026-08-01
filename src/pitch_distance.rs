use crate::{
    pitch::{Fifths, Named, Octaves, Pitch, PitchCoordinates},
    pitch_interval::{IntoInterval, coord_to_interval},
    pitch_note::IntoNote,
};

pub fn distance<F: IntoNote, T: IntoNote>(from_note: F, to_note: T) -> String {
    let (Some(from), Some(to)) = (from_note.into_note(), to_note.into_note()) else {
        return String::new();
    };

    let (from_fifth, from_oct, _) = (&from.coord).into();
    let (to_fifth, to_oct, _) = (&to.coord).into();

    let fifths = to_fifth - from_fifth;

    let oct = match (from_oct, to_oct) {
        (Some(fo), Some(to)) => to - fo,
        _ => -(fifths * 7).div_euclid(12),
    };

    let force_descending =
        to.height == from.height && to.midi.is_some() && from.oct == to.oct && from.step > to.step;

    let coord: PitchCoordinates = (fifths, Some(oct), None).into();

    coord_to_interval(&coord, force_descending)
        .map(|i| i.name)
        .unwrap_or_default()
}

pub fn transpose<N: IntoNote, I: IntoInterval>(note: N, interval: I) -> String {
    let interval = interval.into_interval();
    match interval {
        Some(i) => transpose_with_coord(note, i.coord()),
        None => String::new(),
    }
}

pub fn transpose_with_coord<N: IntoNote, C: Into<(Fifths, Octaves)>>(note: N, coord: C) -> String {
    let (fifths, octaves) = coord.into();
    let Some(note) = note.into_note() else {
        return String::new();
    };
    let (note_fifths, note_oct, _) = (&note.coord).into();
    let tr: PitchCoordinates = if let Some(note_oct) = note_oct {
        (note_fifths + fifths, Some(note_oct + octaves), None).into()
    } else {
        (note_fifths + fifths, None, None).into()
    };
    let pitch: Pitch = (&tr).into();
    if let Some(note) = (&pitch).into_note() {
        note.name()
    } else {
        String::new()
    }
}

pub(crate) fn tonic_intervals_transposer(
    intervals: &[&str],
    tonic: Option<&str>,
) -> impl Fn(i32) -> String {
    let len = intervals.len() as i32;
    move |normalized: i32| -> String {
        let Some(tonic) = tonic else {
            return String::new();
        };
        let index = if normalized < 0 {
            (len - (-normalized % len)) % len
        } else {
            normalized % len
        } as usize;
        let octaves = normalized.div_euclid(len);
        let root = transpose_with_coord(tonic, (0, octaves));
        transpose(root.as_str(), intervals[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // distance(from, n) for each space-separated name, rejoined.
    fn all_from(from: &str, names: &str) -> String {
        names
            .split(' ')
            .map(|n| distance(from, n))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn test_interval_between_notes() {
        assert_eq!(all_from("C3", "C3 e3 e4 c2 e2"), "1P 3M 10M -8P -6m");
    }

    #[test]
    fn test_unison_edge_case_243() {
        assert_eq!(distance("Db4", "C#5"), "7A");
        assert_eq!(distance("Db4", "C#4"), "-2d");
        assert_eq!(distance("Db", "C#"), "7A");
        assert_eq!(distance("C#", "Db"), "2d");
    }

    #[test]
    fn test_adjacent_octaves_428() {
        assert_eq!(distance("B#4", "C4"), "-7A");
        assert_eq!(distance("B#4", "C6"), "9d");
        assert_eq!(distance("B#4", "C5"), "2d");
        assert_eq!(distance("B##4", "C#5"), "2d");
        assert_eq!(distance("B#5", "C6"), "2d");
    }

    #[test]
    fn test_pitch_classes_always_ascending() {
        assert_eq!(distance("C", "D"), "2M");
        assert_eq!(all_from("C", "c d e f g a b"), "1P 2M 3M 4P 5P 6M 7M");
        assert_eq!(all_from("G", "c d e f g a b"), "4P 5P 6M 7m 1P 2M 3M");
    }

    #[test]
    fn test_pitch_class_distance_is_between_pitch_classes() {
        assert_eq!(distance("C", "C2"), "1P");
        assert_eq!(distance("C2", "C"), "1P");
    }

    #[test]
    fn test_notes_must_be_valid() {
        assert_eq!(distance("one", "two"), "");
    }

    // transpose is new here too; cover the index.ts doc examples.
    #[test]
    fn test_transpose() {
        assert_eq!(transpose("d3", "3M"), "F#3");
        assert_eq!(transpose("D", "3M"), "F#");
        let mapped: Vec<String> = ["C", "D", "E", "F", "G"]
            .iter()
            .map(|pc| transpose(*pc, "3M"))
            .collect();
        assert_eq!(mapped, ["E", "F#", "G#", "A", "B"]);
    }
}
