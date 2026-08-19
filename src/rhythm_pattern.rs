pub type RhythmPattern = Vec<u8>;

pub fn binary(numbers: &[u32]) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for number in numbers {
        for digit in format!("{number:b}").chars() {
            pattern.push(if digit == '1' { 1 } else { 0 });
        }
    }
    pattern
}

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

pub fn onsets(numbers: &[u32]) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for number in numbers {
        pattern.push(1);
        pattern.extend(std::iter::repeat_n(0, *number as usize));
    }
    pattern
}

pub fn random(length: usize, probability: f64, mut rnd: impl FnMut() -> f64) -> RhythmPattern {
    let mut pattern = RhythmPattern::new();
    for _ in 0..length {
        pattern.push(if rnd() >= probability { 1 } else { 0 });
    }
    pattern
}

pub fn probability(probabilities: &[f64], mut rnd: impl FnMut() -> f64) -> RhythmPattern {
    probabilities
        .iter()
        .map(|p| if rnd() <= *p { 1 } else { 0 })
        .collect()
}

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
    use rand::RngExt;

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
    fn test_random() {
        let mut rng = rand::rng();
        let pattern = random(10, 0.5, || rng.random::<f64>());
        assert_eq!(pattern.len(), 10);

        let mut current = 0.25;
        let sequential = || {
            current += 0.1;
            current
        };
        assert_eq!(random(5, 0.5, sequential), [0, 0, 1, 1, 1]);
    }

    #[test]
    fn test_probability() {
        assert_eq!(
            probability(&[0.5, 0.2, 0.0, 1.0, 0.0], || 0.5),
            [1, 0, 0, 1, 0]
        );
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
