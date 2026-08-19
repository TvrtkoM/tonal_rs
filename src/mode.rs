use std::sync::LazyLock;

use crate::collection;
use crate::interval;
use crate::note::transpose;
use crate::scale_type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mode {
    pub(crate) name: String,
    pub(crate) mode_num: i32,
    pub(crate) set_num: i32,
    pub(crate) chroma: String,
    pub(crate) normalized: String,
    pub(crate) alt: i32,
    pub(crate) triad: String,
    pub(crate) seventh: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) intervals: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ModeParts<'a> {
    pub name: &'a str,
    pub mode_num: i32,
    pub set_num: i32,
    pub chroma: &'a str,
    pub normalized: &'a str,
    pub alt: i32,
    pub triad: &'a str,
    pub seventh: &'a str,
    pub aliases: &'a [String],
    pub intervals: &'a [String],
}

impl Mode {
    pub fn parts(&self) -> ModeParts<'_> {
        ModeParts {
            name: &self.name,
            mode_num: self.mode_num,
            set_num: self.set_num,
            chroma: &self.chroma,
            normalized: &self.normalized,
            alt: self.alt,
            triad: &self.triad,
            seventh: &self.seventh,
            aliases: &self.aliases,
            intervals: &self.intervals,
        }
    }
}

#[rustfmt::skip]
const MODES: &[(i32, i32, i32, &str, &str, &str, &str)] = &[
    (0, 2773,  0, "ionian",     "",    "Maj7",  "major"),
    (1, 2902,  2, "dorian",     "m",   "m7",    ""),
    (2, 3418,  4, "phrygian",   "m",   "m7",    ""),
    (3, 2741, -1, "lydian",     "",    "Maj7",  ""),
    (4, 2774,  1, "mixolydian", "",    "7",     ""),
    (5, 2906,  3, "aeolian",    "m",   "m7",    "minor"),
    (6, 3434,  5, "locrian",    "dim", "m7b5",  ""),
];

static MODES_CACHE: LazyLock<Vec<Mode>> = LazyLock::new(|| MODES.iter().map(to_mode).collect());

fn to_mode(datum: &(i32, i32, i32, &str, &str, &str, &str)) -> Mode {
    let (mode_num, set_num, alt, name, triad, seventh, alias) = *datum;
    let aliases = if alias.is_empty() {
        vec![]
    } else {
        vec![alias.to_string()]
    };
    let chroma = format!("{set_num:b}");
    let intervals = scale_type::get(name)
        .map(|s| s.intervals.clone())
        .unwrap_or_default();

    Mode {
        name: name.to_string(),
        mode_num,
        set_num,
        normalized: chroma.clone(),
        chroma,
        alt,
        triad: triad.to_string(),
        seventh: seventh.to_string(),
        aliases,
        intervals,
    }
}

pub trait IntoModeName {
    fn into_mode_name(self) -> Option<String>;
}

impl IntoModeName for &str {
    fn into_mode_name(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoModeName for String {
    fn into_mode_name(self) -> Option<String> {
        Some(self)
    }
}

impl IntoModeName for &Mode {
    fn into_mode_name(self) -> Option<String> {
        Some(self.name.clone())
    }
}

pub fn get<N: IntoModeName>(name: N) -> Option<Mode> {
    let key = name.into_mode_name()?.to_lowercase();
    MODES_CACHE
        .iter()
        .find(|m| m.name == key || m.aliases.contains(&key))
        .cloned()
}

pub fn all() -> Vec<Mode> {
    MODES_CACHE.clone()
}

pub fn names() -> Vec<String> {
    MODES_CACHE.iter().map(|m| m.name.clone()).collect()
}

pub fn notes<N: IntoModeName>(mode_name: N, tonic: &str) -> Vec<String> {
    match get(mode_name) {
        Some(mode) => mode
            .intervals
            .iter()
            .map(|ivl| transpose(tonic, ivl.as_str()))
            .collect(),
        None => vec![],
    }
}

fn build_chords<N: IntoModeName>(column: &[&str], mode_name: N, tonic: &str) -> Vec<String> {
    let Some(mode) = get(mode_name) else {
        return vec![];
    };
    let rotated = collection::rotated(mode.mode_num, column);
    let tonics: Vec<String> = mode
        .intervals
        .iter()
        .map(|ivl| transpose(tonic, ivl.as_str()))
        .collect();

    rotated
        .iter()
        .enumerate()
        .map(|(i, chord)| format!("{}{chord}", tonics[i]))
        .collect()
}

pub fn triads<N: IntoModeName>(mode_name: N, tonic: &str) -> Vec<String> {
    let column: Vec<&str> = MODES.iter().map(|m| m.4).collect();
    build_chords(&column, mode_name, tonic)
}

pub fn seventh_chords<N: IntoModeName>(mode_name: N, tonic: &str) -> Vec<String> {
    let column: Vec<&str> = MODES.iter().map(|m| m.5).collect();
    build_chords(&column, mode_name, tonic)
}

pub fn distance<D: IntoModeName, S: IntoModeName>(destination: D, source: S) -> Option<String> {
    let from = get(source)?;
    let to = get(destination)?;
    let transposed = interval::transpose_fifths("1P", to.alt - from.alt)?;
    interval::simplify(transposed.as_str())
}

pub fn relative_tonic<D: IntoModeName, S: IntoModeName>(
    destination: D,
    source: S,
    tonic: &str,
) -> Option<String> {
    let dist = distance(destination, source)?;
    Some(transpose(tonic, dist.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties() {
        let m = get("ionian").unwrap();
        assert_eq!(m.mode_num, 0);
        assert_eq!(m.name, "ionian");
        assert_eq!(m.set_num, 2773);
        assert_eq!(m.chroma, "101011010101");
        assert_eq!(m.normalized, "101011010101");
        assert_eq!(m.alt, 0);
        assert_eq!(m.triad, "");
        assert_eq!(m.seventh, "Maj7");
        assert_eq!(m.aliases, ["major"]);
        assert_eq!(m.intervals, ["1P", "2M", "3M", "4P", "5P", "6M", "7M"]);

        assert_eq!(get("major"), get("ionian"));
    }

    #[test]
    fn test_accept_mode_as_parameter() {
        let major = get("major").unwrap();
        assert_eq!(get(&major), get("major"));
        assert_eq!(get("Major"), get("major"));
    }

    #[test]
    fn test_name_is_case_independent() {
        assert_eq!(get("Dorian"), get("dorian"));
    }

    #[test]
    fn test_set_num() {
        let set_nums: Vec<i32> = names()
            .iter()
            .map(|n| get(n.as_str()).unwrap().set_num)
            .collect();
        assert_eq!(set_nums, [2773, 2902, 3418, 2741, 2774, 2906, 3434]);
    }

    #[test]
    fn test_alt() {
        let alts: Vec<i32> = names()
            .iter()
            .map(|n| get(n.as_str()).unwrap().alt)
            .collect();
        assert_eq!(alts, [0, 2, 4, -1, 1, 3, 5]);
    }

    #[test]
    fn test_triad() {
        let triads: Vec<String> = names()
            .iter()
            .map(|n| get(n.as_str()).unwrap().triad)
            .collect();
        assert_eq!(triads, ["", "m", "m", "", "", "m", "dim"]);
    }

    #[test]
    fn test_seventh() {
        let sevenths: Vec<String> = names()
            .iter()
            .map(|n| get(n.as_str()).unwrap().seventh)
            .collect();
        assert_eq!(sevenths, ["Maj7", "m7", "m7", "Maj7", "7", "m7", "m7b5"]);
    }

    #[test]
    fn test_aliases() {
        assert_eq!(get("major"), get("ionian"));
        assert_eq!(get("minor"), get("aeolian"));
    }

    #[test]
    fn test_names() {
        assert_eq!(
            names(),
            [
                "ionian",
                "dorian",
                "phrygian",
                "lydian",
                "mixolydian",
                "aeolian",
                "locrian",
            ]
        );
    }

    #[test]
    fn test_notes() {
        assert_eq!(notes("major", "C").join(" "), "C D E F G A B");
        assert_eq!(notes("dorian", "C").join(" "), "C D Eb F G A Bb");
        assert_eq!(notes("dorian", "F").join(" "), "F G Ab Bb C D Eb");
        assert_eq!(notes("lydian", "F").join(" "), "F G A B C D E");
        assert_eq!(notes("anything", "F").join(" "), "");
    }

    #[test]
    fn test_triads_fn() {
        assert_eq!(triads("minor", "C").join(" "), "Cm Ddim Eb Fm Gm Ab Bb");
        assert_eq!(
            triads("mixolydian", "Bb").join(" "),
            "Bb Cm Ddim Eb Fm Gm Ab"
        );
    }

    #[test]
    fn test_seventh_chords() {
        assert_eq!(
            seventh_chords("major", "C#").join(" "),
            "C#Maj7 D#m7 E#m7 F#Maj7 G#7 A#m7 B#m7b5"
        );
        assert_eq!(
            seventh_chords("dorian", "G").join(" "),
            "Gm7 Am7 BbMaj7 C7 Dm7 Em7b5 FMaj7"
        );
    }

    #[test]
    fn test_relative_tonic() {
        assert_eq!(relative_tonic("major", "minor", "A"), Some("C".to_string()));
        assert_eq!(relative_tonic("major", "minor", "D"), Some("F".to_string()));
        assert_eq!(
            relative_tonic("minor", "dorian", "D"),
            Some("A".to_string())
        );
        assert_eq!(relative_tonic("nonsense", "dorian", "D"), None);
    }
}
