use crate::{
    pitch::{NoteCoordinates, Pitch, PitchCoordinates},
    pitch_interval::{IntervalType, IntoInterval, coord_to_interval},
};

const INTERVALS: [&str; 7] = ["1P", "2M", "3M", "4P", "5P", "6m", "7m"];

pub fn names() -> [&'static str; 7] {
    INTERVALS
}

pub use crate::pitch_interval::interval as get;

pub fn name(name: &str) -> Option<String> {
    name.into_interval().map(|i| i.name)
}

pub fn semitones(name: &str) -> Option<i32> {
    name.into_interval().map(|i| i.semitones)
}

pub fn quality(name: &str) -> Option<String> {
    name.into_interval().map(|i| i.q.to_string())
}

pub fn num(name: &str) -> Option<i32> {
    name.into_interval().map(|i| i.num)
}

pub fn simplify(name: &str) -> Option<String> {
    name.into_interval().map(|i| format!("{}{}", i.simple, i.q))
}

pub fn invert(name: &str) -> Option<String> {
    let i = name.into_interval()?;

    let step = (7 - i.step).rem_euclid(7);
    let alt = if i.kind == IntervalType::Perfectable {
        -i.alt
    } else {
        -(i.alt + 1)
    };

    let res = Pitch {
        step,
        alt,
        oct: Some(i.oct),
        dir: Some(i.dir),
    }
    .into_interval()?;

    Some(res.name)
}

const IN: [i32; 12] = [1, 2, 2, 3, 3, 4, 5, 5, 6, 6, 7, 7];
const IQ: [char; 12] = ['P', 'm', 'M', 'm', 'M', 'P', 'd', 'P', 'm', 'M', 'm', 'M'];

pub fn from_semitones(semitones: i32) -> String {
    let d = if semitones < 0 { -1 } else { 1 };
    let n = semitones.abs();
    let c = n.rem_euclid(12) as usize;
    let o = n.div_euclid(12);

    format!("{}{}", d * (IN[c] + 7 * o), IQ[c])
}

pub use crate::pitch_distance::distance as dist;

pub fn add(a: &str, b: &str) -> Option<String> {
    combinator(a, b, |a, b| NoteCoordinates(a.0 + b.0, a.1 + b.1))
}

pub fn subtract(a: &str, b: &str) -> Option<String> {
    combinator(a, b, |a, b| NoteCoordinates(a.0 - b.0, a.1 - b.1))
}

pub fn add_to(interval: &str) -> impl Fn(&str) -> Option<String> {
    move |other| add(interval, other)
}

pub fn transpose_fifths(interval: &str, fifths: i32) -> Option<String> {
    let ivl = interval.into_interval()?;

    let (n_fifths, n_octs) = ivl.coord.into();

    let new_coord = NoteCoordinates(n_fifths + fifths, n_octs);
    let pitch_coord = PitchCoordinates::Note(new_coord);

    Some(coord_to_interval(&pitch_coord, false)?.name)
}

fn combinator(
    a: &str,
    b: &str,
    f: impl Fn(NoteCoordinates, NoteCoordinates) -> NoteCoordinates,
) -> Option<String> {
    let coord_a = a.into_interval()?.coord;
    let coord_b = b.into_interval()?.coord;
    let coord = f(coord_a, coord_b);

    let pitch_coord = PitchCoordinates::Note(coord);

    Some(coord_to_interval(&pitch_coord, false)?.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::{Direction, NoteCoordinates};
    use crate::pitch_interval::Quality;

    fn map_str(input: &str, f: impl Fn(&str) -> Option<String>) -> Vec<String> {
        input.split(' ').map(|n| f(n).unwrap()).collect()
    }

    #[test]
    fn test_properties() {
        let p = get("P4").unwrap();
        let parts = p.parts();
        assert_eq!(parts.step, 3);
        assert_eq!(parts.alt, 0);
        assert_eq!(parts.oct, 0);
        assert_eq!(parts.dir, Direction::Up);
        assert_eq!(parts.name, "4P");
        assert_eq!(parts.num, 4);
        assert_eq!(parts.q, Quality::Perfect);
        assert_eq!(parts.kind, IntervalType::Perfectable);
        assert_eq!(parts.simple, 4);
        assert_eq!(parts.semitones, 5);
        assert_eq!(parts.chroma, 5);
        assert_eq!(parts.coord, NoteCoordinates(-1, 1));
    }

    #[test]
    fn test_shorthand_properties() {
        assert_eq!(name("d5"), Some("5d".to_string()));
        assert_eq!(num("d5"), Some(5));
        assert_eq!(quality("d5"), Some("d".to_string()));
        assert_eq!(semitones("d5"), Some(6));
    }

    #[test]
    fn test_distance() {
        assert_eq!(dist("C4", "G4"), "5P");
    }

    #[test]
    fn test_names() {
        assert_eq!(names(), ["1P", "2M", "3M", "4P", "5P", "6m", "7m"]);
    }

    #[test]
    fn test_simplify_intervals() {
        assert_eq!(
            map_str("1P 2M 3M 4P 5P 6M 7M", simplify),
            ["1P", "2M", "3M", "4P", "5P", "6M", "7M"]
        );
        assert_eq!(
            map_str("8P 9M 10M 11P 12P 13M 14M", simplify),
            ["8P", "2M", "3M", "4P", "5P", "6M", "7M"]
        );
        assert_eq!(
            map_str("1d 1P 1A 8d 8P 8A 15d 15P 15A", simplify),
            ["1d", "1P", "1A", "8d", "8P", "8A", "1d", "1P", "1A"]
        );
        assert_eq!(
            map_str("-1P -2M -3M -4P -5P -6M -7M", simplify),
            ["-1P", "-2M", "-3M", "-4P", "-5P", "-6M", "-7M"]
        );
        assert_eq!(
            map_str("-8P -9M -10M -11P -12P -13M -14M", simplify),
            ["-8P", "-2M", "-3M", "-4P", "-5P", "-6M", "-7M"]
        );
    }

    #[test]
    fn test_invert_intervals() {
        assert_eq!(
            map_str("1P 2M 3M 4P 5P 6M 7M", invert),
            ["1P", "7m", "6m", "5P", "4P", "3m", "2m"]
        );
        assert_eq!(
            map_str("1d 2m 3m 4d 5d 6m 7m", invert),
            ["1A", "7M", "6M", "5A", "4A", "3M", "2M"]
        );
        assert_eq!(
            map_str("1A 2A 3A 4A 5A 6A 7A", invert),
            ["1d", "7d", "6d", "5d", "4d", "3d", "2d"]
        );
        assert_eq!(
            map_str("-1P -2M -3M -4P -5P -6M -7M", invert),
            ["-1P", "-7m", "-6m", "-5P", "-4P", "-3m", "-2m"]
        );
        assert_eq!(
            map_str("8P 9M 10M 11P 12P 13M 14M", invert),
            ["8P", "14m", "13m", "12P", "11P", "10m", "9m"]
        );
    }

    #[test]
    fn test_from_semitones() {
        let map_semis =
            |semis: &[i32]| -> Vec<String> { semis.iter().map(|s| from_semitones(*s)).collect() };
        assert_eq!(
            map_semis(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
            ["1P", "2m", "2M", "3m", "3M", "4P", "5d", "5P", "6m", "6M", "7m", "7M"]
        );
        assert_eq!(
            map_semis(&[12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]),
            ["8P", "9m", "9M", "10m", "10M", "11P", "12d", "12P", "13m", "13M", "14m", "14M"]
        );
        assert_eq!(
            map_semis(&[0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11]),
            ["1P", "-2m", "-2M", "-3m", "-3M", "-4P", "-5d", "-5P", "-6m", "-6M", "-7m", "-7M"]
        );
        assert_eq!(
            map_semis(&[-12, -13, -14, -15, -16, -17, -18, -19, -20, -21, -22, -23]),
            ["-8P", "-9m", "-9M", "-10m", "-10M", "-11P", "-12d", "-12P", "-13m", "-13M", "-14m", "-14M"]
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(add("3m", "5P"), Some("7m".to_string()));
        let from_fifth: Vec<String> = names().iter().map(|n| add("5P", n).unwrap()).collect();
        assert_eq!(from_fifth, ["5P", "6M", "7M", "8P", "9M", "10m", "11P"]);
        let via_add_to: Vec<String> = names().iter().map(|n| add_to("5P")(n).unwrap()).collect();
        assert_eq!(via_add_to, ["5P", "6M", "7M", "8P", "9M", "10m", "11P"]);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract("5P", "3M"), Some("3m".to_string()));
        assert_eq!(subtract("3M", "5P"), Some("-3m".to_string()));
        let subs: Vec<String> = names().iter().map(|n| subtract("5P", n).unwrap()).collect();
        assert_eq!(subs, ["5P", "4P", "3m", "2M", "1P", "-2m", "-3m"]);
    }

    #[test]
    fn test_transpose_fifths() {
        assert_eq!(transpose_fifths("4P", 1), Some("8P".to_string()));
        let up: Vec<String> = [0, 1, 2, 3, 4]
            .iter()
            .map(|f| transpose_fifths("1P", *f).unwrap())
            .collect();
        assert_eq!(up, ["1P", "5P", "9M", "13M", "17M"]);
        let down: Vec<String> = [0, -1, -2, -3, -4]
            .iter()
            .map(|f| transpose_fifths("1P", *f).unwrap())
            .collect();
        assert_eq!(down, ["1P", "-5P", "-9M", "-13M", "-17M"]);
    }
}
