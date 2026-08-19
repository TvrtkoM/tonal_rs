use crate::{
    chord::tokenize, note::transpose, pitch_distance::distance, pitch_interval::IntoInterval,
    pitch_note::IntoNote, roman_numeral::IntoRomanNumeral,
};

pub fn from_roman_numerals<N>(tonic: N, chords: &[&str]) -> Vec<String>
where
    N: IntoNote + Clone,
{
    let roman_numerals: Vec<_> = chords.iter().map(|c| c.into_roman_numeral()).collect();

    roman_numerals
        .into_iter()
        .map(|roman| {
            roman
                .as_ref()
                .map(|rn| {
                    let tr = transpose(tonic.clone(), rn);
                    format!("{tr}{}", rn.chord_type)
                })
                .unwrap_or("".to_string())
        })
        .collect()
}

pub fn to_roman_numerals<N>(tonic: N, chords: &[&str]) -> Vec<String>
where
    N: IntoNote + Clone,
{
    chords
        .iter()
        .map(|chord| {
            let (note, chord_kind, _) = tokenize(chord);
            let interval_name = distance(tonic.clone(), note.as_str());
            let interval = interval_name.as_str().into_interval();
            let roman = interval.and_then(|ivl| ivl.into_roman_numeral());
            roman.map_or(chord_kind.clone(), |rn| format!("{}{chord_kind}", rn.name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> Vec<&str> {
        s.split(' ').collect()
    }

    fn in_c(chords: &str) -> Vec<String> {
        from_roman_numerals("C", &split(chords))
    }

    #[test]
    fn test_concrete() {
        assert_eq!(in_c("I IIm7 V7"), ["C", "Dm7", "G7"]);
        assert_eq!(in_c("Imaj7 2 IIIm7"), ["Cmaj7", "", "Em7"]);
        assert_eq!(in_c("I II III IV V VI VII"), ["C", "D", "E", "F", "G", "A", "B"]);
        assert_eq!(
            in_c("bI bII bIII bIV bV bVI bVII"),
            ["Cb", "Db", "Eb", "Fb", "Gb", "Ab", "Bb"]
        );
        assert_eq!(
            in_c("#Im7 #IIm7 #III #IVMaj7 #V7 #VI #VIIo"),
            ["C#m7", "D#m7", "E#", "F#Maj7", "G#7", "A#", "B#o"]
        );
    }

    #[test]
    fn test_abstract() {
        let roman = to_roman_numerals("C", &["Cmaj7", "Dm7", "G7"]);
        assert_eq!(roman, ["Imaj7", "IIm7", "V7"]);
    }
}
