use crate::chord_type::{ChordQuality, ChordType, get as get_chord_type};
use crate::interval::subtract;
use crate::note::distance;
use crate::pitch_distance::transpose as transpose_note;
use crate::pitch_note::{IntoNote, Note, tokenize_note};

pub type ChordNameTokens = (String, String, String); // tonic, type, bass

#[derive(Debug, Clone)]
pub struct Chord {
    pub(crate) name: String,
    pub(crate) quality: ChordQuality,
    pub(crate) set_num: i32,
    pub(crate) chroma: String,
    pub(crate) normalized: String,
    pub(crate) intervals: Vec<String>,
    pub(crate) aliases: Vec<String>,
    pub(crate) tonic: String,
    pub(crate) kind: String,
    pub(crate) root: String,
    pub(crate) bass: String,
    pub(crate) root_degree: Option<i32>,
    pub(crate) symbol: String,
    pub(crate) notes: Vec<String>,
}

fn tokenize_bass(note: &str, chord: &str) -> ChordNameTokens {
    let split: Vec<_> = chord.split("/").collect();

    if split.len() == 1 {
        return (note.to_string(), split[0].to_string(), String::new());
    }

    let (letter, acc, oct, kind) = tokenize_note(split[1]);

    if !letter.is_empty() && !oct.is_empty() && !kind.is_empty() {
        (
            note.to_string(),
            split[0].to_string(),
            format!("{letter}{acc}"),
        )
    } else {
        (note.to_string(), chord.to_string(), String::new())
    }
}

pub fn tokenize(name: &str) -> ChordNameTokens {
    let (letter, acc, oct, kind) = tokenize_note(name);
    if letter.is_empty() {
        tokenize_bass("", name)
    } else if letter == "A" && kind == "ug" {
        tokenize_bass("", "aug")
    } else {
        tokenize_bass(
            format!("{name}{acc}").as_str(),
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

    let bass_name = if let Some(root) = root {
        format!(" over {}", root.pc)
    } else if has_bass {
        format!(" over {}", bass_pc)
    } else {
        String::new()
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

    let bass_symbol = if let Some(root) = root {
        format!("/{}", root.pc)
    } else if has_bass {
        format!("/{}", bass_pc)
    } else {
        "".to_string()
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

    if (!optional_tonic.is_none() && tonic.is_none())
        || (!optional_bass.is_none() && bass.is_none())
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
        chroma: chord_type.chroma.clone(),
        normalized: chord_type.normalized.clone(),
        intervals,
        aliases: chord_type.aliases.clone(),
        tonic: tonic_pc.to_string(),
        kind: chord_type.name.clone(),
        root: root.map_or(String::new(), |r| r.pc.clone()),
        bass: bass_pc.to_string(),
        root_degree: root_degree.map(|d| d as i32),
        notes,
    })
}

pub trait IntoChord {
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

pub fn get<C: IntoChord>(src: C) -> Option<Chord> {
    src.into_chord()
}

pub use self::get as chord;
