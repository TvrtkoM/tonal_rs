use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use regex::Regex;

use crate::{
    collection::{range, rotated},
    error::TonalParseError,
    note::transpose,
    pitch_interval::{Interval, IntoInterval},
    pitch_note::{IntoNote, Note},
};

#[derive(Debug, Clone)]
pub struct Pcset {
    pub(crate) name: String,
    pub(crate) set_num: i32,
    pub(crate) chroma: String,
    pub(crate) normalized: String,
    pub(crate) intervals: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct PcsetParts<'a> {
    pub name: &'a str,
    pub set_num: i32,
    pub chroma: &'a str,
    pub normalized: &'a str,
    pub intervals: &'a [String],
}

impl Pcset {
    pub fn set_num(&self) -> i32 {
        self.set_num
    }

    pub fn chroma(&self) -> String {
        self.chroma.clone()
    }

    pub fn intervals(&self) -> Vec<String> {
        self.intervals.clone()
    }

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

impl TryFrom<&str> for Pcset {
    type Error = TonalParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if is_chroma(value) {
            chroma_to_pcset(value).cloned().ok_or(TonalParseError {
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

pub trait AsPcset {
    /// Resolve to the cached, program-lifetime `Pcset` for this input.
    fn pcset(self) -> Option<&'static Pcset>;

    fn pcset_num(self) -> i32
    where
        Self: Sized,
    {
        self.pcset().map_or(0, |p| p.set_num)
    }

    fn pcset_chroma(self) -> Option<&'static str>
    where
        Self: Sized,
    {
        Some(self.pcset()?.chroma.as_str())
    }
}

impl AsPcset for &str {
    fn pcset(self) -> Option<&'static Pcset> {
        if is_chroma(self) {
            chroma_to_pcset(self)
        } else {
            None
        }
    }
}

impl AsPcset for i32 {
    fn pcset(self) -> Option<&'static Pcset> {
        if !is_pcset_num(self) {
            None
        } else {
            let chroma = set_num_to_chroma(self);
            chroma_to_pcset(&chroma)
        }
    }
}

impl AsPcset for &Pcset {
    fn pcset(self) -> Option<&'static Pcset> {
        // A caller-owned `Pcset` isn't `'static`; look up the cached one by its
        // chroma so callers still get a program-lifetime reference.
        chroma_to_pcset(&self.chroma)
    }

    fn pcset_num(self) -> i32 {
        self.set_num
    }
}

impl<T: PcChroma> AsPcset for &[T] {
    fn pcset(self) -> Option<&'static Pcset> {
        let chroma = list_to_chroma(self);
        chroma_to_pcset(&chroma)
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

static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[01]{12}$").unwrap());

pub fn is_chroma(s: &str) -> bool {
    REGEX.is_match(s)
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

pub fn get<T: AsPcset>(set: T) -> Option<&'static Pcset> {
    set.pcset()
}

pub fn num<T: AsPcset>(set: T) -> i32 {
    set.pcset_num()
}

pub fn chroma<T: AsPcset>(set: T) -> &'static str {
    set.pcset_chroma().unwrap_or(EMPTY_CHROMA)
}

pub fn intervals<T: AsPcset>(set: T) -> &'static [String] {
    set.pcset().map(|p| p.intervals.as_slice()).unwrap_or(&[])
}

pub fn notes<T: AsPcset>(set: T) -> Vec<String> {
    let Some(pcset) = set.pcset() else {
        return vec![];
    };

    pcset
        .intervals
        .iter()
        .map(|ivl| transpose("C", ivl.as_str()))
        .collect()
}

pub fn chromas() -> Vec<String> {
    range(2048, 4095)
        .into_iter()
        .map(set_num_to_chroma)
        .collect()
}

pub fn modes<T: AsPcset>(set: T, normalize: bool) -> Vec<String> {
    let Some(pcs) = set.pcset() else {
        return vec![];
    };
    let binary = pcs.chroma.chars().collect::<Vec<char>>();
    (0..binary.len())
        .filter_map(|i| {
            let r = rotated(i as i32, &binary);
            if normalize && r[0] == '0' {
                None
            } else {
                Some(r.iter().collect())
            }
        })
        .collect()
}

pub fn is_equal<T: AsPcset>(s1: T, s2: T) -> bool {
    s1.pcset_num() == s2.pcset_num()
}

pub fn is_subset_of<T: AsPcset>(set: T) -> impl Fn(T) -> bool {
    let s = set.pcset_num();

    move |notes: T| -> bool {
        let o = notes.pcset_num();

        s != 0 && s != o && (o & s) == o
    }
}

pub fn is_superset_of<T: AsPcset>(set: T) -> impl Fn(T) -> bool {
    let s = set.pcset_num();

    move |notes: T| -> bool {
        let o = notes.pcset_num();

        s != 0 && s != o && (o | s) == o
    }
}

pub fn is_note_included_in<T>(set: T) -> impl Fn(&str) -> bool
where
    T: AsPcset,
{
    let s = set.pcset_chroma();

    move |note: &str| match (s, note.into_note()) {
        (Some(s), Some(n)) => s.as_bytes().get(n.chroma as usize) == Some(&b'1'),
        _ => false,
    }
}

pub fn filter<T: AsPcset>(set: T) -> impl Fn(&[&str]) -> Vec<String> {
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

fn chroma_rotations(chroma: &str) -> Vec<String> {
    let binary = chroma.chars().collect::<Vec<char>>();
    (0..binary.len())
        .map(|i| rotated(i as i32, &binary).into_iter().collect())
        .collect()
}

// Cached pcsets live for the whole program (a `static` map is never cleared),
// so each computed one is leaked via `Box::leak` and stored as a `&'static
// Pcset`. Returning a cached hit is then just a pointer copy — no clone.
static CACHE: LazyLock<Mutex<HashMap<String, &'static Pcset>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn chroma_to_pcset(chroma: &str) -> Option<&'static Pcset> {
    if let Some(cached) = CACHE.lock().unwrap().get(chroma).copied() {
        return Some(cached);
    }

    let pcset: &'static Pcset = Box::leak(Box::new(chroma_to_pcset_uncached(chroma)?));
    CACHE.lock().unwrap().insert(String::from(chroma), pcset);
    Some(pcset)
}

fn chroma_to_pcset_uncached(chroma: &str) -> Option<Pcset> {
    let set_num = chroma_to_number(chroma);
    if set_num == 0 {
        return None;
    }

    let normalized_num: i32 = chroma_rotations(chroma)
        .iter()
        .filter_map(|c| {
            let n = chroma_to_number(c);
            if n >= 2048 { Some(n) } else { None }
        })
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
            get(&words("d e c")[..]).unwrap().set_num(),
            get(&words("c d e")[..]).unwrap().set_num()
        );
        assert!(get(&["not a note or interval"][..]).is_none());
        let empty: &[&str] = &[];
        assert!(get(empty).is_none());
    }

    #[test]
    fn test_get_from_pcset_number() {
        assert_eq!(
            get(2048).unwrap().set_num(),
            get(&["C"][..]).unwrap().set_num()
        );
        let set = get(&["D"][..]).unwrap();
        assert_eq!(get(set.set_num()).unwrap().chroma(), set.chroma());
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
        assert_eq!(
            intervals("101010101010"),
            words("1P 2M 3M 5d 6m 7m").as_slice()
        );
        assert!(intervals("1010").is_empty());
        assert_eq!(intervals(&words("C G B")[..]), words("1P 5P 7M").as_slice());
        assert_eq!(intervals(&words("D F A")[..]), words("2M 4P 6M").as_slice());
    }

    #[test]
    fn test_is_chroma() {
        assert_eq!(get("101010101010").unwrap().chroma(), "101010101010");
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
