use std::sync::LazyLock;

use regex::Regex;

pub type Fraction = (f64, f64);

#[derive(Debug, Clone)]
pub struct DurationValue {
    pub value: f64,
    pub name: String,
    pub fraction: Fraction,
    pub shorthand: &'static str,
    pub dots: String,
    pub names: &'static [&'static str],
}

static VALUES: LazyLock<Vec<DurationValue>> = LazyLock::new(build_values);

fn build_values() -> Vec<DurationValue> {
    let mut v: Vec<DurationValue> = vec![];
    for &(denominator, shorthand, names) in DATA {
        let value = 1. / denominator;
        v.push(DurationValue {
            value,
            name: String::new(),
            fraction: if denominator < 1. {
                (value, 1.)
            } else {
                (1., denominator)
            },
            shorthand,
            dots: String::new(),
            names,
        });
    }
    v
}

pub fn names() -> Vec<&'static str> {
    let mut res: Vec<&'static str> = vec![];

    for duration in VALUES.iter() {
        res.extend_from_slice(duration.names);
    }

    res
}

pub fn shorthands() -> Vec<&'static str> {
    VALUES.iter().map(|v| v.shorthand).collect()
}

static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([^.]+)(\.*)$").unwrap());

pub fn get(name: &str) -> Option<DurationValue> {
    let c = REGEX.captures(name)?;
    let simple = &c[1];
    let dots = c[2].to_string();
    let base = VALUES
        .iter()
        .find(|dur| dur.shorthand == simple || dur.names.contains(&simple))?;
    let fraction = calc_dots(base.fraction, dots.len());
    let value = fraction.0 / fraction.1;
    Some(DurationValue {
        value,
        name: name.to_string(),
        fraction,
        shorthand: base.shorthand,
        dots,
        names: base.names,
    })
}

pub fn value(name: &str) -> f64 {
    get(name).map_or(0., |d| d.value)
}

pub fn fraction(name: &str) -> Fraction {
    get(name).map_or((0., 0.), |d| d.fraction)
}

fn calc_dots(fraction: Fraction, dots: usize) -> Fraction {
    let pow = (dots as f64).exp2();

    let mut numerator = fraction.0 * pow;
    let mut denominator = fraction.1 * pow;

    let base = numerator;

    for i in 0..dots {
        numerator += base / (i as f64 + 1.).exp2();
    }

    while numerator.rem_euclid(2.) == 0. && denominator.rem_euclid(2.) == 0. {
        numerator /= 2.;
        denominator /= 2.;
    }

    (numerator, denominator)
}

// source: https://en.wikipedia.org/wiki/Note_value
#[rustfmt::skip]
const DATA: &[(f64, &str, &[&str])] = &[
    (0.125, "dl", &["large", "duplex longa", "maxima", "octuple", "octuple whole"]),
    (0.25, "l", &["long", "longa"]),
    (0.5, "d", &["double whole", "double", "breve"]),
    (1.0, "w", &["whole", "semibreve"]),
    (2.0, "h", &["half", "minim"]),
    (4.0, "q", &["quarter", "crotchet"]),
    (8.0, "e", &["eighth", "quaver"]),
    (16.0, "s", &["sixteenth", "semiquaver"]),
    (32.0, "t", &["thirty-second", "demisemiquaver"]),
    (64.0, "sf", &["sixty-fourth", "hemidemisemiquaver"]),
    (128.0, "h", &["hundred twenty-eighth"]),
    (256.0, "th", &["two hundred fifty-sixth"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shorthand() {
        let q = get("q").unwrap();
        assert_eq!(q.name, "q");
        assert_eq!(q.value, 0.25);
        assert_eq!(q.fraction, (1., 4.));
        assert_eq!(q.dots, "");
        assert_eq!(q.shorthand, "q");
        assert_eq!(q.names, ["quarter", "crotchet"]);

        let dl = get("dl.").unwrap();
        assert_eq!(dl.name, "dl.");
        assert_eq!(dl.dots, ".");
        assert_eq!(dl.value, 12.);
        assert_eq!(dl.fraction, (12., 1.));
        assert_eq!(
            dl.names,
            [
                "large",
                "duplex longa",
                "maxima",
                "octuple",
                "octuple whole"
            ]
        );
        assert_eq!(dl.shorthand, "dl");
    }

    #[test]
    fn test_get_long_name() {
        let d = get("large.").unwrap();
        assert_eq!(d.name, "large.");
        assert_eq!(d.shorthand, "dl");
    }

    #[test]
    fn test_value() {
        let map = |names: &[&str]| -> Vec<f64> { names.iter().map(|n| value(n)).collect() };
        assert_eq!(map(&["dl", "dl.", "dl..", "dl..."]), [8., 12., 14., 15.]);
        assert_eq!(map(&["l", "l.", "l..", "l..."]), [4., 6., 7., 7.5]);
        assert_eq!(
            map(&["q", "q.", "q..", "q..."]),
            [0.25, 0.375, 0.4375, 0.46875]
        );
    }

    #[test]
    fn test_fraction() {
        let fr: Vec<Fraction> = ["w", "w.", "w..", "w..."]
            .iter()
            .map(|n| fraction(n))
            .collect();
        assert_eq!(fr, [(1., 1.), (3., 2.), (7., 4.), (15., 8.)]);
    }

    #[test]
    fn test_shorthands() {
        assert_eq!(shorthands().join(","), "dl,l,d,w,h,q,e,s,t,sf,h,th");
    }

    #[test]
    fn test_names() {
        assert_eq!(
            names().join(","),
            "large,duplex longa,maxima,octuple,octuple whole,long,longa,double whole,double,breve,whole,semibreve,half,minim,quarter,crotchet,eighth,quaver,sixteenth,semiquaver,thirty-second,demisemiquaver,sixty-fourth,hemidemisemiquaver,hundred twenty-eighth,two hundred fifty-sixth"
        );
    }
}
