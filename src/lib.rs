//! `tonal_rs` is a music theory library — a Rust port of the JavaScript
//! [tonal](https://github.com/tonaljs/tonal) library.
//!
//! It contains functions to manipulate tonal elements of music (notes,
//! intervals, chords, scales, modes, keys). It deals with abstractions, not
//! with actual audio or sound.
//!
//! The API follows a functional style: functions are pure, there is no data
//! mutation, and musical entities are represented by plain data structures.
//!
//! # Overview
//!
//! Functionality is grouped into modules that mirror the tonal packages:
//!
//! - [`note`] — get properties of notes and transpose them by intervals.
//! - [`interval`] — create and manipulate intervals.
//! - [`chord`], [`chord_type`], [`chord_detect`] — build, describe and detect chords.
//! - [`scale`], [`scale_type`] — build and query scales.
//! - [`key`] — major and minor key signatures and their chords.
//! - [`mode`] — the seven diatonic modes.
//! - [`midi`], [`pitch_distance`] — MIDI conversions and note/interval arithmetic.
//! - [`pcset`] — pitch-class sets and set-theory operations.
//! - [`roman_numeral`], [`progression`] — roman numeral analysis and chord progressions.
//! - [`voicing`], [`voicing_dictionary`], [`voice_leading`] — chord voicings.
//! - [`range`], [`collection`] — note ranges and array helpers.
//! - [`duration_value`], [`rhythm_pattern`], [`time_signature`] — rhythmic values.
//! - [`abc_notation`], [`notation_scientific`] — notation format conversions.
//!
//! # Example
//!
//! `note::midi("C4")` returns `Some(60)`, `note::transpose("C4", "5P")` returns
//! `"G4"`, and `scale::get("C major").notes` returns the seven notes of the C
//! major scale.

pub mod abc_notation;
pub mod chord;
pub mod chord_detect;
pub mod chord_type;
pub mod collection;
pub mod duration_value;
pub mod error;
pub mod interval;
pub mod key;
pub mod midi;
pub mod mode;
pub mod notation_scientific;
pub mod note;
pub mod pcset;
pub mod pitch;
pub mod pitch_distance;
pub mod pitch_interval;
pub mod pitch_note;
pub mod progression;
pub mod range;
pub(crate) mod regexes;
pub mod rhythm_pattern;
pub mod roman_numeral;
pub mod scale;
pub mod scale_type;
pub mod time_signature;
pub mod voice_leading;
pub mod voicing;
pub mod voicing_dictionary;
