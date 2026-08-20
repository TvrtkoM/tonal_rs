//! The scale dictionary — a table of known scale types.
//!
//! Look up a [`ScaleType`] by name, alias, chroma or set number with [`get`],
//! or enumerate the dictionary with [`names`] and [`all`]. This is the
//! abstract counterpart to the [`scale`](crate::scale) module, which pairs a
//! scale type with a tonic to yield actual notes.

use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use crate::{pcset, pitch::Named};

/// A scale type: an abstract scale (its intervals) without a tonic.
#[derive(Debug, Clone)]
pub struct ScaleType {
    /// The canonical name, e.g. `"major"`.
    pub name: String,
    /// The set number (a 12-bit chroma read as an integer).
    pub set_num: i32,
    /// The 12-character chroma bitmap, e.g. `"101011010101"`.
    pub chroma: String,
    /// The chroma rotated so the lowest bit is set (rotation-invariant form).
    pub normalized: String,
    /// The intervals from the tonic, e.g. `["1P", "2M", "3M", …]`.
    pub intervals: Vec<String>,
    /// Alternative names, e.g. `["ionian"]` for `"major"`.
    pub aliases: Vec<String>,
}

impl Named for ScaleType {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for ScaleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// The scale dictionary is a fixed, read-only table built once from `DATA`.
struct Dictionary {
    scales: Vec<ScaleType>,
    // name / alias / chroma / set-num (stringified) -> index into `scales`
    index: HashMap<String, usize>,
}

static DICTIONARY: LazyLock<Dictionary> = LazyLock::new(build_dictionary);

fn build_dictionary() -> Dictionary {
    let mut scales: Vec<ScaleType> = Vec::with_capacity(DATA.len());
    let mut index: HashMap<String, usize> = HashMap::new();

    for &(ivls, name, aliases) in DATA {
        let intervals: Vec<String> = ivls.split(' ').map(String::from).collect();
        let refs: Vec<&str> = ivls.split(' ').collect();

        // every entry in DATA is a valid pitch class set, so this never fails
        let Some(pcs) = pcset::get(&refs[..]) else {
            continue;
        };

        // NOTE: keep the *declared* intervals, not the pcset-derived ones —
        // the pcset intervals are canonicalised from the chroma and would lose
        // the scale's intended enharmonic spelling (e.g. augmented's "2A").
        let scale = ScaleType {
            name: name.to_string(),
            set_num: pcs.set_num,
            chroma: pcs.chroma,
            normalized: pcs.normalized,
            intervals,
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
        };

        let i = scales.len();
        let _ = index.insert(scale.name.clone(), i);
        let _ = index.insert(scale.chroma.clone(), i);
        let _ = index.insert(scale.set_num.to_string(), i);
        for alias in &scale.aliases {
            let _ = index.insert(alias.clone(), i);
        }
        scales.push(scale);
    }

    Dictionary { scales, index }
}

/// Look up a scale type by name, alias, chroma, or set-number (as a string).
///
/// Returns `None` for an unknown scale type.
///
/// ```rust
/// use tonal_rs::scale_type;
/// assert_eq!(scale_type::get("major").unwrap().aliases, ["ionian"]);
/// assert!(scale_type::get("unknown").is_none());
/// ```
pub fn get(name: &str) -> Option<&'static ScaleType> {
    let i = *DICTIONARY.index.get(name)?;
    Some(&DICTIONARY.scales[i])
}

/// All scale-type names, in dictionary order.
pub fn names() -> Vec<&'static str> {
    DICTIONARY.scales.iter().map(|s| s.name.as_str()).collect()
}

/// All scale types, in dictionary order.
pub fn all() -> &'static [ScaleType] {
    &DICTIONARY.scales
}

/// Every key the dictionary is indexed by (names, aliases, chromas, set-nums).
/// Order is unspecified.
pub fn keys() -> Vec<&'static str> {
    DICTIONARY.index.keys().map(|k| k.as_str()).collect()
}

// Format: (intervals, name, aliases)
#[rustfmt::skip]
const DATA: &[(&str, &str, &[&str])] = &[
    // Basic scales
    ("1P 2M 3M 5P 6M", "major pentatonic", &["pentatonic"]),
    ("1P 2M 3M 4P 5P 6M 7M", "major", &["ionian"]),
    ("1P 2M 3m 4P 5P 6m 7m", "minor", &["aeolian"]),

    // Jazz common scales
    ("1P 2M 3m 3M 5P 6M", "major blues", &[]),
    ("1P 3m 4P 5d 5P 7m", "minor blues", &["blues"]),
    ("1P 2M 3m 4P 5P 6M 7M", "melodic minor", &[]),
    ("1P 2M 3m 4P 5P 6m 7M", "harmonic minor", &[]),
    ("1P 2M 3M 4P 5P 6M 7m 7M", "bebop", &[]),
    ("1P 2M 3m 4P 5d 6m 6M 7M", "diminished", &["whole-half diminished"]),

    // Modes
    ("1P 2M 3m 4P 5P 6M 7m", "dorian", &[]),
    ("1P 2M 3M 4A 5P 6M 7M", "lydian", &[]),
    ("1P 2M 3M 4P 5P 6M 7m", "mixolydian", &["dominant"]),
    ("1P 2m 3m 4P 5P 6m 7m", "phrygian", &[]),
    ("1P 2m 3m 4P 5d 6m 7m", "locrian", &[]),

    // 5-note scales
    ("1P 3M 4P 5P 7M", "ionian pentatonic", &[]),
    ("1P 3M 4P 5P 7m", "mixolydian pentatonic", &["indian"]),
    ("1P 2M 4P 5P 6M", "ritusen", &[]),
    ("1P 2M 4P 5P 7m", "egyptian", &[]),
    ("1P 3M 4P 5d 7m", "neapolitan major pentatonic", &[]),
    ("1P 3m 4P 5P 6m", "vietnamese 1", &[]),
    ("1P 2m 3m 5P 6m", "pelog", &[]),
    ("1P 2m 4P 5P 6m", "kumoijoshi", &[]),
    ("1P 2M 3m 5P 6m", "hirajoshi", &[]),
    ("1P 2m 4P 5d 7m", "iwato", &[]),
    ("1P 2m 4P 5P 7m", "in-sen", &[]),
    ("1P 3M 4A 5P 7M", "lydian pentatonic", &["chinese"]),
    ("1P 3m 4P 6m 7m", "malkos raga", &[]),
    ("1P 3m 4P 5d 7m", "locrian pentatonic", &["minor seven flat five pentatonic"]),
    ("1P 3m 4P 5P 7m", "minor pentatonic", &["vietnamese 2"]),
    ("1P 3m 4P 5P 6M", "minor six pentatonic", &[]),
    ("1P 2M 3m 5P 6M", "flat three pentatonic", &["kumoi"]),
    ("1P 2M 3M 5P 6m", "flat six pentatonic", &[]),
    ("1P 2m 3M 5P 6M", "scriabin", &[]),
    ("1P 3M 5d 6m 7m", "whole tone pentatonic", &[]),
    ("1P 3M 4A 5A 7M", "lydian #5p pentatonic", &[]),
    ("1P 3M 4A 5P 7m", "lydian dominant pentatonic", &[]),
    ("1P 3m 4P 5P 7M", "minor #7m pentatonic", &[]),
    ("1P 3m 4d 5d 7m", "super locrian pentatonic", &[]),

    // 6-note scales
    ("1P 2M 3m 4P 5P 7M", "minor hexatonic", &[]),
    ("1P 2A 3M 5P 5A 7M", "augmented", &[]),
    ("1P 2M 4P 5P 6M 7m", "piongio", &[]),
    ("1P 2m 3M 4A 6M 7m", "prometheus neapolitan", &[]),
    ("1P 2M 3M 4A 6M 7m", "prometheus", &[]),
    ("1P 2m 3M 5d 6m 7m", "mystery #1", &[]),
    ("1P 2m 3M 4P 5A 6M", "six tone symmetric", &[]),
    ("1P 2M 3M 4A 5A 6A", "whole tone", &["messiaen's mode #1"]),
    ("1P 2m 4P 4A 5P 7M", "messiaen's mode #5", &[]),

    // 7-note scales
    ("1P 2M 3M 4P 5d 6m 7m", "locrian major", &["arabian"]),
    ("1P 2m 3M 4A 5P 6m 7M", "double harmonic lydian", &[]),
    ("1P 2m 2A 3M 4A 6m 7m", "altered", &["super locrian", "diminished whole tone", "pomeroy"]),
    ("1P 2M 3m 4P 5d 6m 7m", "locrian #2", &["half-diminished", "aeolian b5"]),
    ("1P 2M 3M 4P 5P 6m 7m", "mixolydian b6", &["melodic minor fifth mode", "hindu"]),
    ("1P 2M 3M 4A 5P 6M 7m", "lydian dominant", &["lydian b7", "overtone"]),
    ("1P 2M 3M 4A 5A 6M 7M", "lydian augmented", &[]),
    ("1P 2m 3m 4P 5P 6M 7m", "dorian b2", &["phrygian #6", "melodic minor second mode"]),
    ("1P 2m 3m 4d 5d 6m 7d", "ultralocrian", &["superlocrian bb7", "superlocrian diminished"]),
    ("1P 2m 3m 4P 5d 6M 7m", "locrian 6", &["locrian natural 6", "locrian sharp 6"]),
    ("1P 2A 3M 4P 5P 5A 7M", "augmented heptatonic", &[]),
    ("1P 2M 3m 4A 5P 6M 7m", "dorian #4", &["ukrainian dorian", "romanian minor", "altered dorian"]),
    ("1P 2M 3m 4A 5P 6M 7M", "lydian diminished", &[]),
    ("1P 2M 3M 4A 5A 7m 7M", "leading whole tone", &[]),
    ("1P 2M 3M 4A 5P 6m 7m", "lydian minor", &[]),
    ("1P 2m 3M 4P 5P 6m 7m", "phrygian dominant", &["spanish", "phrygian major"]),
    ("1P 2m 3m 4P 5P 6m 7M", "balinese", &[]),
    ("1P 2m 3m 4P 5P 6M 7M", "neapolitan major", &[]),
    ("1P 2M 3M 4P 5P 6m 7M", "harmonic major", &[]),
    ("1P 2m 3M 4P 5P 6m 7M", "double harmonic major", &["gypsy"]),
    ("1P 2M 3m 4A 5P 6m 7M", "hungarian minor", &[]),
    ("1P 2A 3M 4A 5P 6M 7m", "hungarian major", &[]),
    ("1P 2m 3M 4P 5d 6M 7m", "oriental", &[]),
    ("1P 2m 3m 3M 4A 5P 7m", "flamenco", &[]),
    ("1P 2m 3m 4A 5P 6m 7M", "todi raga", &[]),
    ("1P 2m 3M 4P 5d 6m 7M", "persian", &[]),
    ("1P 2m 3M 5d 6m 7m 7M", "enigmatic", &[]),
    ("1P 2M 3M 4P 5A 6M 7M", "major augmented", &["major #5", "ionian augmented", "ionian #5"]),
    ("1P 2A 3M 4A 5P 6M 7M", "lydian #9", &[]),

    // 8-note scales
    ("1P 2m 2M 4P 4A 5P 6m 7M", "messiaen's mode #4", &[]),
    ("1P 2m 3M 4P 4A 5P 6m 7M", "purvi raga", &[]),
    ("1P 2m 3m 3M 4P 5P 6m 7m", "spanish heptatonic", &[]),
    ("1P 2M 3m 3M 4P 5P 6M 7m", "bebop minor", &[]),
    ("1P 2M 3M 4P 5P 5A 6M 7M", "bebop major", &[]),
    ("1P 2m 3m 4P 5d 5P 6m 7m", "bebop locrian", &[]),
    ("1P 2M 3m 4P 5P 6m 7m 7M", "minor bebop", &[]),
    ("1P 2M 3M 4P 5d 5P 6M 7M", "ichikosucho", &[]),
    ("1P 2M 3m 4P 5P 6m 6M 7M", "minor six diminished", &[]),
    ("1P 2m 3m 3M 4A 5P 6M 7m", "half-whole diminished", &["dominant diminished", "messiaen's mode #2"]),
    ("1P 3m 3M 4P 5P 6M 7m 7M", "kafi raga", &[]),
    ("1P 2M 3M 4P 4A 5A 6A 7M", "messiaen's mode #6", &[]),

    // 9-note scales
    ("1P 2M 3m 3M 4P 5d 5P 6M 7m", "composite blues", &[]),
    ("1P 2M 3m 3M 4A 5P 6m 7m 7M", "messiaen's mode #3", &[]),

    // 10-note scales
    ("1P 2m 2M 3m 4P 4A 5P 6m 6M 7M", "messiaen's mode #7", &[]),

    // 12-note scales
    ("1P 2m 2M 3m 3M 4P 5d 5P 6m 6M 7m 7M", "chromatic", &[]),
];

#[cfg(test)]
mod tests {
    use super::*;

    // mode names of a scale: the normalized-rotation chromas mapped back to
    // their dictionary names (mirrors tonal's `modes(...).map(get(...).name)`).
    fn mode_names(scale: &str) -> Vec<&'static str> {
        let base = get(scale).unwrap();
        let ivls: Vec<&str> = base.intervals.iter().map(|s| s.as_str()).collect();
        pcset::modes(&ivls[..], true)
            .iter()
            .map(|chroma| get(chroma).unwrap().name.as_str())
            .collect()
    }

    #[test]
    fn test_all_length_and_order() {
        assert_eq!(all().len(), 92);
        // dictionary order — first entry in DATA
        assert_eq!(all()[0].name, "major pentatonic");
    }

    #[test]
    fn test_get_major() {
        let major = get("major").unwrap();
        assert_eq!(major.name, "major");
        assert_eq!(major.set_num, 2773);
        assert_eq!(major.chroma, "101011010101");
        assert_eq!(major.normalized, "101010110101");
        assert_eq!(major.intervals, ["1P", "2M", "3M", "4P", "5P", "6M", "7M"]);
        assert_eq!(major.aliases, ["ionian"]);
    }

    #[test]
    fn test_get_by_alias_chroma_and_set_num() {
        let major_num = get("major").unwrap().set_num;
        // alias
        assert_eq!(get("ionian").unwrap().set_num, major_num);
        // chroma
        assert_eq!(get("101011010101").unwrap().set_num, major_num);
        // set-number as string
        assert_eq!(get("2773").unwrap().set_num, major_num);
    }

    #[test]
    fn test_unknown_is_none() {
        assert!(get("unknown").is_none());
    }

    #[test]
    fn test_major_modes() {
        assert_eq!(
            mode_names("major"),
            [
                "major",
                "dorian",
                "phrygian",
                "lydian",
                "mixolydian",
                "minor",
                "locrian",
            ]
        );
    }

    #[test]
    fn test_harmonic_minor_modes() {
        assert_eq!(
            mode_names("harmonic minor"),
            [
                "harmonic minor",
                "locrian 6",
                "major augmented",
                "dorian #4",
                "phrygian dominant",
                "lydian #9",
                "ultralocrian",
            ]
        );
    }

    #[test]
    fn test_melodic_minor_modes() {
        assert_eq!(
            mode_names("melodic minor"),
            [
                "melodic minor",
                "dorian b2",
                "lydian augmented",
                "lydian dominant",
                "mixolydian b6",
                "locrian #2",
                "altered",
            ]
        );
    }
}
