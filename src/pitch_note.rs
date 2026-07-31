use crate::pitch::{Named, Pitch, PitchCoordinates};
use regex::Regex;
use std::sync::LazyLock;

#[cfg(test)]
#[path = "pitch_note_tests.rs"]
mod pitch_note_tests;

type NoteTokens = (String, String, String, String);

#[derive(Debug, PartialEq)]
pub struct Note {
    pub(crate) step: usize,
    pub(crate) alt: i32,
    pub(crate) oct: Option<i32>,
    pub(crate) name: String,
    pub(crate) letter: char,
    pub(crate) acc: String,
    pub(crate) pc: String,
    pub(crate) chroma: i32,
    pub(crate) height: i32,
    pub(crate) coord: PitchCoordinates,
    pub(crate) midi: Option<i32>,
    pub(crate) freq: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct NoteParts<'a> {
    pub step: usize,
    pub alt: i32,
    pub oct: Option<i32>,
    pub name: &'a str,
    pub letter: char,
    pub acc: &'a str,
    pub pc: &'a str,
    pub chroma: i32,
    pub height: i32,
    pub coord: PitchCoordinates,
    pub midi: Option<i32>,
    pub freq: Option<f32>,
}

impl Note {
    pub fn chroma(&self) -> i32 {
        self.chroma
    }

    pub fn midi(&self) -> Option<i32> {
        self.midi
    }

    pub fn parts(&self) -> NoteParts<'_> {
        NoteParts {
            step: self.step,
            alt: self.alt,
            oct: self.oct,
            name: &self.name,
            letter: self.letter,
            acc: &self.acc,
            pc: &self.pc,
            chroma: self.chroma,
            height: self.height,
            coord: self.coord,
            midi: self.midi,
            freq: self.freq,
        }
    }
}

impl Named for Note {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub fn note(name: &str) -> Option<Note> {
    parse(name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseNoteError {
    input: String,
}

impl std::fmt::Display for ParseNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid note name: {}", self.input)
    }
}

impl std::error::Error for ParseNoteError {}

impl TryFrom<&Pitch> for Note {
    type Error = ParseNoteError;
    fn try_from(p: &Pitch) -> Result<Self, Self::Error> {
        let name = p.name();
        note(&name).ok_or(ParseNoteError { input: name })
    }
}

impl TryFrom<&str> for Note {
    type Error = ParseNoteError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        note(s).ok_or_else(|| ParseNoteError {
            input: s.to_string(),
        })
    }
}

pub fn acc_to_alt(acc: &str) -> i32 {
    let len = acc.chars().count() as i32;
    acc.chars()
        .next()
        .map_or(len, |c| if c == 'b' { -len } else { len })
}

static NOTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-gA-G]?)(#+|b+|x+|)(-?\d*)\s*(.*)$").unwrap());

pub(crate) fn tokenize_note(s: &str) -> NoteTokens {
    match NOTE_REGEX.captures(s) {
        Some(c) => (
            c[1].to_uppercase(),
            c[2].replace('x', "##"),
            c[3].to_string(),
            c[4].to_string(),
        ),
        None => (String::new(), String::new(), String::new(), String::new()),
    }
}

fn parse(note_name: &str) -> Option<Note> {
    let (letter, acc, oct_str, rest) = tokenize_note(note_name);

    if letter.is_empty() || !rest.is_empty() {
        return None;
    }

    let letter = letter.chars().next().unwrap();
    let step = ((letter as usize) + 3) % 7;
    let alt = acc_to_alt(&acc);
    let oct = oct_str.parse::<i32>().ok();

    let pitch = Pitch {
        step,
        alt,
        oct,
        dir: None,
    };
    let coord = pitch.coordinates();
    let chroma = pitch.chroma();
    // note height differs from Pitch height: notes are MIDI aligned (+12),
    // pitch classes are chroma-based.
    let height = match oct {
        Some(_) => pitch.height() + 12,
        None => chroma - 12 * 99,
    };
    let midi = pitch.midi();
    let name = pitch.name();

    let pc = format!("{}{}", letter, acc);

    let freq = oct.map(|_| ((height as f32 - 69.) / 12.).exp2() * 440.);

    Some(Note {
        acc,
        alt,
        chroma,
        coord,
        freq,
        height,
        letter,
        midi,
        name,
        oct,
        pc,
        step,
    })
}
