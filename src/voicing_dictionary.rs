use std::sync::LazyLock;

use crate::chord;

pub type VoicingDictionary = [(&'static str, &'static [&'static str])];

#[rustfmt::skip]
pub const TRIADS: &VoicingDictionary = &[
    ("M",   &["1P 3M 5P", "3M 5P 8P", "5P 8P 10M"]),
    ("m",   &["1P 3m 5P", "3m 5P 8P", "5P 8P 10m"]),
    ("o",   &["1P 3m 5d", "3m 5d 8P", "5d 8P 10m"]),
    ("aug", &["1P 3M 5A", "3M 5A 8P", "5A 8P 10M"]),
];

#[rustfmt::skip]
pub const LEFTHAND: &VoicingDictionary = &[
    ("m7",   &["3m 5P 7m 9M", "7m 9M 10m 12P"]),
    ("7",    &["3M 6M 7m 9M", "7m 9M 10M 13M"]),
    ("^7",   &["3M 5P 7M 9M", "7M 9M 10M 12P"]),
    ("69",   &["3M 5P 6A 9M"]),
    ("m7b5", &["3m 5d 7m 8P", "7m 8P 10m 12d"]),
    ("7b9",  &["3M 6m 7m 9m", "7m 9m 10M 13m"]),
    ("7b13", &["3M 6m 7m 9m", "7m 9m 10M 13m"]),
    ("o7",   &["1P 3m 5d 6M", "5d 6M 8P 10m"]),
    ("7#11", &["7m 9M 11A 13A"]),
    ("7#9",  &["3M 7m 9A"]),
    ("mM7",  &["3m 5P 7M 9M", "7M 9M 10m 12P"]),
    ("m6",   &["3m 5P 6M 9M", "6M 9M 10m 12P"]),
];

static ALL: LazyLock<Vec<(&'static str, &'static [&'static str])>> =
    LazyLock::new(|| TRIADS.iter().chain(LEFTHAND.iter()).copied().collect());

pub fn all() -> &'static VoicingDictionary {
    &ALL
}

pub const DEFAULT_DICTIONARY: &VoicingDictionary = LEFTHAND;

pub fn lookup(symbol: &str, dictionary: &VoicingDictionary) -> Option<Vec<String>> {
    if let Some((_, voicings)) = dictionary.iter().find(|(sym, _)| *sym == symbol) {
        return Some(voicings.iter().map(|s| s.to_string()).collect());
    }

    let name = format!("C{symbol}");
    let aliases = chord::get(name.as_str())?.aliases;

    dictionary
        .iter()
        .find(|(sym, _)| aliases.iter().any(|a| a.as_str() == *sym))
        .map(|(_, voicings)| voicings.iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        assert_eq!(
            lookup("M", TRIADS),
            Some(vec![
                "1P 3M 5P".to_string(),
                "3M 5P 8P".to_string(),
                "5P 8P 10M".to_string(),
            ])
        );

        assert_eq!(
            lookup("", TRIADS),
            Some(vec![
                "1P 3M 5P".to_string(),
                "3M 5P 8P".to_string(),
                "5P 8P 10M".to_string(),
            ])
        );

        const CUSTOM: &VoicingDictionary = &[("minor", &["1P 3m 5P"])];
        assert_eq!(lookup("minor", CUSTOM), Some(vec!["1P 3m 5P".to_string()]));
    }
}
