use crate::note;

pub type VoiceLeadingFunction = fn(&[Vec<String>], &[String]) -> Vec<String>;

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
