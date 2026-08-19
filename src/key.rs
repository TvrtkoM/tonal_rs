use std::borrow::Cow;

use crate::note::{transpose, transpose_fifths};
use crate::pitch::Named;
use crate::pitch_note::{IntoNote, acc_to_alt, alt_to_acc};
use crate::regexes;
use crate::roman_numeral::IntoRomanNumeral;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyScale {
    pub(crate) tonic: String,
    pub(crate) grades: Vec<String>,
    pub(crate) intervals: Vec<String>,
    pub(crate) scale: Vec<String>,
    pub(crate) triads: Vec<String>,
    pub(crate) chords: Vec<String>,
    pub(crate) chords_harmonic_function: Vec<String>,
    pub(crate) chord_scales: Vec<String>,
    pub(crate) secondary_dominants: Vec<String>,
    pub(crate) secondary_dominant_supertonics: Vec<String>,
    pub(crate) substitute_dominants: Vec<String>,
    pub(crate) substitute_dominant_supertonics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MajorKey {
    pub(crate) scale: KeyScale,
    pub(crate) minor_relative: String,
    pub(crate) tonic: String,
    pub(crate) alteration: i32,
    pub(crate) key_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinorKey {
    pub(crate) major_relative: String,
    pub(crate) tonic: String,
    pub(crate) alteration: i32,
    pub(crate) key_signature: String,
    pub(crate) natural: KeyScale,
    pub(crate) harmonic: KeyScale,
    pub(crate) melodic: KeyScale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyChord {
    pub(crate) name: String,
    pub(crate) roles: Vec<String>,
}

impl Named for KeyChord {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        Cow::from(self.name.as_str())
    }
}

impl std::fmt::Display for KeyChord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyScaleParts<'a> {
    pub tonic: &'a str,
    pub grades: &'a [String],
    pub intervals: &'a [String],
    pub scale: &'a [String],
    pub triads: &'a [String],
    pub chords: &'a [String],
    pub chords_harmonic_function: &'a [String],
    pub chord_scales: &'a [String],
    pub secondary_dominants: &'a [String],
    pub secondary_dominant_supertonics: &'a [String],
    pub substitute_dominants: &'a [String],
    pub substitute_dominant_supertonics: &'a [String],
}

#[derive(Debug, Clone, Copy)]
pub struct MajorKeyParts<'a> {
    pub scale: &'a KeyScale,
    pub minor_relative: &'a str,
    pub tonic: &'a str,
    pub alteration: i32,
    pub key_signature: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct MinorKeyParts<'a> {
    pub major_relative: &'a str,
    pub tonic: &'a str,
    pub alteration: i32,
    pub key_signature: &'a str,
    pub natural: &'a KeyScale,
    pub harmonic: &'a KeyScale,
    pub melodic: &'a KeyScale,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyChordParts<'a> {
    pub name: &'a str,
    pub roles: &'a [String],
}

impl KeyScale {
    pub fn parts(&self) -> KeyScaleParts<'_> {
        KeyScaleParts {
            tonic: &self.tonic,
            grades: &self.grades,
            intervals: &self.intervals,
            scale: &self.scale,
            triads: &self.triads,
            chords: &self.chords,
            chords_harmonic_function: &self.chords_harmonic_function,
            chord_scales: &self.chord_scales,
            secondary_dominants: &self.secondary_dominants,
            secondary_dominant_supertonics: &self.secondary_dominant_supertonics,
            substitute_dominants: &self.substitute_dominants,
            substitute_dominant_supertonics: &self.substitute_dominant_supertonics,
        }
    }
}

impl MajorKey {
    pub fn parts(&self) -> MajorKeyParts<'_> {
        MajorKeyParts {
            scale: &self.scale,
            minor_relative: &self.minor_relative,
            tonic: &self.tonic,
            alteration: self.alteration,
            key_signature: &self.key_signature,
        }
    }
}

impl MinorKey {
    pub fn parts(&self) -> MinorKeyParts<'_> {
        MinorKeyParts {
            major_relative: &self.major_relative,
            tonic: &self.tonic,
            alteration: self.alteration,
            key_signature: &self.key_signature,
            natural: &self.natural,
            harmonic: &self.harmonic,
            melodic: &self.melodic,
        }
    }
}

impl KeyChord {
    pub fn parts(&self) -> KeyChordParts<'_> {
        KeyChordParts {
            name: &self.name,
            roles: &self.roles,
        }
    }
}

fn map_scale_to_type(scale: &[String], list: &[&str], sep: &str) -> Vec<String> {
    list.iter()
        .enumerate()
        .map(|(i, kind)| format!("{}{sep}{kind}", scale[i]))
        .collect()
}

fn supertonics(dominants: &[String], target_triads: &[&str]) -> Vec<String> {
    dominants
        .iter()
        .enumerate()
        .map(|(i, chord)| {
            if chord.is_empty() {
                return String::new();
            }
            let mut chord_chars = chord.chars();
            chord_chars.next_back();
            let dom_root = chord_chars.as_str();
            let minor_root = transpose(dom_root, "5P");
            let target = target_triads[i];
            let is_minor = target.ends_with('m');

            if is_minor {
                format!("{}{}", minor_root, "m7")
            } else {
                format!("{}{}", minor_root, "m7b5")
            }
        })
        .collect()
}

fn key_scale(
    grades: &[&str],
    triads: &[&str],
    chord_types: &[&str],
    harmonic_functions: &[&str],
    chord_scales: &[&str],
) -> impl Fn(&str) -> KeyScale {
    move |tonic| {
        let intervals: Vec<_> = grades
            .iter()
            .map(|gr| {
                gr.into_roman_numeral()
                    .map(|r| r.interval)
                    .unwrap_or(String::new())
            })
            .collect();
        let scale: Vec<_> = intervals
            .iter()
            .map(|ivl| transpose(tonic, ivl.as_str()))
            .collect();

        let chords = map_scale_to_type(&scale, chord_types, "");

        let secondary_dominants: Vec<_> = scale
            .iter()
            .map(|note| {
                let note = transpose(note.as_str(), "5P");
                let seventh = format!("{}7", note);

                if scale.contains(&note) && !chords.contains(&seventh) {
                    seventh
                } else {
                    String::new()
                }
            })
            .collect();

        let secondary_dominant_supertonics = supertonics(&secondary_dominants, triads);

        let substitute_dominants: Vec<String> = secondary_dominants
            .iter()
            .map(|chord| {
                if chord.is_empty() {
                    return String::new();
                }
                let mut chord_chars = chord.chars();
                chord_chars.next_back();
                let dom_root = chord_chars.as_str();
                let sub_root = transpose(dom_root, "5d");
                format!("{}{}", sub_root, "7")
            })
            .collect();

        let substitute_dominant_supertonics = supertonics(&substitute_dominants, triads);

        let grades: Vec<String> = grades.iter().map(|g| g.to_string()).collect();
        let triads: Vec<String> = map_scale_to_type(&scale, triads, "");

        let chords_harmonic_function: Vec<String> =
            harmonic_functions.iter().map(|h| h.to_string()).collect();

        let chord_scales = map_scale_to_type(&scale, chord_scales, " ");

        KeyScale {
            tonic: tonic.to_string(),
            grades,
            intervals,
            scale,
            triads,
            chords,
            chords_harmonic_function,
            chord_scales,
            secondary_dominants,
            secondary_dominant_supertonics,
            substitute_dominants,
            substitute_dominant_supertonics,
        }
    }
}

fn dist_in_fifths(from: &str, to: &str) -> i32 {
    let (Some(f), Some(t)) = (from.into_note(), to.into_note()) else {
        return 0;
    };

    let from_coords: (i32, _, _) = (&f.coord).into();
    let to_coords: (i32, _, _) = (&t.coord).into();

    to_coords.0 - from_coords.0
}

fn major_scale(tonic: &str) -> KeyScale {
    key_scale(
        &["I", "II", "III", "IV", "V", "VI", "VII"],
        &["", "m", "m", "", "", "m", "dim"],
        &["maj7", "m7", "m7", "maj7", "7", "m7", "m7b5"],
        &["T", "SD", "T", "SD", "D", "T", "D"],
        &[
            "major",
            "dorian",
            "phrygian",
            "lydian",
            "mixolydian",
            "minor",
            "locrian",
        ],
    )(tonic)
}

fn natural_scale(tonic: &str) -> KeyScale {
    key_scale(
        &["I", "II", "bIII", "IV", "V", "bVI", "bVII"],
        &["m", "dim", "", "m", "m", "", ""],
        &["m7", "m7b5", "maj7", "m7", "m7", "maj7", "7"],
        &["T", "SD", "T", "SD", "D", "SD", "SD"],
        &[
            "minor",
            "locrian",
            "major",
            "dorian",
            "phrygian",
            "lydian",
            "mixolydian",
        ],
    )(tonic)
}

fn harmonic_scale(tonic: &str) -> KeyScale {
    key_scale(
        &["I", "II", "bIII", "IV", "V", "bVI", "VII"],
        &["m", "dim", "aug", "m", "", "", "dim"],
        &["mMaj7", "m7b5", "+maj7", "m7", "7", "maj7", "o7"],
        &["T", "SD", "T", "SD", "D", "SD", "D"],
        &[
            "harmonic minor",
            "locrian 6",
            "major augmented",
            "lydian diminished",
            "phrygian dominant",
            "lydian #9",
            "ultralocrian",
        ],
    )(tonic)
}

fn melodic_scale(tonic: &str) -> KeyScale {
    key_scale(
        &["I", "II", "bIII", "IV", "V", "VI", "VII"],
        &["m", "m", "aug", "", "", "dim", "dim"],
        &["m6", "m7", "+maj7", "7", "7", "m7b5", "m7b5"],
        &["T", "SD", "T", "SD", "D", "", ""],
        &[
            "melodic minor",
            "dorian b2",
            "lydian augmented",
            "lydian dominant",
            "mixolydian b6",
            "locrian #2",
            "altered",
        ],
    )(tonic)
}

pub fn major_key(tonic: &str) -> Option<MajorKey> {
    let pc = tonic.into_note().map(|n| n.pc)?;

    let pc_str = pc.as_str();

    let key_scale = major_scale(pc_str);
    let alteration = dist_in_fifths("C", pc_str);

    let major_key = MajorKey {
        scale: key_scale,
        minor_relative: transpose(pc_str, "-3m"),
        alteration,
        key_signature: alt_to_acc(alteration),
        tonic: pc,
    };

    Some(major_key)
}

pub fn minor_key(tonic: &str) -> Option<MinorKey> {
    let pc = tonic.into_note().map(|n| n.pc)?;

    let pc_str = pc.as_str();

    let major_relative = transpose(pc_str, "3m");
    let alteration = dist_in_fifths("C", pc_str) - 3;
    let key_signature = alt_to_acc(alteration);
    let natural = natural_scale(pc_str);
    let harmonic = harmonic_scale(pc_str);
    let melodic = melodic_scale(pc_str);

    let key = MinorKey {
        tonic: pc,
        major_relative,
        alteration,
        key_signature,
        natural,
        harmonic,
        melodic,
    };

    Some(key)
}

fn update_chord(chords: &mut Vec<KeyChord>, name: &String, new_role: &String) {
    if name.is_empty() {
        return;
    }

    let role_empty = new_role.is_empty();

    match chords.iter_mut().find(|chord| chord.name == *name) {
        Some(kc) => {
            if !role_empty && !kc.roles.iter().any(|r| r == new_role) {
                kc.roles.push(new_role.to_string());
            }
        }
        None => chords.push(KeyChord {
            name: name.clone(),
            roles: if role_empty {
                vec![]
            } else {
                vec![new_role.clone()]
            },
        }),
    }
}

fn key_chords_of(key: &KeyScale, chords: &mut Vec<KeyChord>) {
    for (i, name) in key.chords.iter().enumerate() {
        update_chord(chords, name, &key.chords_harmonic_function[i]);
    }
    for (i, name) in key.secondary_dominants.iter().enumerate() {
        update_chord(chords, name, &format!("V/{}", key.grades[i]));
    }
    for (i, name) in key.secondary_dominant_supertonics.iter().enumerate() {
        update_chord(chords, name, &format!("ii/{}", key.grades[i]));
    }
    for (i, name) in key.substitute_dominants.iter().enumerate() {
        update_chord(chords, name, &format!("subV/{}", key.grades[i]));
    }
    for (i, name) in key.substitute_dominant_supertonics.iter().enumerate() {
        update_chord(chords, name, &format!("subii/{}", key.grades[i]));
    }
}

pub fn major_key_chords(tonic: &str) -> Vec<KeyChord> {
    let mut chords: Vec<KeyChord> = vec![];
    if let Some(key) = major_key(tonic) {
        key_chords_of(&key.scale, &mut chords);
    }
    chords
}

pub fn minor_key_chords(tonic: &str) -> Vec<KeyChord> {
    let mut chords: Vec<KeyChord> = vec![];
    if let Some(key) = minor_key(tonic) {
        key_chords_of(&key.natural, &mut chords);
        key_chords_of(&key.harmonic, &mut chords);
        key_chords_of(&key.melodic, &mut chords);
    }
    chords
}

pub trait IntoMajorTonic {
    fn into_major_tonic(self) -> Option<String>;
}

impl IntoMajorTonic for &str {
    fn into_major_tonic(self) -> Option<String> {
        if regexes::ACCIDENTAL.is_match(self) {
            Some(transpose_fifths("C", acc_to_alt(self)))
        } else {
            None
        }
    }
}

impl IntoMajorTonic for i32 {
    fn into_major_tonic(self) -> Option<String> {
        Some(transpose_fifths("C", self))
    }
}

pub fn major_tonic_from_key_signature<T: IntoMajorTonic>(sig: T) -> Option<String> {
    sig.into_major_tonic()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strs(v: &[String]) -> Vec<&str> {
        v.iter().map(|s| s.as_str()).collect()
    }

    #[test]
    fn test_from_alter() {
        assert_eq!(major_tonic_from_key_signature("###"), Some("A".to_string()));
        assert_eq!(major_tonic_from_key_signature(3), Some("A".to_string()));
        assert_eq!(major_tonic_from_key_signature("b"), Some("F".to_string()));
        assert_eq!(major_tonic_from_key_signature("bb"), Some("Bb".to_string()));
        assert_eq!(major_tonic_from_key_signature("other"), None);
    }

    #[test]
    fn test_major_key_signature() {
        let signatures: Vec<String> = "C D E F G A B"
            .split(' ')
            .map(|t| major_key(t).unwrap().key_signature)
            .collect();
        assert_eq!(signatures.join(" "), " ## #### b # ### #####");
    }

    #[test]
    fn test_minor_key_signature() {
        let signatures: Vec<String> = "C D E F G A B"
            .split(' ')
            .map(|t| minor_key(t).unwrap().key_signature)
            .collect();
        assert_eq!(signatures.join(" "), "bbb b # bbbb bb  ##");
    }

    #[test]
    fn test_natural_scale_names() {
        let natural = minor_key("C").unwrap().natural;
        assert_eq!(
            strs(&natural.chord_scales),
            [
                "C minor",
                "D locrian",
                "Eb major",
                "F dorian",
                "G phrygian",
                "Ab lydian",
                "Bb mixolydian",
            ]
        );
    }

    #[test]
    fn test_harmonic_scale_names() {
        let harmonic = minor_key("C").unwrap().harmonic;
        assert_eq!(
            strs(&harmonic.chord_scales),
            [
                "C harmonic minor",
                "D locrian 6",
                "Eb major augmented",
                "F lydian diminished",
                "G phrygian dominant",
                "Ab lydian #9",
                "B ultralocrian",
            ]
        );
    }

    #[test]
    fn test_melodic_scale_names() {
        let melodic = minor_key("C").unwrap().melodic;
        assert_eq!(
            strs(&melodic.chord_scales),
            [
                "C melodic minor",
                "D dorian b2",
                "Eb lydian augmented",
                "F lydian dominant",
                "G mixolydian b6",
                "A locrian #2",
                "B altered",
            ]
        );
    }

    #[test]
    fn test_secondary_dominants() {
        let major = major_key("C").unwrap();
        assert_eq!(
            strs(&major.scale.secondary_dominants),
            ["", "A7", "B7", "C7", "D7", "E7", ""]
        );
    }

    #[test]
    fn test_octaves_are_discarded() {
        assert_eq!(
            major_key("b4").unwrap().scale.scale.join(" "),
            "B C# D# E F# G# A#"
        );
        assert_eq!(
            major_key("g4").unwrap().scale.chords.join(" "),
            "Gmaj7 Am7 Bm7 Cmaj7 D7 Em7 F#m7b5"
        );
        assert_eq!(
            minor_key("C4").unwrap().melodic.scale.join(" "),
            "C D Eb F G A B"
        );
        assert_eq!(
            minor_key("C4").unwrap().melodic.chords.join(" "),
            "Cm6 Dm7 Eb+maj7 F7 G7 Am7b5 Bm7b5"
        );
    }

    #[test]
    fn test_valid_chord_names() {
        let major = major_key("C").unwrap();
        let minor = minor_key("C").unwrap();

        let groups: [&[String]; 7] = [
            &major.scale.chords,
            &major.scale.secondary_dominants,
            &major.scale.substitute_dominant_supertonics,
            &major.scale.substitute_dominants,
            &minor.natural.chords,
            &minor.harmonic.chords,
            &minor.melodic.chords,
        ];

        for group in groups {
            for name in group {
                if !name.is_empty() {
                    let parsed = crate::chord::get(name.as_str());
                    assert!(
                        parsed.is_some_and(|c| !c.name.is_empty()),
                        "invalid chord: {name}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_c_major() {
        let key = major_key("C").unwrap();
        let p = key.parts();

        assert_eq!(p.tonic, "C");
        assert_eq!(p.alteration, 0);
        assert_eq!(p.key_signature, "");
        assert_eq!(p.minor_relative, "A");

        let s = p.scale.parts();
        assert_eq!(s.grades.join(" "), "I II III IV V VI VII");
        assert_eq!(s.intervals.join(" "), "1P 2M 3M 4P 5P 6M 7M");
        assert_eq!(s.scale.join(" "), "C D E F G A B");
        assert_eq!(s.triads.join(" "), "C Dm Em F G Am Bdim");
        assert_eq!(s.chords.join(" "), "Cmaj7 Dm7 Em7 Fmaj7 G7 Am7 Bm7b5");
        assert_eq!(s.chords_harmonic_function.join(" "), "T SD T SD D T D");
        assert_eq!(
            strs(s.chord_scales),
            [
                "C major",
                "D dorian",
                "E phrygian",
                "F lydian",
                "G mixolydian",
                "A minor",
                "B locrian",
            ]
        );
    }

    #[test]
    fn test_c_major_chords() {
        let chords = major_key_chords("C");
        let em7 = chords.iter().find(|c| c.name == "Em7").unwrap();
        assert_eq!(em7.name, "Em7");
        assert_eq!(em7.roles, ["T", "ii/II"]);
    }

    #[test]
    fn test_major_keys() {
        let a = major_key("A").unwrap();
        assert_eq!(a.tonic, "A");
        assert_eq!(a.alteration, 3);
        assert_eq!(a.key_signature, "###");
        assert_eq!(a.minor_relative, "F#");

        let bb = major_key("Bb").unwrap();
        assert_eq!(bb.tonic, "Bb");
        assert_eq!(bb.alteration, -2);
        assert_eq!(bb.key_signature, "bb");
        assert_eq!(bb.minor_relative, "G");

        let e = major_key("E").unwrap();
        assert_eq!(e.tonic, "E");
        assert_eq!(e.alteration, 4);
        assert_eq!(e.key_signature, "####");
        assert_eq!(e.minor_relative, "C#");
    }

    #[test]
    fn test_empty_major_key() {
        assert!(major_key("").is_none());
    }

    #[test]
    fn test_minor_key() {
        let c = minor_key("C").unwrap();
        assert_eq!(c.tonic, "C");
        assert_eq!(c.alteration, -3);
        assert_eq!(c.key_signature, "bbb");
        assert_eq!(c.major_relative, "Eb");

        let bb = minor_key("Bb").unwrap();
        assert_eq!(bb.tonic, "Bb");
        assert_eq!(bb.alteration, -5);
        assert_eq!(bb.key_signature, "bbbbb");
        assert_eq!(bb.major_relative, "Db");

        let b = minor_key("B").unwrap();
        assert_eq!(b.tonic, "B");
        assert_eq!(b.alteration, 2);
        assert_eq!(b.key_signature, "##");
        assert_eq!(b.major_relative, "D");
    }

    #[test]
    fn test_empty_minor_key() {
        assert!(minor_key("nothing").is_none());
    }
}
