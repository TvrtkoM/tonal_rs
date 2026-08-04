use rand::{Rng, seq::SliceRandom};

pub fn range(from: i32, to: i32) -> Vec<i32> {
    if from < to {
        (from..=to).collect()
    } else {
        (to..=from).rev().collect()
    }
}

pub fn rotate<T>(times: i32, vec: &mut [T]) {
    let len = vec.len();
    if len == 0 {
        return;
    }
    let n = times.rem_euclid(len as i32) as usize;
    vec.rotate_left(n);
}

pub fn rotated<T: Clone>(times: i32, vec: &[T]) -> Vec<T> {
    let mut out = vec.to_vec();
    rotate(times, &mut out);
    out
}

pub fn compacted<T: Clone>(vec: &[Option<T>]) -> Vec<T> {
    vec.iter().flatten().cloned().collect()
}

pub fn compact<T>(vec: Vec<Option<T>>) -> Vec<T> {
    vec.into_iter().flatten().collect()
}

pub fn shuffle_with<T, R: Rng + ?Sized>(vec: &mut [T], rng: &mut R) {
    vec.shuffle(rng);
}

pub fn shuffle<T>(vec: &mut [T]) {
    shuffle_with(vec, &mut rand::rng());
}

pub fn permutations<T: Clone>(vec: &[T]) -> Vec<Vec<T>> {
    let Some((first, rest)) = vec.split_first() else {
        return vec![vec![]];
    };

    let n = vec.len();
    let mut result: Vec<Vec<T>> = Vec::new();
    for perm in permutations(rest) {
        for pos in 0..n {
            let mut new_perm = perm.clone();
            new_perm.insert(pos, first.clone());
            result.push(new_perm)
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    // split a space-separated string into words — mirrors the TS `$` helper.
    fn words(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    #[test]
    fn test_range() {
        assert_eq!(range(-2, 2), [-2, -1, 0, 1, 2]);
        assert_eq!(range(2, -2), [2, 1, 0, -1, -2]);
    }

    #[test]
    fn test_rotate() {
        // TS `rotate` returns a new array — that's `rotated` here.
        assert_eq!(rotated(2, &words("a b c d e")), words("c d e a b"));
        // the in-place variant mutates the slice
        let mut v = words("a b c d e");
        rotate(2, &mut v);
        assert_eq!(v, words("c d e a b"));
        // empty is a no-op (no divide-by-zero panic)
        let mut empty: Vec<&str> = vec![];
        rotate(2, &mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_compact() {
        // TS filters JS falsy values; here `null` is modeled as `None` over Option<T>.
        assert_eq!(
            compacted(&[Some("a"), Some("b"), None, Some("c")]),
            words("a b c")
        );
        // consuming variant; note Some(0) is kept, like TS keeps 0.
        assert_eq!(
            compact(vec![Some(1), None, Some(0), None, Some(3)]),
            [1, 0, 3]
        );
    }

    #[test]
    fn test_shuffle() {
        // Can't reproduce the TS rnd=0.2 output (different RNG), so assert the
        // invariant: shuffle is a permutation (same multiset), deterministic per seed.
        let mut v = words("a b c d");
        let original = v.clone();
        let mut rng = StdRng::seed_from_u64(42);
        shuffle_with(&mut v, &mut rng);

        let mut got = v.clone();
        got.sort();
        let mut want = original.clone();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn test_permutations() {
        assert_eq!(
            permutations(&words("a b c")),
            vec![
                words("a b c"),
                words("b a c"),
                words("b c a"),
                words("a c b"),
                words("c a b"),
                words("c b a"),
            ]
        );
        // edge cases
        assert_eq!(permutations::<&str>(&[]), vec![Vec::<&str>::new()]);
        assert_eq!(permutations(&["a"]), vec![vec!["a"]]);
    }
}
