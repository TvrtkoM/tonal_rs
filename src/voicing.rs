use crate::voice_leading::{VoiceLeadingFunction, top_note_diff};
use crate::voicing_dictionary::{self, VoicingDictionary};
use crate::{chord, interval, note, range};

pub const DEFAULT_RANGE: &[&str] = &["C3", "C5"];

pub fn search(chord: &str, range: &[&str], dictionary: &VoicingDictionary) -> Vec<Vec<String>> {
    let (tonic, symbol, _) = chord::tokenize(chord);

    let Some(sets) = voicing_dictionary::lookup(&symbol, dictionary) else {
        return vec![];
    };

    let notes_in_range = range::chromatic(range, None);
    let top = range.get(1).copied().unwrap_or("");

    let mut voiced: Vec<Vec<String>> = vec![];
    for set in &sets {
        let voicing: Vec<String> = set.split(' ').map(String::from).collect();
        let relative = relative_intervals(&voicing);
        let bottom_pc = note::transpose(tonic.as_str(), voicing[0].as_str());

        for start in start_notes(&notes_in_range, &bottom_pc, top, &relative) {
            voiced.push(render_voicing(&start, &relative));
        }
    }
    voiced
}

fn relative_intervals(voicing: &[String]) -> Vec<String> {
    voicing
        .iter()
        .map(|ivl| interval::subtract(ivl.as_str(), voicing[0].as_str()).unwrap_or_default())
        .collect()
}

fn start_notes(
    notes_in_range: &[String],
    bottom_pc: &str,
    top: &str,
    relative: &[String],
) -> Vec<String> {
    let bottom_chroma = note::chroma(bottom_pc);
    let last_interval = relative.last().map(String::as_str).unwrap_or("");
    let top_midi = note::midi(top).unwrap_or(0);

    notes_in_range
        .iter()
        .filter(|n| note::chroma(n.as_str()) == bottom_chroma)
        .filter(|n| {
            let top_note = note::transpose(n.as_str(), last_interval);
            note::midi(top_note.as_str()).unwrap_or(0) <= top_midi
        })
        .map(|n| note::enharmonic(n.as_str(), Some(bottom_pc)))
        .collect()
}

fn render_voicing(start: &str, relative: &[String]) -> Vec<String> {
    relative
        .iter()
        .map(|ivl| note::transpose(start, ivl.as_str()))
        .collect()
}

pub fn get(
    chord: &str,
    range: &[&str],
    dictionary: &VoicingDictionary,
    voice_leading: VoiceLeadingFunction,
    last_voicing: Option<&[String]>,
) -> Vec<String> {
    let voicings = search(chord, range, dictionary);

    match last_voicing {
        Some(last) if !last.is_empty() => voice_leading(&voicings, last),
        _ => voicings.into_iter().next().unwrap_or_default(),
    }
}

pub fn get_default(chord: &str) -> Vec<String> {
    get(
        chord,
        DEFAULT_RANGE,
        voicing_dictionary::all(),
        top_note_diff,
        None,
    )
}

pub fn sequence(
    chords: &[&str],
    range: &[&str],
    dictionary: &VoicingDictionary,
    voice_leading: VoiceLeadingFunction,
    last_voicing: Option<&[String]>,
) -> Vec<Vec<String>> {
    let mut voicings: Vec<Vec<String>> = vec![];
    let mut last: Option<Vec<String>> = last_voicing.map(<[String]>::to_vec);

    for chord in chords {
        let voicing = get(chord, range, dictionary, voice_leading, last.as_deref());
        last = Some(voicing.clone());
        voicings.push(voicing);
    }
    voicings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voicing_dictionary::{LEFTHAND, TRIADS};

    #[test]
    fn test_search_c_major_triad_inversions() {
        assert_eq!(
            search("C", &["C3", "C5"], TRIADS),
            [
                ["C3", "E3", "G3"],
                ["C4", "E4", "G4"],
                ["E3", "G3", "C4"],
                ["E4", "G4", "C5"],
                ["G3", "C4", "E4"],
            ]
        );
    }

    #[test]
    fn test_search_c_maj7_lefthand() {
        assert_eq!(
            search("C^7", &["E3", "D5"], LEFTHAND),
            [
                ["E3", "G3", "B3", "D4"],
                ["E4", "G4", "B4", "D5"],
                ["B3", "D4", "E4", "G4"],
            ]
        );
    }

    #[test]
    fn test_search_cminor7_custom() {
        const CUSTOM: &VoicingDictionary = &[("minor7", &["3m 5P 7m 9M", "7m 9M 10m 12P"])];
        assert_eq!(
            search("Cminor7", &["Eb3", "D5"], CUSTOM),
            [
                ["Eb3", "G3", "Bb3", "D4"],
                ["Eb4", "G4", "Bb4", "D5"],
                ["Bb3", "D4", "Eb4", "G4"],
            ]
        );
    }

    #[test]
    fn test_get() {
        assert_eq!(get_default("Dm7"), ["F3", "A3", "C4", "E4"]);

        assert_eq!(
            get("Dm7", &["F3", "A4"], LEFTHAND, top_note_diff, None),
            ["F3", "A3", "C4", "E4"]
        );

        let last: Vec<String> = ["C4", "E4", "G4", "B4"].iter().map(|s| s.to_string()).collect();
        assert_eq!(
            get("Dm7", &["F3", "A4"], LEFTHAND, top_note_diff, Some(&last)),
            ["C4", "E4", "F4", "A4"]
        );
    }

    #[test]
    fn test_sequence() {
        assert_eq!(
            sequence(&["C", "F", "G"], &["F3", "A4"], TRIADS, top_note_diff, None),
            [
                ["C4", "E4", "G4"],
                ["A3", "C4", "F4"],
                ["B3", "D4", "G4"],
            ]
        );
    }
}
