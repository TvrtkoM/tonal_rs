//! Create and manipulate rhythmic onset patterns.
//!
//! A [`RhythmPattern`] is a sequence of `0`/`1` steps where `1` marks an onset.
//! Build patterns from binary/hex numbers ([`binary`], [`hex`]), inter-onset
//! gaps ([`onsets`]), randomness ([`random`], [`probability`]) or the Euclidean
//! algorithm ([`euclid`]), and [`rotate`] them.

/// A rhythm as a sequence of steps, where `1` is an onset and `0` is a rest.
pub type RhythmPattern = Vec<u8>;

/// Build a pattern from the binary digits of each number.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::binary(&[13]), [1, 1, 0, 1]);
/// assert_eq!(rhythm_pattern::binary(&[12, 13]), [1, 1, 0, 0, 1, 1, 0, 1]);
/// ```
pub fn binary(numbers: &[u32]) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for number in numbers {
        for digit in format!("{number:b}").chars() {
            pattern.push(if digit == '1' { 1 } else { 0 });
        }
    }
    pattern
}

/// Build a pattern from a hexadecimal string, four steps per hex digit.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::hex("8f"), [1, 0, 0, 0, 1, 1, 1, 1]);
/// ```
pub fn hex(hex_number: &str) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for c in hex_number.chars() {
        let binary = match c.to_digit(16) {
            Some(digit) => format!("{digit:04b}"),
            None => "0000".to_string(),
        };
        for digit in binary.chars() {
            pattern.push(if digit == '1' { 1 } else { 0 });
        }
    }
    pattern
}

/// Build a pattern from inter-onset gaps: each number is the count of rests
/// following an onset.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::onsets(&[1, 2, 2, 1]), [1, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
/// ```
pub fn onsets(numbers: &[u32]) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for number in numbers {
        pattern.push(1);
        pattern.extend(std::iter::repeat_n(0, *number as usize));
    }
    pattern
}

/// Build a random pattern of the given length, using `rnd` (values in `[0, 1)`)
/// as the randomness source.
///
/// A step is an onset when `rnd()` is at least `probability`.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// // a constant source above the threshold marks every step as an onset
/// assert_eq!(rhythm_pattern::random_with(4, 0.5, || 1.0), [1, 1, 1, 1]);
/// ```
pub fn random_with(length: usize, probability: f64, mut rnd: impl FnMut() -> f64) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for _ in 0..length {
        pattern.push(if rnd() >= probability { 1 } else { 0 });
    }
    pattern
}

/// Build a random pattern of the given length using the `rand` crate.
///
/// Only available with the `random` feature enabled.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::random(8, 0.5).len(), 8);
/// ```
#[cfg(feature = "random")]
pub fn random(length: usize, probability: f64) -> RhythmPattern {
    random_with(length, probability, rand::random::<f64>)
}

/// Build a pattern where each step is an onset with its own probability, using
/// `rnd` as the randomness source.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(
///     rhythm_pattern::probability_with(&[0.5, 0.2, 0.0, 1.0, 0.0], || 0.5),
///     [1, 0, 0, 1, 0],
/// );
/// ```
pub fn probability_with(probabilities: &[f64], mut rnd: impl FnMut() -> f64) -> RhythmPattern {
    probabilities
        .iter()
        .map(|p| if rnd() <= *p { 1 } else { 0 })
        .collect()
}

/// Build a pattern where each step is an onset with its own probability, using
/// the `rand` crate.
///
/// Only available with the `random` feature enabled.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// // probability 1.0 is always an onset, regardless of the random draw
/// assert_eq!(rhythm_pattern::probability(&[1.0, 1.0, 1.0]), [1, 1, 1]);
/// ```
#[cfg(feature = "random")]
pub fn probability(probabilities: &[f64]) -> RhythmPattern {
    probability_with(probabilities, rand::random::<f64>)
}

/// Rotate a pattern by a number of steps (positive rotates right).
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::rotate(&[1, 0, 0, 1], 1), [1, 1, 0, 0]);
/// assert_eq!(rhythm_pattern::rotate(&[1, 0, 0, 1], -1), [0, 0, 1, 1]);
/// ```
pub fn rotate(pattern: &[u8], rotations: i32) -> RhythmPattern {
    let len = pattern.len();
    if len == 0 {
        return RhythmPattern::new();
    }
    let len_i = len as i32;
    (0..len_i)
        .map(|i| {
            let pos = (i - rotations).rem_euclid(len_i) as usize;
            pattern[pos]
        })
        .collect()
}

/// Build a Euclidean rhythm distributing `beats` onsets as evenly as possible
/// over `steps` steps.
///
/// ```rust
/// use tonal_rs::rhythm_pattern;
/// assert_eq!(rhythm_pattern::euclid(8, 3), [1, 0, 0, 1, 0, 0, 1, 0]);
/// ```
pub fn euclid(steps: usize, beats: usize) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    let mut d: i64 = -1;
    for i in 0..steps {
        let v = ((i as f64) * (beats as f64 / steps as f64)).floor() as i64;
        pattern.push(if v != d { 1 } else { 0 });
        d = v;
    }
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary() {
        assert_eq!(binary(&[13]), [1, 1, 0, 1]);
        assert_eq!(binary(&[12, 13]), [1, 1, 0, 0, 1, 1, 0, 1]);
    }

    #[test]
    fn test_hex() {
        assert_eq!(hex("8f"), [1, 0, 0, 0, 1, 1, 1, 1]);
    }

    #[test]
    fn test_onsets() {
        assert_eq!(onsets(&[1, 2, 2, 1]), [1, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
    }

    #[test]
    fn test_random_with() {
        let mut current = 0.25;
        let sequential = || {
            current += 0.1;
            current
        };
        assert_eq!(random_with(5, 0.5, sequential), [0, 0, 1, 1, 1]);
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_random() {
        let pattern = random(10, 0.5);
        assert_eq!(pattern.len(), 10);
    }

    #[test]
    fn test_probability_with() {
        assert_eq!(
            probability_with(&[0.5, 0.2, 0.0, 1.0, 0.0], || 0.5),
            [1, 0, 0, 1, 0]
        );
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_probability() {
        let pattern = probability(&[0.5, 0.2, 0.0, 1.0, 0.0]);
        assert_eq!(pattern.len(), 5);
    }

    #[test]
    fn test_rotate() {
        assert_eq!(rotate(&[1, 0, 0, 1], 0), [1, 0, 0, 1]);
        assert_eq!(rotate(&[1, 0, 0, 1], 1), [1, 1, 0, 0]);
        assert_eq!(rotate(&[1, 0, 0, 1], 2), [0, 1, 1, 0]);
        assert_eq!(rotate(&[1, 0, 0, 1], 3), [0, 0, 1, 1]);
        assert_eq!(rotate(&[1, 0, 0, 1], 4), [1, 0, 0, 1]);
        assert_eq!(rotate(&[1, 0, 0, 1], -1), [0, 0, 1, 1]);
        assert_eq!(rotate(&[1, 0, 0, 1], -2), [0, 1, 1, 0]);
    }

    #[test]
    fn test_euclid() {
        assert_eq!(euclid(8, 3), [1, 0, 0, 1, 0, 0, 1, 0]);
    }
}
