//! Voice-leading strategies for choosing between candidate voicings.
//!
//! A [`VoiceLeadingFunction`] picks the voicing that best follows a previous
//! one. [`top_note_diff`] chooses the candidate whose top note is closest to
//! the previous voicing's top note.

use crate::note;

/// A function that selects one voicing from candidates, given the previous
/// voicing.
pub type VoiceLeadingFunction = fn(&[Vec<String>], &[String]) -> Vec<String>;

/// Voice-leading by minimizing the movement of the top note.
///
/// Returns the candidate voicing whose highest note is closest (in MIDI
/// distance) to the top note of `last_voicing`.
///
/// ```rust
/// use tonal_rs::voice_leading::top_note_diff;
/// let voicings = vec![
///     vec!["F3".to_string(), "A3".to_string(), "C4".to_string(), "E4".to_string()],
///     vec!["C4".to_string(), "E4".to_string(), "F4".to_string(), "A4".to_string()],
/// ];
/// let last = vec!["C4".to_string(), "E4".to_string(), "G4".to_string(), "B4".to_string()];
/// assert_eq!(top_note_diff(&voicings, &last), ["C4", "E4", "F4", "A4"]);
/// ```
pub fn top_note_diff(voicings: &[Vec<String>], last_voicing: &[String]) -> Vec<String> {
    if last_voicing.is_empty() {
        return voicings.first().cloned().unwrap_or_default();
    }

    let top_note_midi = |voicing: &[String]| {
        note::midi(voicing.last().map(String::as_str).unwrap_or("")).unwrap_or(0)
    };

    let last_top = top_note_midi(last_voicing);

    voicings
        .iter()
        .min_by_key(|voicing| (last_top - top_note_midi(voicing)).abs())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voicing(notes: &[&str]) -> Vec<String> {
        notes.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_top_note_diff() {
        let voicings = [
            voicing(&["F3", "A3", "C4", "E4"]),
            voicing(&["C4", "E4", "F4", "A4"]),
        ];
        let last = voicing(&["C4", "E4", "G4", "B4"]);

        assert_eq!(top_note_diff(&voicings, &last), ["C4", "E4", "F4", "A4"]);
    }
}
