use crate::{
    collection::{range, rotated},
    midi::ToMidi,
    note::{enharmonic, from_midi, sorted_uniq_names},
    pcset::{chroma, is_chroma, is_subset_of, is_superset_of, modes},
    pitch_distance::{tonic_intervals_transposer, transpose},
    pitch_note::IntoNote,
    scale_type::{all as scale_types, get as get_scale_type, names as scale_type_names},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub(crate) name: String,
    pub(crate) set_num: i32,
    pub(crate) chroma: &'static str,
    pub(crate) normalized: &'static str,
    pub(crate) intervals: &'static [String],
    pub(crate) aliases: &'static [String],
    pub(crate) tonic: String,
    pub(crate) kind: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct ScaleParts<'a> {
    pub name: &'a str,
    pub set_num: i32,
    pub chroma: &'a str,
    pub normalized: &'a str,
    pub intervals: &'a [String],
    pub aliases: &'a [String],
    pub tonic: &'a str,
    pub kind: &'a str,
    pub notes: &'a [String],
}

impl Scale {
    pub fn parts(&self) -> ScaleParts<'_> {
        ScaleParts {
            name: &self.name,
            set_num: self.set_num,
            chroma: &self.chroma,
            normalized: &self.normalized,
            intervals: &self.intervals,
            aliases: &self.aliases,
            tonic: &self.tonic,
            kind: &self.kind,
            notes: &self.notes,
        }
    }
}

pub fn tokenize(name: &str) -> (String, String) {
    let head = match name.find(' ') {
        Some(i) => &name[..i],
        None => "",
    };

    let Some(tonic) = head.into_note() else {
        let Some(n) = name.into_note() else {
            return (String::new(), name.to_lowercase());
        };
        return (n.name, String::new());
    };

    let typ = name
        .chars()
        .skip(tonic.name.chars().count() + 1)
        .collect::<String>()
        .to_lowercase();

    (tonic.name, typ)
}

pub fn names() -> Vec<&'static str> {
    scale_type_names()
}

fn parse_tuple(src: (&str, &str)) -> Option<Scale> {
    let note = src.0.into_note();
    let tonic = note.map_or(String::new(), |n| n.name);
    let st = get_scale_type(src.1)?;

    let kind = &st.name;
    let notes = if !tonic.is_empty() {
        st.intervals
            .iter()
            .map(|i| transpose(tonic.as_str(), i.as_str()))
            .collect()
    } else {
        Vec::new()
    };

    let name = if tonic.is_empty() {
        kind.clone()
    } else {
        format!("{tonic} {kind}")
    };

    Some(Scale {
        name,
        normalized: st.normalized.as_str(),
        intervals: st.intervals.as_slice(),
        kind: kind.clone(),
        set_num: st.set_num,
        chroma: st.chroma.as_str(),
        aliases: &st.aliases,
        tonic,
        notes,
    })
}

fn parse_str(src: &str) -> Option<Scale> {
    let (t1, t2) = tokenize(src);
    parse_tuple((t1.as_str(), t2.as_str()))
}

pub trait IntoScale {
    fn into_scale(self) -> Option<Scale>;
}

impl IntoScale for &str {
    fn into_scale(self) -> Option<Scale> {
        parse_str(self)
    }
}

impl IntoScale for (&str, &str) {
    fn into_scale(self) -> Option<Scale> {
        parse_tuple(self)
    }
}

pub fn get<T: IntoScale>(src: T) -> Option<Scale> {
    src.into_scale()
}

#[derive(Debug, Clone, Default)]
pub struct ScaleDetectOptions {
    tonic: Option<String>,
    exact_match: bool,
}

pub fn detect(notes: &[&str], options: ScaleDetectOptions) -> Vec<String> {
    let notes_chroma = chroma(notes);
    let tonic = if let Some(t) = options.tonic {
        t.as_str().into_note()
    } else if !notes.is_empty() {
        notes.first().unwrap().into_note()
    } else {
        "".into_note()
    };
    let tonic_ref = tonic.as_ref();
    let tonic_chroma = tonic_ref.map(|t| t.chroma);
    let tonic_name = tonic_ref.map_or(String::new(), |t| t.name.clone());

    let Some(tonic_chroma) = tonic_chroma else {
        return Vec::new();
    };

    let mut pitch_classes: Vec<String> = notes_chroma.chars().map(String::from).collect();
    pitch_classes[tonic_chroma as usize] = String::from("1");
    let scale_chroma = rotated(tonic_chroma, &pitch_classes[..]).join("");

    let m = scale_types().iter().find(|st| st.chroma == scale_chroma);

    let mut results: Vec<String> = vec![];

    if let Some(m) = m {
        results.push(format!("{} {}", tonic_name, m.name));
    }

    if options.exact_match {
        return results;
    }

    for name in extended(&scale_chroma).iter() {
        results.push(format!("{} {}", tonic_name, name))
    }

    results
}

pub fn extended(name: &str) -> Vec<String> {
    let chroma = if is_chroma(name) {
        name
    } else {
        get(name).map_or("", |s| s.chroma)
    };
    let is_superset = is_superset_of(chroma);

    scale_types()
        .iter()
        .filter_map(|st| {
            if is_superset(&st.chroma) {
                Some(st.name.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn reduced(name: &str) -> Vec<String> {
    let chroma = get(name).map_or("", |s| s.chroma);
    let is_subset = is_subset_of(chroma);

    scale_types()
        .iter()
        .filter_map(|st| {
            if is_subset(&st.chroma) {
                Some(st.name.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn scale_notes(notes: &[&str]) -> Vec<String> {
    let pcset: Vec<_> = notes
        .iter()
        .filter_map(|n| {
            let note = n.into_note();
            note.map(|ni| ni.pc)
        })
        .collect();
    let Some(tonic) = pcset.first() else {
        return vec![];
    };
    let scale = sorted_uniq_names(&pcset);
    let Some(pos) = scale.iter().position(|x| x == tonic) else {
        return vec![];
    };
    rotated(pos as i32, &scale)
}

pub fn mode_names(name: &str) -> Vec<(String, String)> {
    let s = get(name);

    let Some(s) = s else {
        return Vec::new();
    };

    let tonics = if s.tonic.is_empty() {
        s.intervals
    } else {
        s.notes.as_slice()
    };

    modes(s.chroma, true)
        .iter()
        .enumerate()
        .filter_map(|(i, chroma)| {
            let mode_name = get(chroma.as_str()).map(|s| s.name)?;
            Some((tonics[i].clone(), mode_name))
        })
        .collect()
}

pub trait IntoScaleNotes {
    fn into_scale_notes(self) -> Vec<String>;
}

impl IntoScaleNotes for &str {
    fn into_scale_notes(self) -> Vec<String> {
        self.into_scale().map_or(Vec::new(), |s| s.notes)
    }
}

impl IntoScaleNotes for &[&str] {
    fn into_scale_notes(self) -> Vec<String> {
        scale_notes(self)
    }
}

fn get_note_name_of<S, M>(scale: S) -> impl Fn(M) -> Option<String>
where
    S: IntoScaleNotes,
    M: ToMidi,
{
    let names = scale.into_scale_notes();
    let chromas: Vec<_> = names
        .iter()
        // safe to unwrap . names has valid note names
        .map(|n| n.as_str().into_note().unwrap().chroma)
        .collect();

    move |note_or_midi: M| -> Option<String> {
        let midi = note_or_midi.to_midi()?;
        let curr_note = from_midi(midi).as_str().into_note()?;
        let pos = chromas.iter().position(|&c| c == curr_note.chroma)?;
        let enh = enharmonic(curr_note.name.as_str(), names.get(pos).map(|s| s.as_str()));
        if enh.is_empty() { None } else { Some(enh) }
    }
}

pub fn range_of<S: IntoScaleNotes>(scale: S) -> impl Fn(&str, &str) -> Vec<String> {
    let get_name = get_note_name_of(scale);

    move |from, to| {
        let [Some(from), Some(to)] = [from, to].map(|n| n.into_note().and_then(|n| n.midi)) else {
            return vec![];
        };

        range(from, to).iter().flat_map(|&m| get_name(m)).collect()
    }
}

pub fn degrees<S: IntoScale>(s: S) -> impl Fn(i32) -> String {
    let (intervals, tonic) = match get(s) {
        Some(s) => (s.intervals.to_owned(), Some(s.tonic)),
        None => (vec![], None),
    };
    let transpose = tonic_intervals_transposer(intervals, tonic);
    move |degree| {
        if degree == 0 {
            String::new()
        } else {
            transpose(if degree > 0 { degree - 1 } else { degree })
        }
    }
}

pub fn steps<S: IntoScale>(s: S) -> impl Fn(i32) -> String {
    let (intervals, tonic) = match get(s) {
        Some(s) => (s.intervals.to_owned(), Some(s.tonic)),
        None => (Vec::new(), None),
    };
    tonic_intervals_transposer(intervals, tonic)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    #[test]
    fn test_get_major() {
        let s = get("major").unwrap();
        assert_eq!(s.tonic, "");
        assert_eq!(s.notes, Vec::<String>::new());
        assert_eq!(s.kind, "major");
        assert_eq!(s.name, "major");
        assert_eq!(s.intervals, w("1P 2M 3M 4P 5P 6M 7M"));
        assert_eq!(s.aliases, ["ionian"]);
        assert_eq!(s.set_num, 2773);
        assert_eq!(s.chroma, "101011010101");
        assert_eq!(s.normalized, "101010110101");
    }

    #[test]
    fn test_get_pentatonic_with_tonic() {
        let s = get("c5 pentatonic").unwrap();
        assert_eq!(s.name, "C5 major pentatonic");
        assert_eq!(s.kind, "major pentatonic");
        assert_eq!(s.tonic, "C5");
        assert_eq!(s.notes, w("C5 D5 E5 G5 A5"));
        assert_eq!(s.intervals, w("1P 2M 3M 5P 6M"));
        assert_eq!(s.aliases, ["pentatonic"]);
        assert_eq!(s.set_num, 2708);
        assert_eq!(s.chroma, "101010010100");
        assert_eq!(s.normalized, "100101001010");
    }

    #[test]
    fn test_get_equivalences() {
        assert_eq!(get("C4 major"), get(("C4", "major")));
        assert_eq!(get("C4 Major"), get("C4 major"));
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("c major"), ("C".into(), "major".into()));
        assert_eq!(tokenize("cb3 major"), ("Cb3".into(), "major".into()));
        assert_eq!(
            tokenize("melodic minor"),
            ("".into(), "melodic minor".into())
        );
        assert_eq!(tokenize("dorian"), ("".into(), "dorian".into()));
        assert_eq!(tokenize("c"), ("C".into(), "".into()));
        assert_eq!(tokenize(""), ("".into(), "".into()));
    }

    #[test]
    fn test_is_known() {
        assert!(get("major").is_some());
        assert!(get("Db major").is_some());
        assert!(get("hello").is_none());
        assert!(get("").is_none());
        assert!(get("Maj7").is_none());
    }

    #[test]
    fn test_mixed_cases() {
        assert_eq!(
            get("C lydian #5P pentatonic"),
            get("C lydian #5p pentatonic")
        );
        assert_eq!(get("lydian #5p PENTATONIC"), get("lydian #5P pentatonic"));
    }

    #[test]
    fn test_interval_qualifiers_in_name() {
        let lydian5p = get("C lydian #5p pentatonic").unwrap();
        assert_eq!(lydian5p.name, "C lydian #5p pentatonic");
        assert_eq!(lydian5p.notes, w("C E F# G# B"));

        let minor7m = get("C minor #7m pentatonic").unwrap();
        assert_eq!(minor7m.name, "C minor #7m pentatonic");
        assert_eq!(minor7m.notes, w("C Eb F G B"));
    }

    #[test]
    fn test_intervals() {
        assert_eq!(get("major").unwrap().intervals, w("1P 2M 3M 4P 5P 6M 7M"));
        assert_eq!(get("C major").unwrap().intervals, w("1P 2M 3M 4P 5P 6M 7M"));
        assert!(get("blah").is_none());
    }

    #[test]
    fn test_notes() {
        assert_eq!(get("C major").unwrap().notes, w("C D E F G A B"));
        assert_eq!(get("C lydian #9").unwrap().notes, w("C D# E F# G A B"));
        assert_eq!(get(("C", "major")).unwrap().notes, w("C D E F G A B"));
        assert_eq!(
            get(("C4", "major")).unwrap().notes,
            w("C4 D4 E4 F4 G4 A4 B4")
        );
        assert_eq!(
            get(("eb", "bebop")).unwrap().notes,
            w("Eb F G Ab Bb C Db D")
        );
        assert!(get(("C", "no-scale")).is_none());
        assert_eq!(
            get(("no-note", "major")).unwrap().notes,
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_ukrainian_dorian() {
        assert_eq!(
            get("C romanian minor").unwrap().notes,
            w("C D Eb F# G A Bb")
        );
        assert_eq!(
            get("C ukrainian dorian").unwrap().notes,
            w("C D Eb F# G A Bb")
        );
        assert_eq!(
            get("B romanian minor").unwrap().notes,
            w("B C# D E# F# G# A")
        );
        assert_eq!(get("B dorian #4").unwrap().notes, w("B C# D E# F# G# A"));
        assert_eq!(
            get("B altered dorian").unwrap().notes,
            w("B C# D E# F# G# A")
        );
    }

    #[test]
    fn test_whole_tone_uses_sixth() {
        assert_eq!(
            get("C whole tone").unwrap().notes.join(" "),
            "C D E F# G# A#"
        );
        assert_eq!(
            get("Db whole tone").unwrap().notes.join(" "),
            "Db Eb F G A B"
        );
    }

    #[test]
    fn test_extended() {
        assert_eq!(
            extended("major"),
            ["bebop", "bebop major", "ichikosucho", "chromatic"]
        );
        assert_eq!(extended("none"), Vec::<String>::new());
    }

    #[test]
    fn test_reduced() {
        assert_eq!(
            reduced("major"),
            ["major pentatonic", "ionian pentatonic", "ritusen"]
        );
        assert_eq!(reduced("D major"), reduced("major"));
        assert_eq!(reduced("none"), Vec::<String>::new());
    }

    #[test]
    fn test_scale_notes() {
        assert_eq!(scale_notes(&["C4", "c3", "C5", "C4", "c4"]), ["C"]);
        assert_eq!(
            scale_notes(&["C4", "f3", "c#10", "b5", "d4", "cb4"]),
            w("C C# D F B Cb")
        );
        assert_eq!(scale_notes(&["D4", "c#5", "A5", "F#6"]), w("D F# A C#"));
    }

    #[test]
    fn test_mode_names() {
        let expect = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
            pairs
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect()
        };
        assert_eq!(
            mode_names("pentatonic"),
            expect(&[
                ("1P", "major pentatonic"),
                ("2M", "egyptian"),
                ("3M", "malkos raga"),
                ("5P", "ritusen"),
                ("6M", "minor pentatonic"),
            ])
        );
        assert_eq!(
            mode_names("whole tone pentatonic"),
            expect(&[("1P", "whole tone pentatonic")])
        );
        assert_eq!(
            mode_names("C pentatonic"),
            expect(&[
                ("C", "major pentatonic"),
                ("D", "egyptian"),
                ("E", "malkos raga"),
                ("G", "ritusen"),
                ("A", "minor pentatonic"),
            ])
        );
    }

    #[test]
    fn test_range_of_name() {
        let r = range_of("C pentatonic");
        assert_eq!(r("C4", "C5").join(" "), "C4 D4 E4 G4 A4 C5");
        assert_eq!(r("C5", "C4").join(" "), "C5 A4 G4 E4 D4 C4");
        assert_eq!(r("g3", "a2").join(" "), "G3 E3 D3 C3 A2");
    }

    #[test]
    fn test_range_of_flat_and_sharp() {
        let flat = range_of("Cb major");
        assert_eq!(
            flat("Cb4", "Cb5").join(" "),
            "Cb4 Db4 Eb4 Fb4 Gb4 Ab4 Bb4 Cb5"
        );
        let sharp = range_of("C# major");
        assert_eq!(
            sharp("C#4", "C#5").join(" "),
            "C#4 D#4 E#4 F#4 G#4 A#4 B#4 C#5"
        );
    }

    #[test]
    fn test_range_of_no_tonic_and_list() {
        let no_tonic = range_of("pentatonic");
        assert_eq!(no_tonic("C4", "C5"), Vec::<String>::new());
        let list = range_of(&["c4", "g4", "db3", "g"][..]);
        assert_eq!(list("c4", "c5").join(" "), "C4 Db4 G4 C5");
    }

    #[test]
    fn test_degrees_positive() {
        let d = degrees("C major");
        let got: Vec<String> = (1..=10).map(&d).collect();
        assert_eq!(got.join(" "), "C D E F G A B C D E");

        let d = degrees("C4 major");
        let got: Vec<String> = (1..=10).map(&d).collect();
        assert_eq!(got.join(" "), "C4 D4 E4 F4 G4 A4 B4 C5 D5 E5");

        let d = degrees("C4 pentatonic");
        let got: Vec<String> = (1..=11).map(&d).collect();
        assert_eq!(got.join(" "), "C4 D4 E4 G4 A4 C5 D5 E5 G5 A5 C6");
    }

    #[test]
    fn test_degrees_invalid() {
        assert_eq!(degrees("C major")(0), "");
        assert_eq!(degrees("C nonsense")(0), "");
    }

    #[test]
    fn test_degrees_negative() {
        let d = degrees("C major");
        let got: Vec<String> = (1..=10).map(|i| d(-i)).collect();
        assert_eq!(got.join(" "), "B A G F E D C B A G");

        let d = degrees("C4 major");
        let got: Vec<String> = (1..=10).map(|i| d(-i)).collect();
        assert_eq!(got.join(" "), "B3 A3 G3 F3 E3 D3 C3 B2 A2 G2");

        let d = degrees("C4 pentatonic");
        let got: Vec<String> = (1..=11).map(|i| d(-i)).collect();
        assert_eq!(got.join(" "), "A3 G3 E3 D3 C3 A2 G2 E2 D2 C2 A1");
    }

    #[test]
    fn test_steps() {
        let s = steps("C4 major");
        let got: Vec<String> = [-3, -2, -1, 0, 1, 2].iter().map(|&i| s(i)).collect();
        assert_eq!(got, w("G3 A3 B3 C4 D4 E4"));
    }

    #[test]
    fn test_detect_exact() {
        let exact = || ScaleDetectOptions {
            tonic: None,
            exact_match: true,
        };
        assert_eq!(
            detect(&["D", "E", "F#", "A", "B"], exact()),
            ["D major pentatonic"]
        );
        assert_eq!(
            detect(
                &["D", "E", "F#", "A", "B"],
                ScaleDetectOptions {
                    tonic: Some("B".into()),
                    exact_match: true
                }
            ),
            ["B minor pentatonic"]
        );
        assert_eq!(
            detect(&["D", "F#", "B", "C", "C#"], exact()),
            Vec::<String>::new()
        );
        assert_eq!(
            detect(&["c", "d", "e", "f", "g", "a", "b"], exact()),
            ["C major"]
        );
        assert_eq!(
            detect(
                &["c2", "d6", "e3", "f1", "g7", "a6", "b5"],
                ScaleDetectOptions {
                    tonic: Some("d".into()),
                    exact_match: true
                }
            ),
            ["D dorian"]
        );
    }

    #[test]
    fn test_detect_fit() {
        assert_eq!(
            detect(
                &["C", "D", "E", "F", "G", "A", "B"],
                ScaleDetectOptions::default()
            ),
            [
                "C major",
                "C bebop",
                "C bebop major",
                "C ichikosucho",
                "C chromatic"
            ]
        );
        assert_eq!(
            detect(&["D", "F#", "B", "C", "C#"], ScaleDetectOptions::default()),
            ["D bebop", "D kafi raga", "D chromatic"]
        );
    }

    #[test]
    fn test_detect_tonic_added() {
        assert_eq!(
            detect(
                &["c", "d", "e", "f", "g", "b"],
                ScaleDetectOptions {
                    tonic: None,
                    exact_match: true
                }
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            detect(
                &["c", "d", "e", "f", "g", "b"],
                ScaleDetectOptions {
                    tonic: Some("a".into()),
                    exact_match: true
                }
            ),
            ["A minor"]
        );
    }
}
