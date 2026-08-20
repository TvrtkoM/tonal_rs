//! Build and query chords.
//!
//! A chord pairs a chord type (see [`chord_type`](crate::chord_type)) with a
//! tonic, and optionally a bass note for slash/inverted chords. Use [`get`] to
//! build a [`Chord`] from a symbol like `"Cmaj7"` or `"Cmaj7/E"`, reading its
//! `notes`, `intervals`, `symbol` and related data. Other functions transpose
//! chords ([`transpose`]), map degrees to notes ([`degrees`], [`steps`]), and
//! find related chords and scales ([`extended`], [`reduced`], [`chord_scales`]).

use std::borrow::Cow;

use crate::chord_type::{ChordQuality, ChordType, all as chord_types, get as get_chord_type};
use crate::interval::subtract;
use crate::note::distance;
use crate::pcset::{is_subset_of, is_superset_of};
use crate::pitch::Named;
use crate::pitch_distance::{tonic_intervals_transposer, transpose as transpose_note};
use crate::pitch_note::{IntoNote, Note, tokenize_note};
use crate::scale_type::all as scale_types;

/// A tokenized chord name as `(tonic, type, bass)`.
pub type ChordNameTokens = (String, String, String); // tonic, type, bass

/// A chord: a chord type applied to a tonic (and optional bass), with its notes.
///
/// Fields are private; read them through [`Chord::parts`].
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub(crate) name: String,
    pub(crate) quality: ChordQuality,
    pub(crate) set_num: i32,
    pub(crate) chroma: &'static str,
    pub(crate) normalized: &'static str,
    pub(crate) intervals: Vec<String>,
    pub(crate) aliases: &'static [String],
    pub(crate) tonic: String,
    pub(crate) kind: &'static str,
    pub(crate) root: String,
    pub(crate) bass: String,
    pub(crate) root_degree: Option<i32>,
    pub(crate) symbol: String,
    pub(crate) notes: Vec<String>,
}

/// A borrowing, fully public view of a [`Chord`]'s fields, returned by
/// [`Chord::parts`].
#[derive(Debug, Clone, Copy)]
pub struct ChordParts<'a> {
    /// Full name, e.g. `"C major seventh"`.
    pub name: &'a str,
    /// Broad quality.
    pub quality: ChordQuality,
    /// Set number of the chord type.
    pub set_num: i32,
    /// Chroma bitmap.
    pub chroma: &'a str,
    /// Rotation-invariant chroma.
    pub normalized: &'a str,
    /// Intervals from the tonic (reordered for inversions).
    pub intervals: &'a [String],
    /// Chord symbols/aliases of the type.
    pub aliases: &'a [String],
    /// The tonic pitch class, or empty if none.
    pub tonic: &'a str,
    /// The chord-type name, e.g. `"major seventh"`.
    pub kind: &'a str,
    /// The root pitch class, when a bass note is a chord tone.
    pub root: &'a str,
    /// The bass pitch class, for slash chords (empty otherwise).
    pub bass: &'a str,
    /// The degree of the root within the chord, when inverted.
    pub root_degree: Option<i32>,
    /// The chord symbol, e.g. `"Cmaj7"`.
    pub symbol: &'a str,
    /// The chord notes (empty when there is no tonic).
    pub notes: &'a [String],
}

impl Named for Chord {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Chord {
    /// A borrowing view of all fields.
    pub fn parts(&self) -> ChordParts<'_> {
        ChordParts {
            name: self.name.as_str(),
            intervals: &self.intervals,
            quality: self.quality,
            set_num: self.set_num,
            chroma: self.chroma,
            normalized: self.normalized,
            aliases: self.aliases,
            tonic: self.tonic.as_str(),
            kind: self.kind,
            root: self.root.as_str(),
            bass: self.bass.as_str(),
            root_degree: self.root_degree,
            symbol: self.symbol.as_str(),
            notes: &self.notes,
        }
    }
}

fn tokenize_bass(note: &str, chord: &str) -> ChordNameTokens {
    let split: Vec<_> = chord.split("/").collect();

    if split.len() == 1 {
        return (note.to_string(), split[0].to_string(), String::new());
    }

    let (letter, acc, oct, kind) = tokenize_note(split[1]);

    if !letter.is_empty() && oct.is_empty() && kind.is_empty() {
        (
            note.to_string(),
            split[0].to_string(),
            format!("{letter}{acc}"),
        )
    } else {
        (note.to_string(), chord.to_string(), String::new())
    }
}

/// Split a chord name into `(tonic, type, bass)`.
///
/// ```rust
/// use tonal_rs::chord;
/// let (tonic, kind, bass) = chord::tokenize("Cmaj7/E");
/// assert_eq!(tonic, "C");
/// assert_eq!(kind, "maj7");
/// assert_eq!(bass, "E");
/// ```
pub fn tokenize(name: &str) -> ChordNameTokens {
    let (letter, acc, oct, kind) = tokenize_note(name);
    if letter.is_empty() {
        tokenize_bass("", name)
    } else if letter == "A" && kind == "ug" {
        tokenize_bass("", "aug")
    } else {
        tokenize_bass(
            format!("{letter}{acc}").as_str(),
            format!("{oct}{kind}").as_str(),
        )
    }
}

fn up_octave(interval: &str) -> String {
    let mut chars = interval.chars();
    let (Some(d), Some(q)) = (chars.next(), chars.next()) else {
        return interval.to_string();
    };
    match d.to_digit(10) {
        Some(num) => format!("{}{q}", num + 7),
        None => interval.to_string(),
    }
}

fn chord_name(
    chord_type: &ChordType,
    root: Option<&Note>,
    bass_pc: &str,
    tonic_pc: &str,
) -> String {
    let has_bass = !bass_pc.is_empty() && bass_pc != tonic_pc;
    let tonic_name = if !tonic_pc.is_empty() {
        format!("{} ", tonic_pc)
    } else {
        String::new()
    };

    let over = root.filter(|r| r.pc != tonic_pc).map(|r| r.pc.as_str());
    let bass_name = match over {
        Some(pc) => format!(" over {pc}"),
        None if has_bass => format!(" over {bass_pc}"),
        None => String::new(),
    };

    format!("{}{}{}", tonic_name, chord_type.name, bass_name)
}

fn chord_symbol(
    chord_type: &ChordType,
    type_name: &str,
    root: Option<&Note>,
    bass_pc: &str,
    tonic_pc: &str,
) -> String {
    let has_bass = !bass_pc.is_empty() && bass_pc != tonic_pc;
    let type_name = if chord_type.aliases.iter().any(|a| a == type_name) {
        type_name
    } else {
        chord_type.aliases.first().map(|a| a.as_str()).unwrap_or("")
    };

    let over = root.filter(|r| r.pc != tonic_pc).map(|r| r.pc.as_str());
    let bass_symbol = match over {
        Some(pc) => format!("/{pc}"),
        None if has_bass => format!("/{bass_pc}"),
        None => String::new(),
    };
    format!("{}{}{}", tonic_pc, type_name, bass_symbol)
}

fn process_intervals(
    intervals: &[String],
    root_degree: Option<usize>,
    has_bass: bool,
    bass_interval: &str,
) -> Vec<String> {
    let mut intervals = intervals.to_owned();

    if let Some(root_degree) = root_degree {
        let k = (root_degree - 1).min(intervals.len());
        for ivl in &mut intervals[..k] {
            *ivl = up_octave(ivl);
        }
        intervals.rotate_left(k);
    } else if has_bass && let Some(ivl) = subtract(bass_interval, "8P") {
        intervals.insert(0, ivl);
    }

    intervals
}

/// Build a [`Chord`] from an explicit type name, tonic and bass.
///
/// The low-level constructor behind [`get`]. `type_name` is a chord type or
/// symbol; `optional_tonic` and `optional_bass` are note names (or `None`).
/// Returns `None` for an unknown type or invalid note.
///
/// ```rust
/// use tonal_rs::chord;
/// assert_eq!(chord::get_chord("maj7", Some("C"), None).unwrap().parts().symbol, "Cmaj7");
/// assert!(chord::get_chord("nope", Some("C"), None).is_none());
/// ```
pub fn get_chord(
    type_name: &str,
    optional_tonic: Option<&str>,
    optional_bass: Option<&str>,
) -> Option<Chord> {
    let chord_type = get_chord_type(type_name);
    let tonic = optional_tonic.and_then(|s| s.into_note());
    let bass = optional_bass.and_then(|s| s.into_note());

    let tonic_ref = tonic.as_ref();
    let bass_ref = bass.as_ref();

    let chord_type = chord_type?;

    if (optional_tonic.is_some_and(|s| !s.is_empty()) && tonic.is_none())
        || (optional_bass.is_some_and(|s| !s.is_empty()) && bass.is_none())
    {
        return None;
    }

    let tonic_pc = tonic_ref.map_or("", |t| &t.pc[..]);
    let bass_pc = bass_ref.map_or("", |b| &b.pc[..]);

    let bass_interval = distance(tonic_pc, bass_pc);
    let bass_index = chord_type
        .intervals
        .iter()
        .position(|i| *i == bass_interval);
    let root = if bass_index.is_some() { bass_ref } else { None };
    let root_degree = bass_index.map(|i| i + 1);
    let has_bass = !bass.is_none() && bass_pc != tonic_pc;

    let intervals = process_intervals(
        &chord_type.intervals,
        root_degree,
        has_bass,
        bass_interval.as_str(),
    );

    let notes = if !tonic_pc.is_empty() {
        intervals
            .iter()
            .map(|i| transpose_note(tonic_pc, i.as_str()))
            .collect()
    } else {
        vec![]
    };

    Some(Chord {
        name: chord_name(chord_type, root, bass_pc, tonic_pc),
        symbol: chord_symbol(chord_type, type_name, root, bass_pc, tonic_pc),
        quality: chord_type.quality,
        set_num: chord_type.set_num,
        chroma: chord_type.chroma.as_str(),
        normalized: chord_type.normalized.as_str(),
        intervals,
        aliases: &chord_type.aliases,
        tonic: tonic_pc.to_string(),
        kind: chord_type.name.as_str(),
        root: root.map_or(String::new(), |r| r.pc.clone()),
        bass: if has_bass {
            bass_pc.to_string()
        } else {
            String::new()
        },
        root_degree: root_degree.map(|d| d as i32),
        notes,
    })
}

/// Conversion into a [`Chord`], implemented for a chord symbol (`&str`) and for
/// tuples: `(tonic,)`, `(tonic, type)` and `(tonic, type, bass)`.
pub trait IntoChord {
    /// Parse or build a [`Chord`], returning `None` if it is not valid.
    fn into_chord(self) -> Option<Chord>;
}

impl IntoChord for &str {
    fn into_chord(self) -> Option<Chord> {
        if self.is_empty() {
            None
        } else {
            let (tonic, kind, bass) = tokenize(self);
            let chord = get_chord(kind.as_str(), Some(tonic.as_str()), Some(bass.as_str()));
            match chord {
                Some(chord) => Some(chord),
                None => get_chord(self, None, None),
            }
        }
    }
}

impl IntoChord for (&str,) {
    fn into_chord(self) -> Option<Chord> {
        get_chord("", Some(self.0), None)
    }
}

impl IntoChord for (&str, &str) {
    fn into_chord(self) -> Option<Chord> {
        get_chord(self.1, Some(self.0), None)
    }
}

impl IntoChord for (&str, &str, &str) {
    fn into_chord(self) -> Option<Chord> {
        get_chord(self.1, Some(self.0), Some(self.2))
    }
}

/// Get a [`Chord`] from a symbol or a `(tonic, type[, bass])` tuple.
///
/// Returns `None` if the chord is not valid.
///
/// ```rust
/// use tonal_rs::chord;
/// assert_eq!(chord::get("Cmaj7").unwrap().parts().notes.to_vec(), ["C", "E", "G", "B"]);
/// assert_eq!(chord::get(("C", "maj7")).unwrap().parts().symbol, "Cmaj7");
/// ```
pub fn get<C: IntoChord>(src: C) -> Option<Chord> {
    src.into_chord()
}

/// Alias of [`get`].
pub use self::get as chord;

/// Transpose a chord name by an interval.
///
/// If the chord has no tonic, the input is returned unchanged (not an empty
/// string).
///
/// ```rust
/// use tonal_rs::chord;
/// assert_eq!(chord::transpose("Cmaj7", "5P"), "Gmaj7");
/// assert_eq!(chord::transpose("maj7", "5P"), "maj7");
/// ```
pub fn transpose(chord_name: &str, interval: &str) -> String {
    let (tonic, kind, bass) = tokenize(chord_name);

    if tonic.is_empty() {
        return chord_name.to_string();
    }

    let tr = transpose_note(bass, interval);

    let slash = if tr.is_empty() {
        String::new()
    } else {
        format!("/{}", tr)
    };

    format!("{}{kind}{slash}", transpose_note(tonic, interval))
}

/// Get the names of scales that contain all the notes of the chord.
///
/// Returns an empty vector for an invalid chord.
pub fn chord_scales(name: &str) -> Vec<String> {
    let c = name.into_chord();
    let Some(chord) = c else {
        return vec![];
    };

    let is_chord_included = is_superset_of(chord.chroma);

    scale_types()
        .iter()
        .filter_map(|s| {
            if is_chord_included(s.chroma.as_str()) {
                Some(s.name.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Get the symbols of chords that are supersets of the given chord (contain all
/// its notes and at least one more).
///
/// Returns an empty vector for an invalid chord.
pub fn extended(name: &str) -> Vec<String> {
    let Some(chord) = name.into_chord() else {
        return vec![];
    };

    let is_superset = is_superset_of(chord.chroma);

    chord_types()
        .iter()
        .filter_map(|c| {
            if is_superset(c.chroma.as_str()) {
                Some(format!(
                    "{}{}",
                    chord.tonic,
                    c.aliases.first().map_or("", |a| a.as_str())
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Get the symbols of chords that are subsets of the given chord (all their
/// notes are contained in it).
///
/// Returns an empty vector for an invalid chord.
pub fn reduced(name: &str) -> Vec<String> {
    let Some(chord) = name.into_chord() else {
        return vec![];
    };

    let is_subset = is_subset_of(chord.chroma);

    chord_types()
        .iter()
        .filter_map(|c| {
            if is_subset(c.chroma.as_str()) {
                Some(format!(
                    "{}{}",
                    chord.tonic,
                    c.aliases.first().map_or("", |a| a.as_str())
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Get the notes of a chord, optionally overriding the tonic.
///
/// Returns an empty vector for an invalid chord or a chord with no tonic.
///
/// ```rust
/// use tonal_rs::chord;
/// assert_eq!(chord::notes("Cmaj7", None), ["C", "E", "G", "B"]);
/// assert_eq!(chord::notes("maj7", Some("D")), ["D", "F#", "A", "C#"]);
/// ```
pub fn notes<C: IntoChord>(chord_name: C, tonic: Option<&str>) -> Vec<String> {
    let Some(chord) = chord_name.into_chord() else {
        return vec![];
    };

    let note = tonic.filter(|s| !s.is_empty()).unwrap_or(&chord.tonic);

    if note.is_empty() {
        return vec![];
    }

    chord
        .intervals
        .iter()
        .map(|i| transpose_note(note, i.as_str()))
        .collect()
}

/// Build a function mapping a 1-based chord degree to a note.
///
/// Degree `0` returns an empty string; negative degrees descend.
///
/// ```rust
/// use tonal_rs::chord;
/// let d = chord::degrees("Cm", None);
/// assert_eq!(d(1), "C");
/// assert_eq!(d(2), "Eb");
/// assert_eq!(d(0), "");
/// ```
pub fn degrees<C: IntoChord>(chord_name: C, tonic: Option<&str>) -> impl Fn(i32) -> String {
    let (chord_tonic, intervals) = match chord_name.into_chord() {
        Some(c) => (c.tonic, c.intervals),
        _ => (String::new(), vec![]),
    };

    let note = tonic
        .filter(|s| !s.is_empty())
        .unwrap_or(chord_tonic.as_str());

    let transpose = tonic_intervals_transposer(intervals, Some(note.to_string()));

    move |degree| {
        if degree == 0 {
            String::new()
        } else {
            transpose(if degree > 0 { degree - 1 } else { degree })
        }
    }
}

/// Build a function mapping a 0-based chord step to a note.
///
/// Like [`degrees`] but 0-indexed: step `0` is the tonic.
///
/// ```rust
/// use tonal_rs::chord;
/// let s = chord::steps("Cm", None);
/// assert_eq!(s(0), "C");
/// assert_eq!(s(1), "Eb");
/// ```
pub fn steps<C: IntoChord>(chord_name: C, tonic: Option<&str>) -> impl Fn(i32) -> String {
    let (chord_tonic, intervals) = match chord_name.into_chord() {
        Some(c) => (c.tonic, c.intervals),
        _ => (String::new(), vec![]),
    };

    let note = tonic
        .filter(|s| !s.is_empty())
        .unwrap_or(chord_tonic.as_str());

    tonic_intervals_transposer(intervals, Some(note.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(a: &str, b: &str, c: &str) -> ChordNameTokens {
        (a.to_string(), b.to_string(), c.to_string())
    }

    fn w(s: &str) -> Vec<String> {
        s.split(' ').map(|x| x.to_string()).collect()
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("Cmaj7"), t("C", "maj7", ""));
        assert_eq!(tokenize("c7"), t("C", "7", ""));
        assert_eq!(tokenize("maj7"), t("", "maj7", ""));
        assert_eq!(tokenize("c#4 m7b5"), t("C#", "4m7b5", ""));
        assert_eq!(tokenize("c#4m7b5"), t("C#", "4m7b5", ""));
        assert_eq!(tokenize("Cb7b5"), t("Cb", "7b5", ""));
        assert_eq!(tokenize("Eb7add6"), t("Eb", "7add6", ""));
        assert_eq!(tokenize("Bb6b5"), t("Bb", "6b5", ""));
        assert_eq!(tokenize("aug"), t("", "aug", ""));
        assert_eq!(tokenize("C11"), t("C", "11", ""));
        assert_eq!(tokenize("C13no5"), t("C", "13no5", ""));
        assert_eq!(tokenize("C64"), t("C", "64", ""));
        assert_eq!(tokenize("C9"), t("C", "9", ""));
        assert_eq!(tokenize("C5"), t("C", "5", ""));
        assert_eq!(tokenize("C4"), t("C", "4", ""));
        assert_eq!(tokenize("C4|\n"), t("", "C4|\n", ""));
        assert_eq!(tokenize("Cmaj7/G"), t("C", "maj7", "G"));
        assert_eq!(tokenize("bb6/a##"), t("Bb", "6", "A##"));
        assert_eq!(tokenize("bb6/a##5"), t("Bb", "6/a##5", ""));
    }

    #[test]
    fn test_get_chord_properties() {
        let c = get_chord("maj7", Some("G4"), Some("G4")).unwrap();
        assert_eq!(c.name, "G major seventh");
        assert_eq!(c.symbol, "Gmaj7");
        assert_eq!(c.tonic, "G");
        assert_eq!(c.root, "G");
        assert_eq!(c.bass, "");
        assert_eq!(c.root_degree, Some(1));
        assert_eq!(c.set_num, 2193);
        assert_eq!(c.kind, "major seventh");
        assert_eq!(c.aliases, ["maj7", "Δ", "ma7", "M7", "Maj7", "^7"]);
        assert_eq!(c.chroma, "100010010001");
        assert_eq!(c.intervals, ["1P", "3M", "5P", "7M"]);
        assert_eq!(c.normalized, "100010010001");
        assert_eq!(c.notes, ["G", "B", "D", "F#"]);
        assert_eq!(c.quality, ChordQuality::Major);
    }

    #[test]
    fn test_first_inversion() {
        let c = get_chord("maj7", Some("G4"), Some("B4")).unwrap();
        assert_eq!(c.name, "G major seventh over B");
        assert_eq!(c.symbol, "Gmaj7/B");
        assert_eq!(c.tonic, "G");
        assert_eq!(c.root, "B");
        assert_eq!(c.bass, "B");
        assert_eq!(c.root_degree, Some(2));
        assert_eq!(c.intervals, ["3M", "5P", "7M", "8P"]);
        assert_eq!(c.notes, ["B", "D", "F#", "G"]);
    }

    #[test]
    fn test_first_inversion_without_octave() {
        let c = get_chord("maj7", Some("G"), Some("B")).unwrap();
        assert_eq!(c.name, "G major seventh over B");
        assert_eq!(c.symbol, "Gmaj7/B");
        assert_eq!(c.root, "B");
        assert_eq!(c.bass, "B");
        assert_eq!(c.root_degree, Some(2));
        assert_eq!(c.intervals, ["3M", "5P", "7M", "8P"]);
        assert_eq!(c.notes, ["B", "D", "F#", "G"]);
    }

    #[test]
    fn test_second_inversion() {
        let c = get_chord("maj7", Some("G4"), Some("D5")).unwrap();
        assert_eq!(c.name, "G major seventh over D");
        assert_eq!(c.symbol, "Gmaj7/D");
        assert_eq!(c.root, "D");
        assert_eq!(c.bass, "D");
        assert_eq!(c.root_degree, Some(3));
        assert_eq!(c.intervals, ["5P", "7M", "8P", "10M"]);
        assert_eq!(c.notes, ["D", "F#", "G", "B"]);
    }

    #[test]
    fn test_without_root() {
        let c = get_chord("M7", Some("G"), None).unwrap();
        assert_eq!(c.symbol, "GM7");
        assert_eq!(c.name, "G major seventh");
        assert_eq!(c.notes, ["G", "B", "D", "F#"]);
    }

    #[test]
    fn test_root_degrees() {
        assert_eq!(
            get_chord("maj7", Some("C"), Some("C")).unwrap().root_degree,
            Some(1)
        );
        assert_eq!(
            get_chord("maj7", Some("C"), Some("D")).unwrap().root_degree,
            None
        );
    }

    #[test]
    fn test_without_tonic_nor_root() {
        let c = get_chord("dim", None, None).unwrap();
        assert_eq!(c.symbol, "dim");
        assert_eq!(c.name, "diminished");
        assert_eq!(c.tonic, "");
        assert_eq!(c.root, "");
        assert_eq!(c.bass, "");
        assert_eq!(c.root_degree, None);
        assert_eq!(c.kind, "diminished");
        assert_eq!(c.aliases, ["dim", "°", "o"]);
        assert_eq!(c.chroma, "100100100000");
        assert_eq!(c.intervals, ["1P", "3m", "5d"]);
        assert_eq!(c.normalized, "100000100100");
        assert!(c.notes.is_empty());
        assert_eq!(c.quality, ChordQuality::Diminished);
        assert_eq!(c.set_num, 2336);
    }

    #[test]
    fn test_chord() {
        let c = get("Cmaj7").unwrap();
        assert_eq!(c.symbol, "Cmaj7");
        assert_eq!(c.name, "C major seventh");
        assert_eq!(c.tonic, "C");
        assert_eq!(c.root, "");
        assert_eq!(c.bass, "");
        assert_eq!(c.root_degree, None);
        assert_eq!(c.kind, "major seventh");
        assert_eq!(c.intervals, ["1P", "3M", "5P", "7M"]);
        assert_eq!(c.notes, ["C", "E", "G", "B"]);

        assert!(get("hello").is_none());
        assert!(get("").is_none());
        assert_eq!(get("C").unwrap().name, "C major");

        let cb = chord("C/Bb").unwrap();
        assert_eq!(cb.aliases, ["M", "^", "", "maj"]);
        assert_eq!(cb.bass, "Bb");
        assert_eq!(cb.chroma, "100010010000");
        assert_eq!(cb.intervals, ["-2M", "1P", "3M", "5P"]);
        assert_eq!(cb.name, "C major over Bb");
        assert_eq!(cb.notes, ["Bb", "C", "E", "G"]);
        assert_eq!(cb.root, "");
        assert_eq!(cb.root_degree, None);
        assert_eq!(cb.set_num, 2192);
        assert_eq!(cb.symbol, "C/Bb");
        assert_eq!(cb.tonic, "C");
        assert_eq!(cb.kind, "major");
    }

    #[test]
    fn test_chord_without_tonic() {
        assert_eq!(get("dim").unwrap().name, "diminished");
        assert_eq!(get("dim7").unwrap().name, "diminished seventh");
        assert_eq!(get("alt7").unwrap().name, "altered");
    }

    #[test]
    fn test_notes_property() {
        assert_eq!(get("Cmaj7").unwrap().notes, ["C", "E", "G", "B"]);
        assert_eq!(get("Eb7add6").unwrap().notes, ["Eb", "G", "Bb", "Db", "C"]);
        assert_eq!(get(("C4", "maj7")).unwrap().notes, ["C", "E", "G", "B"]);
        assert_eq!(get("C7").unwrap().notes, ["C", "E", "G", "Bb"]);
        assert_eq!(get("Cmaj7#5").unwrap().notes, ["C", "E", "G#", "B"]);
        assert!(get("blah").is_none());
    }

    #[test]
    fn test_notes_with_two_params() {
        assert_eq!(get(("C", "maj7")).unwrap().notes, ["C", "E", "G", "B"]);
        assert_eq!(get(("C6", "maj7")).unwrap().notes, ["C", "E", "G", "B"]);
    }

    #[test]
    fn test_augmented_chords() {
        assert_eq!(get("Caug").unwrap().notes, ["C", "E", "G#"]);
        assert_eq!(get(("C", "aug")).unwrap().notes, ["C", "E", "G#"]);
    }

    #[test]
    fn test_intervals() {
        assert_eq!(get("maj7").unwrap().intervals, ["1P", "3M", "5P", "7M"]);
        assert_eq!(get("Cmaj7").unwrap().intervals, ["1P", "3M", "5P", "7M"]);
        assert_eq!(get("aug").unwrap().intervals, ["1P", "3M", "5A"]);
        assert_eq!(
            get("C13no5").unwrap().intervals,
            ["1P", "3M", "7m", "9M", "13M"]
        );
        assert_eq!(get("major").unwrap().intervals, ["1P", "3M", "5P"]);
    }

    #[test]
    fn test_notes_fn() {
        assert_eq!(notes("Cmaj7", None), ["C", "E", "G", "B"]);
        assert!(notes("maj7", None).is_empty());
        assert_eq!(notes("maj7", Some("C4")), ["C4", "E4", "G4", "B4"]);
        assert_eq!(notes("Cmaj7", Some("C4")), ["C4", "E4", "G4", "B4"]);
        assert_eq!(notes("Cmaj7", Some("D4")), ["D4", "F#4", "A4", "C#5"]);
        assert_eq!(notes("C/Bb", Some("D4")), ["C4", "D4", "F#4", "A4"]);
    }

    #[test]
    fn test_existence() {
        assert_eq!(get("C6add9").unwrap().name, "C sixth added ninth");
        assert!(get("maj7").is_some());
        assert!(get("Cmaj7").is_some());
        assert!(get("mixolydian").is_none());
    }

    #[test]
    fn test_chord_scales() {
        assert_eq!(
            chord_scales("C7b9"),
            [
                "phrygian dominant",
                "flamenco",
                "spanish heptatonic",
                "half-whole diminished",
                "chromatic"
            ]
        );
    }

    #[test]
    fn test_transpose_chord_names() {
        assert_eq!(transpose("Eb7b9", "5P"), "Bb7b9");
        assert_eq!(transpose("7b9", "5P"), "7b9");
        assert_eq!(transpose("Cmaj7/B", "P5"), "Gmaj7/F#");
    }

    #[test]
    fn test_extended() {
        let mut got = extended("CMaj7");
        got.sort();
        let mut want = w("Cmaj#4 Cmaj7#9#11 Cmaj9 CM7add13 Cmaj13 Cmaj9#11 CM13#11 CM7b9");
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn test_reduced() {
        assert_eq!(reduced("CMaj7"), ["C5", "CM"]);
    }

    #[test]
    fn test_degrees_ascending() {
        let f = degrees("C", None);
        assert_eq!([1, 2, 3, 4].map(&f), ["C", "E", "G", "C"]);
        let f = degrees("CM", Some("C4"));
        assert_eq!([1, 2, 3, 4].map(&f), ["C4", "E4", "G4", "C5"]);
        let f = degrees("Cm6", Some("C4"));
        let got: Vec<String> = (1..=10).map(&f).collect();
        assert_eq!(got, w("C4 Eb4 G4 A4 C5 Eb5 G5 A5 C6 Eb6"));
        let f = degrees("C/B", None);
        assert_eq!([1, 2, 3, 4].map(&f), ["B", "C", "E", "G"]);
    }

    #[test]
    fn test_degrees_descending() {
        let f = degrees("C", None);
        assert_eq!([-1, -2, -3].map(&f), ["G", "E", "C"]);
        let f = degrees("CM", Some("C4"));
        assert_eq!([-1, -2, -3].map(&f), ["G3", "E3", "C3"]);
    }

    #[test]
    fn test_steps() {
        let f = steps("aug", Some("C4"));
        let got: Vec<String> = [-3, -2, -1, 0, 1, 2, 3].map(&f).to_vec();
        assert_eq!(got, w("C3 E3 G#3 C4 E4 G#4 C5"));
    }
}
