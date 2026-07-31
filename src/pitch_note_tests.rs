use super::*;
use crate::pitch::{Pitch, PitchCoordinates};

// ---- helpers ------------------------------------------------------------

// Build a NoteTokens tuple from &str, so the tokenize asserts read like the TS.
fn tok(a: &str, b: &str, c: &str, d: &str) -> NoteTokens {
    (a.into(), b.into(), c.into(), d.into())
}

// note(n).height for each space-separated name (all names must be valid).
fn heights(s: &str) -> Vec<i32> {
    s.split(' ').map(|n| note(n).unwrap().height).collect()
}

// note(n).midi for each space-separated name.
fn midis(s: &str) -> Vec<Option<i32>> {
    s.split(' ').map(|n| note(n).unwrap().midi).collect()
}

// f32 comparison with relative tolerance (freq is f32 here; tonal uses f64).
fn assert_close(a: f32, b: f32) {
    assert!(
        (a - b).abs() <= 1e-4 * b.abs().max(1.0),
        "expected {a} to be close to {b}"
    );
}

// ---- tokenize -----------------------------------------------------------

#[test]
fn test_tokenize() {
    assert_eq!(tokenize_note("Cbb5 major"), tok("C", "bb", "5", "major"));
    assert_eq!(tokenize_note("Ax"), tok("A", "##", "", ""));
    assert_eq!(tokenize_note("CM"), tok("C", "", "", "M"));
    assert_eq!(tokenize_note("maj7"), tok("", "", "", "maj7"));
    assert_eq!(tokenize_note(""), tok("", "", "", ""));
    assert_eq!(tokenize_note("bb"), tok("B", "b", "", ""));
    assert_eq!(tokenize_note("##"), tok("", "##", "", ""));
    assert_eq!(tokenize_note(" |\n"), tok("", "", "", ""));
}

// ---- note properties from string ---------------------------------------

#[test]
fn test_properties() {
    let n = note("A4").unwrap();
    assert_eq!(n.name, "A4");
    assert_eq!(n.letter, 'A');
    assert_eq!(n.acc, "");
    assert_eq!(n.pc, "A");
    assert_eq!(n.step, 5);
    assert_eq!(n.alt, 0);
    assert_eq!(n.oct, Some(4));
    assert_eq!(n.coord, PitchCoordinates::Note(3, 3));
    assert_eq!(n.height, 69);
    assert_eq!(n.chroma, 9);
    assert_eq!(n.midi, Some(69));
    assert_close(n.freq.unwrap(), 440.0);
}

#[test]
fn test_roundtrip_from_note_name() {
    // TS: note(note("C4")) === note("C4"). Here we re-parse a note's own name.
    let c4 = note("C4").unwrap();
    assert_eq!(note(&c4.name), note("C4"));
}

#[test]
fn test_height() {
    assert_eq!(heights("C4 D4 E4 F4 G4"), [60, 62, 64, 65, 67]);
    assert_eq!(heights("B-2 C-1 D-1"), [-1, 0, 2]);
    assert_eq!(heights("F9 G9 A9"), [125, 127, 129]);
    assert_eq!(heights("C-4 D-4 E-4 F-4 G-4"), [-36, -34, -32, -31, -29]);
    assert_eq!(heights("C D E F G"), [-1188, -1186, -1184, -1183, -1181]);
    // enharmonic notes (with octave)
    assert_eq!(
        heights("Cb4 Cbb4 Cbbb4 B#4 B##4 B###4"),
        heights("B3 Bb3 Bbb3 C5 C#5 C##5"),
    );
    // enharmonic pitch classes (folded by chroma)
    assert_eq!(
        heights("Cb Cbb Cbbb B# B## B###"),
        heights("B Bb Bbb C C# C##"),
    );
}

#[test]
fn test_midi() {
    assert_eq!(
        midis("C4 D4 E4 F4 G4"),
        [Some(60), Some(62), Some(64), Some(65), Some(67)]
    );
    assert_eq!(midis("B-2 C-1 D-1"), [None, Some(0), Some(2)]);
    assert_eq!(midis("F9 G9 A9"), [Some(125), Some(127), None]);
    assert_eq!(midis("C-4 D-4 E-4 F-4"), [None, None, None, None]);
    assert_eq!(midis("C D E F"), [None, None, None, None]);
}

#[test]
fn test_freq() {
    assert_close(note("C4").unwrap().freq.unwrap(), 261.625_57);
    assert_close(note("B-2").unwrap().freq.unwrap(), 7.716_926_6);
    assert_close(note("F9").unwrap().freq.unwrap(), 11175.303);
    assert_close(note("C-4").unwrap().freq.unwrap(), 1.021_974_9);
    assert_eq!(note("C").unwrap().freq, None);
    // TS: note("x").freq === null. Here "x" is not a valid note at all.
    assert!(note("x").is_none());
}

// ---- note properties from pitch (via TryFrom<&Pitch>) ------------------

#[test]
fn test_from_pitch() {
    let pitch = |step, alt, oct| Pitch {
        step,
        alt,
        oct,
        dir: None,
    };

    assert_eq!(Note::try_from(&pitch(1, -1, None)).unwrap().name, "Db");
    assert_eq!(Note::try_from(&pitch(2, 1, None)).unwrap().name, "E#");
    assert_eq!(Note::try_from(&pitch(2, 1, Some(4))).unwrap().name, "E#4");
    assert_eq!(Note::try_from(&pitch(5, 0, None)).unwrap().name, "A");
    // out-of-range step -> no letter -> invalid
    assert!(Note::try_from(&pitch(8, 0, None)).is_err());
    // NOTE: the TS `{ step: -1 }` case is unrepresentable here because `step`
    // is `usize`; the type system rules out negative steps entirely.
}

// ---- Rust-specific: differences from the TS API ------------------------

#[test]
fn test_invalid_returns_none() {
    // TS returns a frozen NoNote object with empty:true; we return None.
    assert!(note("").is_none());
    assert!(note("blah").is_none());
    assert!(note("x").is_none());
    assert!(note("C4 major").is_none()); // trailing text -> not a note
}

#[test]
fn test_try_from_str_error() {
    assert!(Note::try_from("A4").is_ok());
    let err = Note::try_from("x").unwrap_err();
    assert_eq!(err, ParseNoteError { input: "x".into() });
}

#[test]
fn test_acc_to_alt() {
    assert_eq!(acc_to_alt(""), 0);
    assert_eq!(acc_to_alt("b"), -1);
    assert_eq!(acc_to_alt("bb"), -2);
    assert_eq!(acc_to_alt("#"), 1);
    assert_eq!(acc_to_alt("##"), 2);
}
