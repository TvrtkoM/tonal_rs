use std::{collections::HashMap, sync::LazyLock};

use crate::pcset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Augmented,
    Diminished,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ChordType {
    pub name: String,
    pub quality: ChordQuality,
    pub set_num: i32,
    pub chroma: String,
    pub normalized: String,
    pub intervals: Vec<String>,
    pub aliases: Vec<String>,
}

struct Dictionary {
    chords: Vec<ChordType>,
    index: HashMap<String, usize>,
}

static DICTIONARY: LazyLock<Dictionary> = LazyLock::new(build_dictionary);

fn get_quality(intervals: &[&str]) -> ChordQuality {
    let has = |i: &str| intervals.contains(&i);
    if has("5A") {
        ChordQuality::Augmented
    } else if has("3M") {
        ChordQuality::Major
    } else if has("5d") {
        ChordQuality::Diminished
    } else if has("3m") {
        ChordQuality::Minor
    } else {
        ChordQuality::Unknown
    }
}

fn build_dictionary() -> Dictionary {
    let mut chords: Vec<ChordType> = Vec::with_capacity(DATA.len());

    for &(ivls, name, aliases) in DATA {
        let refs: Vec<&str> = ivls.split(' ').collect();

        let Some(pcs) = pcset::get(&refs[..]) else {
            continue;
        };

        chords.push(ChordType {
            name: name.to_string(),
            quality: get_quality(&refs),
            set_num: pcs.set_num,
            chroma: pcs.chroma,
            normalized: pcs.normalized,
            intervals: refs.iter().map(|s| s.to_string()).collect(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
        });
    }

    let mut order: Vec<usize> = (0..chords.len()).collect();
    order.sort_by_key(|&i| chords[i].set_num);

    let mut sorted_pos = vec![0usize; chords.len()];
    for (pos, &orig) in order.iter().enumerate() {
        sorted_pos[orig] = pos;
    }

    let sorted: Vec<ChordType> = order.iter().map(|&i| chords[i].clone()).collect();

    let mut index: HashMap<String, usize> = HashMap::new();
    for (orig, chord) in chords.iter().enumerate() {
        let pos = sorted_pos[orig];
        if !chord.name.is_empty() {
            index.insert(chord.name.clone(), pos);
        }
        index.insert(chord.set_num.to_string(), pos);
        index.insert(chord.chroma.clone(), pos);
        for alias in &chord.aliases {
            index.insert(alias.clone(), pos);
        }
    }

    Dictionary {
        chords: sorted,
        index,
    }
}

pub fn get(name: &str) -> Option<&'static ChordType> {
    let i = *DICTIONARY.index.get(name)?;
    Some(&DICTIONARY.chords[i])
}

pub fn all() -> &'static [ChordType] {
    &DICTIONARY.chords
}

pub fn names() -> Vec<&'static str> {
    DICTIONARY
        .chords
        .iter()
        .map(|c| c.name.as_str())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn symbols() -> Vec<&'static str> {
    DICTIONARY
        .chords
        .iter()
        .filter_map(|c| c.aliases.first())
        .map(|a| a.as_str())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn keys() -> Vec<&'static str> {
    DICTIONARY.index.keys().map(|k| k.as_str()).collect()
}

#[rustfmt::skip]
const DATA: &[(&str, &str, &[&str])] = &[
    ("1P 3M 5P", "major", &["M", "^", "maj"]),
    ("1P 3M 5P 7M", "major seventh", &["maj7", "Δ", "ma7", "M7", "Maj7", "^7"]),
    ("1P 3M 5P 7M 9M", "major ninth", &["maj9", "Δ9", "^9"]),
    ("1P 3M 5P 7M 9M 13M", "major thirteenth", &["maj13", "Maj13", "^13"]),
    ("1P 3M 5P 6M", "sixth", &["6", "add6", "add13", "M6"]),
    ("1P 3M 5P 6M 9M", "sixth added ninth", &["6add9", "6/9", "69", "M69"]),
    ("1P 3M 6m 7M", "major seventh flat sixth", &["M7b6", "^7b6"]),
    ("1P 3M 5P 7M 11A", "major seventh sharp eleventh", &["maj#4", "Δ#4", "Δ#11", "M7#11", "^7#11", "maj7#11"]),
    ("1P 3m 5P", "minor", &["m", "min", "-"]),
    ("1P 3m 5P 7m", "minor seventh", &["m7", "min7", "mi7", "-7"]),
    ("1P 3m 5P 7M", "minor/major seventh", &["m/ma7", "m/maj7", "mM7", "mMaj7", "m/M7", "-Δ7", "mΔ", "-^7", "-maj7"]),
    ("1P 3m 5P 6M", "minor sixth", &["m6", "-6"]),
    ("1P 3m 5P 7m 9M", "minor ninth", &["m9", "-9"]),
    ("1P 3m 5P 7M 9M", "minor/major ninth", &["mM9", "mMaj9", "-^9"]),
    ("1P 3m 5P 7m 9M 11P", "minor eleventh", &["m11", "-11"]),
    ("1P 3m 5P 7m 9M 13M", "minor thirteenth", &["m13", "-13"]),
    ("1P 3m 5d", "diminished", &["dim", "°", "o"]),
    ("1P 3m 5d 7d", "diminished seventh", &["dim7", "°7", "o7"]),
    ("1P 3m 5d 7m", "half-diminished", &["m7b5", "ø", "-7b5", "h7", "h"]),
    ("1P 3M 5P 7m", "dominant seventh", &["7", "dom"]),
    ("1P 3M 5P 7m 9M", "dominant ninth", &["9"]),
    ("1P 3M 5P 7m 9M 13M", "dominant thirteenth", &["13"]),
    ("1P 3M 5P 7m 11A", "lydian dominant seventh", &["7#11", "7#4"]),
    ("1P 3M 5P 7m 9m", "dominant flat ninth", &["7b9"]),
    ("1P 3M 5P 7m 9A", "dominant sharp ninth", &["7#9"]),
    ("1P 3M 7m 9m", "altered", &["alt7"]),
    ("1P 4P 5P", "suspended fourth", &["sus4", "sus"]),
    ("1P 2M 5P", "suspended second", &["sus2"]),
    ("1P 4P 5P 7m", "suspended fourth seventh", &["7sus4", "7sus"]),
    ("1P 5P 7m 9M 11P", "eleventh", &["11"]),
    ("1P 4P 5P 7m 9m", "suspended fourth flat ninth", &["b9sus", "phryg", "7b9sus", "7b9sus4"]),
    ("1P 5P", "fifth", &["5"]),
    ("1P 3M 5A", "augmented", &["aug", "+", "+5", "^#5"]),
    ("1P 3m 5A", "minor augmented", &["m#5", "-#5", "m+"]),
    ("1P 3M 5A 7M", "augmented seventh", &["maj7#5", "maj7+5", "+maj7", "^7#5"]),
    ("1P 3M 5P 7M 9M 11A", "major sharp eleventh (lydian)", &["maj9#11", "Δ9#11", "^9#11"]),
    ("1P 2M 4P 5P", "", &["sus24", "sus4add9"]),
    ("1P 3M 5A 7M 9M", "", &["maj9#5", "Maj9#5"]),
    ("1P 3M 5A 7m", "", &["7#5", "+7", "7+", "7aug", "aug7"]),
    ("1P 3M 5A 7m 9A", "", &["7#5#9", "7#9#5", "7alt"]),
    ("1P 3M 5A 7m 9M", "", &["9#5", "9+"]),
    ("1P 3M 5A 7m 9M 11A", "", &["9#5#11"]),
    ("1P 3M 5A 7m 9m", "", &["7#5b9", "7b9#5"]),
    ("1P 3M 5A 7m 9m 11A", "", &["7#5b9#11"]),
    ("1P 3M 5A 9A", "", &["+add#9"]),
    ("1P 3M 5A 9M", "", &["M#5add9", "+add9"]),
    ("1P 3M 5P 6M 11A", "", &["M6#11", "M6b5", "6#11", "6b5"]),
    ("1P 3M 5P 6M 7M 9M", "", &["M7add13"]),
    ("1P 3M 5P 6M 9M 11A", "", &["69#11"]),
    ("1P 3m 5P 6M 9M", "", &["m69", "-69"]),
    ("1P 3M 5P 6m 7m", "", &["7b6"]),
    ("1P 3M 5P 7M 9A 11A", "", &["maj7#9#11"]),
    ("1P 3M 5P 7M 9M 11A 13M", "", &["M13#11", "maj13#11", "M13+4", "M13#4"]),
    ("1P 3M 5P 7M 9m", "", &["M7b9"]),
    ("1P 3M 5P 7m 11A 13m", "", &["7#11b13", "7b5b13"]),
    ("1P 3M 5P 7m 13M", "", &["7add6", "67", "7add13"]),
    ("1P 3M 5P 7m 9A 11A", "", &["7#9#11", "7b5#9", "7#9b5"]),
    ("1P 3M 5P 7m 9A 11A 13M", "", &["13#9#11"]),
    ("1P 3M 5P 7m 9A 11A 13m", "", &["7#9#11b13"]),
    ("1P 3M 5P 7m 9A 13M", "", &["13#9"]),
    ("1P 3M 5P 7m 9A 13m", "", &["7#9b13"]),
    ("1P 3M 5P 7m 9M 11A", "", &["9#11", "9+4", "9#4"]),
    ("1P 3M 5P 7m 9M 11A 13M", "", &["13#11", "13+4", "13#4"]),
    ("1P 3M 5P 7m 9M 11A 13m", "", &["9#11b13", "9b5b13"]),
    ("1P 3M 5P 7m 9m 11A", "", &["7b9#11", "7b5b9", "7b9b5"]),
    ("1P 3M 5P 7m 9m 11A 13M", "", &["13b9#11"]),
    ("1P 3M 5P 7m 9m 11A 13m", "", &["7b9b13#11", "7b9#11b13", "7b5b9b13"]),
    ("1P 3M 5P 7m 9m 13M", "", &["13b9"]),
    ("1P 3M 5P 7m 9m 13m", "", &["7b9b13"]),
    ("1P 3M 5P 7m 9m 9A", "", &["7b9#9"]),
    ("1P 3M 5P 9M", "", &["Madd9", "2", "add9", "add2"]),
    ("1P 3M 5P 9m", "", &["Maddb9"]),
    ("1P 3M 5d", "", &["Mb5"]),
    ("1P 3M 5d 6M 7m 9M", "", &["13b5"]),
    ("1P 3M 5d 7M", "", &["M7b5"]),
    ("1P 3M 5d 7M 9M", "", &["M9b5"]),
    ("1P 3M 5d 7m", "", &["7b5"]),
    ("1P 3M 5d 7m 9M", "", &["9b5"]),
    ("1P 3M 7m", "", &["7no5"]),
    ("1P 3M 7m 13m", "", &["7b13"]),
    ("1P 3M 7m 9M", "", &["9no5"]),
    ("1P 3M 7m 9M 13M", "", &["13no5"]),
    ("1P 3M 7m 9M 13m", "", &["9b13"]),
    ("1P 3m 4P 5P", "", &["madd4"]),
    ("1P 3m 5P 6m 7M", "", &["mMaj7b6"]),
    ("1P 3m 5P 6m 7M 9M", "", &["mMaj9b6"]),
    ("1P 3m 5P 7m 11P", "", &["m7add11", "m7add4"]),
    ("1P 3m 5P 9M", "", &["madd9"]),
    ("1P 3m 5d 6M 7M", "", &["o7M7"]),
    ("1P 3m 5d 7M", "", &["oM7"]),
    ("1P 3m 6m 7M", "", &["mb6M7"]),
    ("1P 3m 6m 7m", "", &["m7#5"]),
    ("1P 3m 6m 7m 9M", "", &["m9#5"]),
    ("1P 3m 5A 7m 9M 11P", "", &["m11A"]),
    ("1P 3m 6m 9m", "", &["mb6b9"]),
    ("1P 2M 3m 5d 7m", "", &["m9b5"]),
    ("1P 4P 5A 7M", "", &["M7#5sus4"]),
    ("1P 4P 5A 7M 9M", "", &["M9#5sus4"]),
    ("1P 4P 5A 7m", "", &["7#5sus4"]),
    ("1P 4P 5P 7M", "", &["M7sus4"]),
    ("1P 4P 5P 7M 9M", "", &["M9sus4"]),
    ("1P 4P 5P 7m 9M", "", &["9sus4", "9sus"]),
    ("1P 4P 5P 7m 9M 13M", "", &["13sus4", "13sus"]),
    ("1P 4P 5P 7m 9m 13m", "", &["7sus4b9b13", "7b9b13sus4"]),
    ("1P 4P 7m 10m", "", &["4", "quartal"]),
    ("1P 5P 7m 9m 11P", "", &["11b9"]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch_interval::interval;

    #[test]
    fn test_names() {
        assert_eq!(
            &names()[..5],
            [
                "fifth",
                "suspended fourth",
                "suspended fourth seventh",
                "augmented",
                "major seventh flat sixth",
            ]
        );
    }

    #[test]
    fn test_symbols() {
        assert_eq!(&symbols()[..3], ["5", "M7#5sus4", "7#5sus4"]);
    }

    #[test]
    fn test_all_length() {
        assert_eq!(all().len(), 106);
    }

    #[test]
    fn test_get_major() {
        let c = get("major").unwrap();
        assert_eq!(c.name, "major");
        assert_eq!(c.set_num, 2192);
        assert_eq!(c.quality, ChordQuality::Major);
        assert_eq!(c.intervals, ["1P", "3M", "5P"]);
        assert_eq!(c.aliases, ["M", "^", "maj"]);
        assert_eq!(c.chroma, "100010010000");
        assert_eq!(c.normalized, "100001000100");
    }

    #[test]
    fn test_get_by_alias_chroma_and_set_num() {
        let num = get("major").unwrap().set_num;
        assert_eq!(get("M").unwrap().set_num, num);
        assert_eq!(get("100010010000").unwrap().set_num, num);
        assert_eq!(get("2192").unwrap().set_num, num);
    }

    #[test]
    fn test_unknown_is_none() {
        assert!(get("unknown chord").is_none());
    }

    #[test]
    fn test_no_repeated_intervals() {
        let mut ivls: Vec<&str> = DATA.iter().map(|&(i, _, _)| i).collect();
        ivls.sort();
        for w in ivls.windows(2) {
            assert_ne!(w[0], w[1]);
        }
    }

    #[test]
    fn test_all_have_abbreviatures() {
        for &(_, _, aliases) in DATA {
            assert!(!aliases.is_empty());
            assert!(aliases.iter().all(|a| !a.trim().is_empty()));
        }
    }

    #[test]
    fn test_intervals_ascending() {
        for &(ivls, _, _) in DATA {
            let semis: Vec<i32> = ivls
                .split(' ')
                .map(|i| interval(i).map(|iv| iv.semitones).unwrap_or(0))
                .collect();
            for w in semis.windows(2) {
                assert!(w[0] < w[1], "intervals not ascending in {ivls:?}");
            }
        }
    }
}
