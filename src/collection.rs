//! Generic collection helpers used across the crate.
//!
//! Small array utilities: inclusive integer [`range`], in-place and copying
//! [`rotate`]/[`rotated`], `None`-dropping [`compact`]/[`compacted`],
//! [`shuffle`] and [`permutations`].

/// An inclusive range of integers, ascending or descending.
///
/// ```rust
/// use tonal_rs::collection;
/// assert_eq!(collection::range(-2, 2), [-2, -1, 0, 1, 2]);
/// assert_eq!(collection::range(2, -2), [2, 1, 0, -1, -2]);
/// ```
pub fn range(from: i32, to: i32) -> Vec<i32> {
    if from < to {
        (from..=to).collect()
    } else {
        (to..=from).rev().collect()
    }
}

/// Rotate a slice left by `times` positions, in place (wrapping and handling
/// negatives).
///
/// ```rust
/// use tonal_rs::collection;
/// let mut v = vec!["a", "b", "c", "d", "e"];
/// collection::rotate(2, &mut v);
/// assert_eq!(v, ["c", "d", "e", "a", "b"]);
/// ```
pub fn rotate<T>(times: i32, vec: &mut [T]) {
    let len = vec.len();
    if len == 0 {
        return;
    }
    let n = times.rem_euclid(len as i32) as usize;
    vec.rotate_left(n);
}

/// Like [`rotate`], but returns a new rotated `Vec` and leaves the input
/// untouched.
///
/// ```rust
/// use tonal_rs::collection;
/// assert_eq!(collection::rotated(2, &["a", "b", "c", "d", "e"]), ["c", "d", "e", "a", "b"]);
/// ```
pub fn rotated<T: Clone>(times: i32, vec: &[T]) -> Vec<T> {
    let mut out = vec.to_vec();
    rotate(times, &mut out);
    out
}

/// Collect the `Some` values of a slice of `Option`s into a new `Vec` (by
/// cloning).
///
/// ```rust
/// use tonal_rs::collection;
/// assert_eq!(collection::compacted(&[Some("a"), None, Some("c")]), ["a", "c"]);
/// ```
pub fn compacted<T: Clone>(vec: &[Option<T>]) -> Vec<T> {
    vec.iter().flatten().cloned().collect()
}

/// Collect the `Some` values of an owned `Vec` of `Option`s, consuming it.
///
/// ```rust
/// use tonal_rs::collection;
/// assert_eq!(collection::compact(vec![Some(1), None, Some(3)]), [1, 3]);
/// ```
pub fn compact<T>(vec: Vec<Option<T>>) -> Vec<T> {
    vec.into_iter().flatten().collect()
}

/// In-place Fisher–Yates shuffle driven by `rnd`, which must yield values in
/// `[0, 1)` (each call supplies the randomness for one swap).
///
/// ```rust
/// use tonal_rs::collection;
/// let mut v = vec![1, 2, 3, 4];
/// // a constant source that maps every swap to the identity leaves order intact
/// collection::shuffle_with(&mut v, || 1.0);
/// assert_eq!(v, [1, 2, 3, 4]);
/// ```
pub fn shuffle_with<T>(vec: &mut [T], mut rnd: impl FnMut() -> f64) {
    for i in (1..vec.len()).rev() {
        // map rnd() in [0, 1) onto an index in 0..=i
        let j = ((rnd() * (i as f64 + 1.0)) as usize).min(i);
        vec.swap(i, j);
    }
}

/// Shuffle a slice in place using the `rand` crate.
///
/// Only available with the `random` feature enabled.
///
/// ```rust
/// use tonal_rs::collection;
/// let mut v = vec![1, 2, 3, 4];
/// collection::shuffle(&mut v);
/// assert_eq!(v.len(), 4);
/// ```
#[cfg(feature = "random")]
pub fn shuffle<T>(vec: &mut [T]) {
    shuffle_with(vec, rand::random::<f64>);
}

/// All permutations of a slice.
///
/// ```rust
/// use tonal_rs::collection;
/// assert_eq!(
///     collection::permutations(&["a", "b"]),
///     vec![vec!["a", "b"], vec!["b", "a"]],
/// );
/// ```
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
    fn test_shuffle_with() {
        // assert the invariant: shuffle is a permutation (same multiset).
        // a deterministic pseudo-random source keeps the test reproducible.
        let mut seed = 0.123_f64;
        let rnd = || {
            seed = (seed * 7.0 + 0.31).fract();
            seed
        };
        let mut v = words("a b c d");
        let original = v.clone();
        shuffle_with(&mut v, rnd);

        let mut got = v.clone();
        got.sort();
        let mut want = original.clone();
        want.sort();
        assert_eq!(got, want);
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_shuffle() {
        let mut v = words("a b c d");
        let original = v.clone();
        shuffle(&mut v);

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
