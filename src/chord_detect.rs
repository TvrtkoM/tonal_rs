use crate::{
    chord_type::{ChordType, all},
    pcset::{IntoPcset, modes},
    pitch_note::IntoNote,
};

#[derive(Debug, Clone, PartialEq)]
struct FoundChord {
    pub weight: f32,
    pub name: String,
}

struct NamedSet {
    pc_to_name: [Option<String>; 12],
}

impl NamedSet {
    pub fn from(notes: &[&str]) -> Self {
        let mut arr: [Option<String>; 12] = Default::default();
        for n in notes {
            let Some(note) = n.into_note() else {
                continue;
            };
            if let Some(slot) = arr.get_mut(note.chroma as usize) {
                let _ = slot.get_or_insert(note.name);
            }
        }
        Self { pc_to_name: arr }
    }

    pub fn get(&self, chroma: usize) -> Option<&String> {
        self.pc_to_name.get(chroma)?.as_ref()
    }
}

// 3m 000100000000
// 3M 000010000000
const ANY_THIRD: u16 = 384;

// 5P 000000010000
const PERFECT_FIFTH: u16 = 16;

// 5d 000000100000
// 5A 000000001000
const NON_PERFECT_FIFTH: u16 = 40;

const ANY_SEVENTH: u16 = 3;

fn has_any_third(chroma_number: u16) -> bool {
    (chroma_number & ANY_THIRD) != 0
}

fn has_perfect_fifth(chroma_number: u16) -> bool {
    (chroma_number & PERFECT_FIFTH) != 0
}

fn has_any_seventh(chroma_number: u16) -> bool {
    (chroma_number & ANY_SEVENTH) != 0
}

fn has_non_perfect_fifth(chroma_number: u16) -> bool {
    (chroma_number & NON_PERFECT_FIFTH) != 0
}

fn has_any_third_and_perfect_fifth_and_any_seventh(chord_type: &ChordType) -> bool {
    let chroma_num = chord_type.set_num as u16;
    has_any_third(chroma_num) && has_perfect_fifth(chroma_num) && has_any_seventh(chroma_num)
}

fn with_perfect_fifth(chroma: &str) -> Option<String> {
    let chroma_num = chroma.pcset_num() as u16;
    if has_non_perfect_fifth(chroma_num) {
        Some(chroma.to_string())
    } else {
        ((chroma_num | PERFECT_FIFTH) as i32).pcset_chroma()
    }
}

#[derive(Default, Clone, Copy)]
pub struct DetectOptions {
    pub assume_perfect_fifth: bool,
}

fn find_matches(notes: &[&str], weight: f32, options: DetectOptions) -> Vec<FoundChord> {
    let Some(tonic) = notes.first() else {
        return Vec::new();
    };
    let Some(tonic_chroma) = tonic.into_note().map(|n| n.chroma) else {
        return Vec::new();
    };
    let named_set = NamedSet::from(notes);
    let all_modes = modes(notes, false);

    let mut found: Vec<FoundChord> = Vec::new();

    let es = String::new();

    for (index, mode) in all_modes.iter().enumerate() {
        let mode_with_perfect_fifth = options
            .assume_perfect_fifth
            .then(|| with_perfect_fifth(mode))
            .flatten();
        let mwpf_ref = &mode_with_perfect_fifth;

        let base_note = named_set.get(index).unwrap_or(&es);

        let is_inversion = index != tonic_chroma as usize;

        let chord_types: Vec<_> = all()
            .iter()
            .filter(|&ct| {
                if let Some(mwpf) = mwpf_ref.as_ref()
                    && options.assume_perfect_fifth
                    && has_any_third_and_perfect_fifth_and_any_seventh(ct)
                {
                    ct.chroma == *mwpf
                } else {
                    ct.chroma == *mode
                }
            })
            .collect();

        for ct in chord_types {
            let chord_name = ct.aliases.first().unwrap_or(&es);

            if is_inversion {
                found.push(FoundChord {
                    weight: 0.5 * weight,
                    name: format!("{base_note}{chord_name}/{tonic}"),
                });
            } else {
                found.push(FoundChord {
                    weight,
                    name: format!("{base_note}{chord_name}"),
                })
            }
        }
    }

    found
}

pub fn detect(source: &[&str], options: DetectOptions) -> Vec<String> {
    let notes: Vec<_> = source
        .iter()
        .filter_map(|s| s.into_note().map(|n| n.pc))
        .collect();
    if notes.is_empty() {
        return Vec::new();
    }
    let notes_str: Vec<_> = notes.iter().map(|n| n.as_str()).collect();

    let mut found = find_matches(&notes_str, 1., options);

    found.sort_by(|a, b| b.weight.total_cmp(&a.weight));

    found.into_iter().map(|c| c.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect_default(source: &[&str]) -> Vec<String> {
        detect(source, DetectOptions::default())
    }

    fn detect_pf(source: &[&str], assume_perfect_fifth: bool) -> Vec<String> {
        detect(
            source,
            DetectOptions {
                assume_perfect_fifth,
            },
        )
    }

    #[test]
    fn test_detect() {
        assert_eq!(detect_default(&["D", "F#", "A", "C"]), ["D7"]);
        assert_eq!(detect_default(&["F#", "A", "C", "D"]), ["D7/F#"]);
        assert_eq!(detect_default(&["A", "C", "D", "F#"]), ["D7/A"]);
        assert_eq!(detect_default(&["E", "G#", "B", "C#"]), ["E6", "C#m7/E"]);
    }

    #[test]
    fn test_assume_perfect_fifth() {
        assert_eq!(detect_pf(&["D", "F", "C"], true), ["Dm7"]);
        assert_eq!(detect_pf(&["D", "F", "C"], false), Vec::<String>::new());
        assert_eq!(detect_pf(&["D", "F", "A", "C"], true), ["Dm7", "F6/D"]);
        assert_eq!(detect_pf(&["D", "F", "A", "C"], false), ["Dm7", "F6/D"]);
        assert_eq!(detect_pf(&["D", "F", "Ab", "C"], true), ["Dm7b5", "Fm6/D"]);
    }

    #[test]
    fn test_regression_detect_aug() {
        assert_eq!(
            detect_default(&["C", "E", "G#"]),
            ["Caug", "Eaug/C", "G#aug/C"]
        );
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(detect_default(&[]), Vec::<String>::new());
    }
}
