# tonal_rs

A music theory library for Rust — a port of the JavaScript
[tonal](https://github.com/tonaljs/tonal) library.

It provides functions to manipulate the tonal elements of music: notes,
intervals, chords, scales, modes and keys. It deals with abstractions, not with
audio or sound. The API is functional: functions are pure, there is no data
mutation, and musical entities are plain data structures.

## Install

```sh
cargo add tonal_rs
```

or add it to `Cargo.toml` manually:

```toml
[dependencies]
tonal_rs = "0.1"
```

## Example

```rust
use tonal_rs::{chord, interval, note, scale};

// Notes
assert_eq!(note::midi("C4"), Some(60));
assert_eq!(note::freq("A4"), Some(440.0));
assert_eq!(note::transpose("C4", "5P"), "G4");

// Intervals
assert_eq!(interval::semitones("5P"), Some(7));
assert_eq!(note::distance("C4", "G4"), "5P");

// Scales
assert_eq!(
    scale::get("C major").unwrap().parts().notes.to_vec(),
    ["C", "D", "E", "F", "G", "A", "B"],
);

// Chords
assert_eq!(
    chord::get("Cmaj7").unwrap().parts().notes.to_vec(),
    ["C", "E", "G", "B"],
);
```

## Modules

Functionality is grouped into modules that mirror the tonal packages:

- `note`, `interval`, `pitch_distance` — note/interval properties and arithmetic
- `chord`, `chord_type`, `chord_detect` — build, describe and detect chords
- `scale`, `scale_type`, `mode` — scales and the diatonic modes
- `key` — major and minor keys and their chords
- `pcset` — pitch-class sets and set-theory operations
- `roman_numeral`, `progression` — roman numeral analysis and chord progressions
- `voicing`, `voicing_dictionary`, `voice_leading` — chord voicings
- `midi`, `range`, `collection` — MIDI conversions, ranges and array helpers
- `duration_value`, `rhythm_pattern`, `time_signature` — rhythmic values
- `abc_notation`, `notation_scientific` — notation format conversions

## Features

- `random` (off by default) — enables randomized helpers (`collection::shuffle`,
  `rhythm_pattern::random`, `rhythm_pattern::probability`) via the `rand` crate.

Enable it when adding the crate:

```sh
cargo add tonal_rs --features random
```

or in `Cargo.toml`:

```toml
[dependencies]
tonal_rs = { version = "0.1", features = ["random"] }
```

## Documentation

API docs are published on [docs.rs](https://docs.rs/tonal_rs). Every public
function carries a runnable example; run them with `cargo test --doc`.

## License

Licensed under the [MIT License](LICENSE).

This crate is a port of [tonaljs](https://github.com/tonaljs/tonal), which is
also MIT-licensed.
