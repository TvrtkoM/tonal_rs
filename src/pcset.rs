//! Pitch-class sets and set-theory operations.
//!
//! A pitch-class set is the collection of pitch classes (0–11) present in a
//! group of notes or intervals, represented as a 12-bit chroma string
//! (`"101011010101"`) or its integer `set_num`. This module builds
//! [`Pcset`]s and provides set operations: [`is_subset_of`],
//! [`is_superset_of`], [`is_equal`], [`modes`], [`filter`] and more.

use std::{borrow::Cow, str::FromStr};

use crate::{
    collection::range,
    error::TonalParseError,
    note::transpose,
    pitch::Named,
    pitch_interval::{Interval, IntoInterval},
    pitch_note::{IntoNote, Note},
    regexes,
};

/// A pitch-class set with its chroma, set number and intervals.
///
/// Fields are private; read them through [`Pcset::parts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcset {
    pub(crate) name: String,
    pub(crate) set_num: i32,
    pub(crate) chroma: String,
    pub(crate) normalized: String,
    pub(crate) intervals: Vec<String>,
}

/// A borrowing, fully public view of a [`Pcset`]'s fields, returned by
/// [`Pcset::parts`].
#[derive(Debug, Clone, Copy)]
pub struct PcsetParts<'a> {
    /// The set name (usually empty).
    pub name: &'a str,
    /// The set number (chroma as an integer).
    pub set_num: i32,
    /// The 12-character chroma bitmap.
    pub chroma: &'a str,
    /// The rotation-invariant chroma.
    pub normalized: &'a str,
    /// The intervals from the lowest pitch class.
    pub intervals: &'a [String],
}

impl Pcset {
    /// A borrowing view of all fields.
    pub fn parts(&self) -> PcsetParts<'_> {
        PcsetParts {
            name: &self.name,
            set_num: self.set_num,
            chroma: &self.chroma,
            normalized: &self.normalized,
            intervals: &self.intervals,
        }
    }
}

impl Named for Pcset {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for Pcset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
impl TryFrom<&str> for Pcset {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if is_chroma(value) {
            chroma_to_pcset(value).ok_or(TonalParseError {
                entity: String::from("pcset"),
                input: String::from(value),
            })
        } else {
            Err(TonalParseError {
                entity: String::from("pcset"),
                input: String::from(value),
            })
        }
    }
}

impl FromStr for Pcset {
    type Err = TonalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Pcset::try_from(s)
    }
}

/// Conversion into a pitch-class set, implemented for chroma strings, set
/// numbers (`i32`), [`Pcset`] references, and slices of notes/intervals.
pub trait IntoPcset {
    /// The full [`Pcset`], or `None` if the input is not a valid set.
    fn into_pcset(self) -> Option<Pcset>;

    /// The set number, or `0` if invalid.
    fn pcset_num(self) -> i32
    where
        Self: Sized,
    {
        self.into_pcset().map_or(0, |p| p.set_num)
    }

    /// The chroma string, or `None` if invalid.
    fn pcset_chroma(self) -> Option<String>
    where
        Self: Sized,
    {
        Some(self.into_pcset()?.chroma)
    }
}

impl IntoPcset for &str {
    fn into_pcset(self) -> Option<Pcset> {
        self.try_into().ok()
    }

    fn pcset_num(self) -> i32 {
        if is_chroma(self) {
            chroma_to_number(self)
        } else {
            0
        }
    }

    fn pcset_chroma(self) -> Option<String> {
        if is_chroma(self) {
            Some(String::from(self))
        } else {
            None
        }
    }
}

impl IntoPcset for i32 {
    fn into_pcset(self) -> Option<Pcset> {
        if !is_pcset_num(self) {
            None
        } else {
            let chroma = set_num_to_chroma(self);
            chroma_to_pcset(&chroma)
        }
    }

    fn pcset_num(self) -> i32 {
        if is_pcset_num(self) { self } else { 0 }
    }

    fn pcset_chroma(self) -> Option<String> {
        if is_pcset_num(self) {
            Some(set_num_to_chroma(self))
        } else {
            None
        }
    }
}

impl IntoPcset for &Pcset {
    fn into_pcset(self) -> Option<Pcset> {
        Some(self.clone())
    }

    fn pcset_num(self) -> i32 {
        self.set_num
    }

    fn pcset_chroma(self) -> Option<String> {
        Some(self.chroma.clone())
    }
}

impl<T: PcChroma> IntoPcset for &[T] {
    fn into_pcset(self) -> Option<Pcset> {
        let chroma = list_to_chroma(self);
        chroma_to_pcset(&chroma)
    }

    fn pcset_num(self) -> i32 {
        chroma_to_number(&list_to_chroma(self))
    }

    fn pcset_chroma(self) -> Option<String> {
        Some(list_to_chroma(self))
    }
}

const EMPTY_CHROMA: &str = "000000000000";

fn set_num_to_chroma(num: i32) -> String {
    format!("{num:0>12b}")
}

fn chroma_to_number(chroma: &str) -> i32 {
    i32::from_str_radix(chroma, 2).unwrap_or(0)
}

fn is_pcset_num(set: i32) -> bool {
    (0..=4095).contains(&set)
}

/// Whether a string is a valid 12-character chroma bitmap (`"101011010101"`).
///
/// ```rust
/// use tonal_rs::pcset;
/// assert!(pcset::is_chroma("101011010101"));
/// assert!(!pcset::is_chroma("nope"));
/// ```
pub fn is_chroma(s: &str) -> bool {
    regexes::CHROMA.is_match(s)
}

const IVLS: [&str; 12] = [
    "1P", "2m", "2M", "3m", "3M", "4P", "5d", "5P", "6m", "6M", "7m", "7M",
];

fn chroma_to_intervals(chroma: &str) -> Vec<String> {
    chroma
        .chars()
        .zip(IVLS)
        .filter(|(c, _)| *c == '1')
        .map(|(_, ivl)| String::from(ivl))
        .collect()
}

/// Get a [`Pcset`] from a chroma string, set number, or note/interval list.
///
/// Returns `None` for invalid input.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert_eq!(pcset::get("101011010101").unwrap().parts().set_num, 2773);
/// assert!(pcset::get("nope").is_none());
/// ```
pub fn get<T: IntoPcset>(set: T) -> Option<Pcset> {
    set.into_pcset()
}

/// Get the set number of a pitch-class set, or `0` if invalid.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert_eq!(pcset::num("101011010101"), 2773);
/// assert_eq!(pcset::num("nope"), 0);
/// ```
pub fn num<T: IntoPcset>(set: T) -> i32 {
    set.pcset_num()
}

/// Get the chroma string of a pitch-class set.
///
/// Returns all-zeros (`"000000000000"`) for invalid input.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert_eq!(pcset::chroma(&["C", "E", "G"][..]), "100010010000");
/// assert_eq!(pcset::chroma("nope"), "000000000000");
/// ```
pub fn chroma<T: IntoPcset>(set: T) -> String {
    set.pcset_chroma()
        .unwrap_or_else(|| String::from(EMPTY_CHROMA))
}

/// Get the intervals of a pitch-class set (from its lowest pitch class).
///
/// Returns an empty vector for invalid input.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert_eq!(pcset::intervals("100010010000"), ["1P", "3M", "5P"]);
/// ```
pub fn intervals<T: IntoPcset>(set: T) -> Vec<String> {
    set.into_pcset().map(|p| p.intervals).unwrap_or_default()
}

/// Get the notes of a pitch-class set, spelled from C.
///
/// Returns an empty vector for invalid input.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert_eq!(pcset::notes("100010010000"), ["C", "E", "G"]);
/// ```
pub fn notes<T: IntoPcset>(set: T) -> Vec<String> {
    let Some(pcset) = set.into_pcset() else {
        return vec![];
    };

    pcset
        .intervals
        .into_iter()
        .map(|ivl| transpose("C", ivl.as_str()))
        .collect()
}

/// Get all 2048 chroma strings for sets that include the first pitch class
/// (set numbers 2048–4095).
pub fn chromas() -> Vec<String> {
    range(2048, 4095)
        .into_iter()
        .map(set_num_to_chroma)
        .collect()
}

/// Get the chromas of all rotations (modes) of a pitch-class set.
///
/// With `normalize = true`, only rotations that start on a set pitch class are
/// returned. Returns an empty vector for invalid input.
///
/// ```rust
/// use tonal_rs::pcset;
/// // the major scale has seven distinct rotations
/// assert_eq!(pcset::modes("101011010101", true).len(), 7);
/// ```
pub fn modes<T: IntoPcset>(set: T, normalize: bool) -> Vec<String> {
    let Some(pcs) = set.into_pcset() else {
        return vec![];
    };
    std::iter::successors(Some(pcs.set_num), |&v| Some(rotate_chroma(v)))
        .take(12)
        .filter(|&num| !normalize || num >= 2048)
        .map(set_num_to_chroma)
        .collect()
}

/// Whether two pitch-class sets contain the same pitch classes.
///
/// ```rust
/// use tonal_rs::pcset;
/// assert!(pcset::is_equal("101011010101", "101011010101"));
/// assert!(!pcset::is_equal("101011010101", "100010010000"));
/// ```
pub fn is_equal<T: IntoPcset>(s1: T, s2: T) -> bool {
    s1.pcset_num() == s2.pcset_num()
}

/// Build a predicate that is true when its argument is a proper subset of `set`.
///
/// `is_subset_of(larger)(smaller)` is true when all of `smaller`'s pitch
/// classes are contained in `larger` and the two differ.
///
/// ```rust
/// use tonal_rs::pcset;
/// let is_subset = pcset::is_subset_of("111111111111");
/// assert!(is_subset("101011010101"));
/// ```
pub fn is_subset_of<T: IntoPcset>(set: T) -> impl Fn(T) -> bool {
    let s = set.pcset_num();

    move |notes: T| -> bool {
        let o = notes.pcset_num();

        s != 0 && s != o && (o & s) == o
    }
}

/// Build a predicate that is true when its argument is a proper superset of
/// `set`.
///
/// `is_superset_of(smaller)(larger)` is true when `larger` contains all of
/// `smaller`'s pitch classes and the two differ.
///
/// ```rust
/// use tonal_rs::pcset;
/// let is_superset = pcset::is_superset_of("100010010000");
/// assert!(is_superset("101011010101"));
/// ```
pub fn is_superset_of<T: IntoPcset>(set: T) -> impl Fn(T) -> bool {
    let s = set.pcset_num();

    move |notes: T| -> bool {
        let o = notes.pcset_num();

        s != 0 && s != o && (o | s) == o
    }
}

/// Build a predicate testing whether a note belongs to the pitch-class set.
///
/// ```rust
/// use tonal_rs::pcset;
/// let in_c_major = pcset::is_note_included_in("101011010101");
/// assert!(in_c_major("E"));
/// assert!(!in_c_major("C#"));
/// ```
pub fn is_note_included_in<T>(set: T) -> impl Fn(&str) -> bool
where
    T: IntoPcset,
{
    let s = set.pcset_chroma();

    move |note: &str| match (&s, note.into_note()) {
        (Some(s), Some(n)) => s.as_bytes().get(n.chroma as usize) == Some(&b'1'),
        _ => false,
    }
}

/// Build a function that keeps only the notes belonging to the pitch-class set.
///
/// ```rust
/// use tonal_rs::pcset;
/// let keep = pcset::filter("101011010101");
/// assert_eq!(keep(&["C", "C#", "D"]), ["C", "D"]);
/// ```
pub fn filter<T: IntoPcset>(set: T) -> impl Fn(&[&str]) -> Vec<String> {
    let is_included = is_note_included_in(set);

    move |notes: &[&str]| {
        notes
            .iter()
            .copied()
            .filter(|n| is_included(n))
            .map(String::from)
            .collect()
    }
}

// one-position cyclic rotation of 12-bit chroma
fn rotate_chroma(v: i32) -> i32 {
    ((v << 1) | (v >> 11)) & 0xFFF
}

fn chroma_to_pcset(chroma: &str) -> Option<Pcset> {
    let set_num = chroma_to_number(chroma);
    if set_num == 0 {
        return None;
    }

    let normalized_num = std::iter::successors(Some(set_num), |&v| Some(rotate_chroma(v)))
        .take(12)
        .filter(|&n| n >= 2048)
        .min()?;

    let normalized = set_num_to_chroma(normalized_num);
    let intervals = chroma_to_intervals(chroma);

    Some(Pcset {
        chroma: String::from(chroma),
        name: String::new(),
        set_num,
        normalized,
        intervals,
    })
}

trait PcChroma {
    fn pc_chroma(&self) -> Option<i32>;
}

impl PcChroma for Note {
    fn pc_chroma(&self) -> Option<i32> {
        Some(self.chroma)
    }
}

impl PcChroma for Interval {
    fn pc_chroma(&self) -> Option<i32> {
        Some(self.chroma)
    }
}

impl PcChroma for &str {
    fn pc_chroma(&self) -> Option<i32> {
        (*self)
            .into_note()
            .map(|n| n.chroma)
            .or_else(|| (*self).into_interval().map(|i| i.chroma))
    }
}

fn list_to_chroma<T: PcChroma>(set: &[T]) -> String {
    if set.is_empty() {
        return String::from(EMPTY_CHROMA);
    }

    let binary = &mut [0u8; 12];

    for item in set {
        let Some(chroma) = item.pc_chroma() else {
            continue;
        };

        if let Some(slot) = binary.get_mut(chroma as usize) {
            *slot = 1
        }
    }

    binary.iter().map(|&n| (b'0' + n) as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // split a space-separated string into words — mirrors the TS `$` helper.
    fn words(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    #[test]
    fn test_get_from_note_list() {
        let p = get(&words("c d e")[..]).unwrap();
        assert_eq!(p.name, "");
        assert_eq!(p.set_num, 2688);
        assert_eq!(p.chroma, "101010000000");
        assert_eq!(p.normalized, "100000001010");
        assert_eq!(p.intervals, words("1P 2M 3M"));

        // order independent — same canonical set number
        assert_eq!(
            get(&words("d e c")[..]).unwrap().set_num,
            get(&words("c d e")[..]).unwrap().set_num,
        );
        assert!(get(&["not a note or interval"][..]).is_none());
        let empty: &[&str] = &[];
        assert!(get(empty).is_none());
    }

    #[test]
    fn test_get_from_pcset_number() {
        assert_eq!(get(2048).unwrap().set_num, get(&["C"][..]).unwrap().set_num);
        let set = get(&["D"][..]).unwrap();
        assert_eq!(get(set.set_num).unwrap().chroma, set.chroma);
    }

    #[test]
    fn test_num() {
        assert_eq!(num("000000000001"), 1);
        assert_eq!(num(&["B"][..]), 1);
        assert_eq!(num(&["Cb"][..]), 1);
        assert_eq!(num(&words("C E G")[..]), 2192);
        assert_eq!(num(&["C"][..]), 2048);
        assert_eq!(num("100000000000"), 2048);
        assert_eq!(num("111111111111"), 4095);
    }

    #[test]
    fn test_normalized() {
        let like_c = chroma(&["C"][..]); // "100000000000"
        for pc in ["c", "d", "e", "f", "g", "a", "b"] {
            assert_eq!(get(&[pc][..]).unwrap().normalized, like_c);
        }
        assert_eq!(
            get(&words("E F#")[..]).unwrap().normalized,
            get(&words("C D")[..]).unwrap().normalized
        );
    }

    #[test]
    fn test_chroma() {
        assert_eq!(chroma(&["C"][..]), "100000000000");
        assert_eq!(chroma(&["D"][..]), "001000000000");
        assert_eq!(chroma(&words("c d e")[..]), "101010000000");
        assert_eq!(chroma(&words("g g#4 a bb5")[..]), "000000011110");
        assert_eq!(
            chroma(&words("P1 M2 M3 P4 P5 M6 M7")[..]),
            chroma(&words("c d e f g a b")[..])
        );
        assert_eq!(chroma("101010101010"), "101010101010");
        assert_eq!(chroma(&words("one two")[..]), "000000000000");
        assert_eq!(chroma("A B C"), "000000000000");
    }

    #[test]
    fn test_chromas() {
        assert_eq!(chromas().len(), 2048);
        assert_eq!(chromas()[0], "100000000000");
        assert_eq!(chromas()[2047], "111111111111");
    }

    #[test]
    fn test_intervals() {
        assert_eq!(intervals("101010101010"), words("1P 2M 3M 5d 6m 7m"));
        assert!(intervals("1010").is_empty());
        assert_eq!(intervals(&words("C G B")[..]), words("1P 5P 7M"));
        assert_eq!(intervals(&words("D F A")[..]), words("2M 4P 6M"));
    }

    #[test]
    fn test_is_chroma() {
        assert_eq!(get("101010101010").unwrap().chroma, "101010101010");
        assert_eq!(chroma("1010101"), "000000000000");
        assert_eq!(chroma("blah"), "000000000000");
        assert_eq!(chroma("c d e"), "000000000000");
    }

    #[test]
    fn test_is_subset_of() {
        let is_in_c_major = is_subset_of(&["c4", "e6", "g"][..]);
        assert!(is_in_c_major(&["c2", "g7"][..]));
        assert!(is_in_c_major(&["c2", "e"][..]));
        assert!(!is_in_c_major(&["c2", "e3", "g4"][..]));
        assert!(!is_in_c_major(&["c2", "e3", "b5"][..]));
        assert!(is_subset_of(&["c", "d", "e"][..])(&["C", "D"][..]));
    }

    #[test]
    fn test_is_subset_of_with_chroma() {
        let is_subset = is_subset_of("101010101010");
        assert!(is_subset("101000000000"));
        assert!(!is_subset("111000000000"));
    }

    #[test]
    fn test_is_superset_of() {
        let extends_c_major = is_superset_of(&["c", "e", "g"][..]);
        assert!(extends_c_major(&["c2", "g3", "e4", "f5"][..]));
        assert!(!extends_c_major(&["e", "c", "g"][..]));
        assert!(!extends_c_major(&["c", "e", "f"][..]));
        assert!(is_superset_of(&["c", "d"][..])(&["c", "d", "e"][..]));
    }

    #[test]
    fn test_is_superset_of_with_chroma() {
        let is_superset = is_superset_of("101000000000");
        assert!(is_superset("101010101010"));
        assert!(!is_superset("110010101010"));
    }

    #[test]
    fn test_is_equal() {
        assert!(is_equal(
            &["c2", "d3", "e7", "f5"][..],
            &["c4", "c", "d5", "e6", "f1"][..]
        ));
        assert!(is_equal(&["c", "f"][..], &["c4", "c", "f1"][..]));
    }

    #[test]
    fn test_is_note_included_in() {
        let is_included_in_c = is_note_included_in(&["c", "d", "e"][..]);
        assert!(is_included_in_c("C4"));
        assert!(!is_included_in_c("C#4"));
    }

    #[test]
    fn test_filter() {
        let in_c_major = filter(&["c", "d", "e"][..]);
        assert_eq!(
            in_c_major(&["c2", "c#2", "d2", "c3", "c#3", "d3"][..]),
            words("c2 d2 c3 d3")
        );
        assert_eq!(
            filter(&["c"][..])(&["c2", "c#2", "d2", "c3", "c#3", "d3"][..]),
            words("c2 c3")
        );
    }

    #[test]
    fn test_notes() {
        assert_eq!(notes(&words("c d e f g a b")[..]), words("C D E F G A B"));
        assert_eq!(notes(&words("b a g f e d c")[..]), words("C D E F G A B"));
        assert_eq!(
            notes(&words("D3 A3 Bb3 C4 D4 E4 F4 G4 A4")[..]),
            words("C D E F G A Bb")
        );
        assert_eq!(notes("101011010110"), words("C D E F G A Bb"));
        assert!(notes(&["blah", "x"][..]).is_empty());
    }

    #[test]
    fn test_modes() {
        assert_eq!(
            modes(&words("c d e f g a b")[..], true),
            words(
                "101011010101 101101010110 110101011010 101010110101 \
                 101011010110 101101011010 110101101010"
            )
        );
        assert_eq!(
            modes(&words("c d e f g a b")[..], false),
            words(
                "101011010101 010110101011 101101010110 011010101101 \
                 110101011010 101010110101 010101101011 101011010110 \
                 010110101101 101101011010 011010110101 110101101010"
            )
        );
        assert!(modes(&["blah", "bleh"][..], true).is_empty());
    }
}
