use super::*;

// Pitch classes
fn c() -> Pitch {
    Pitch { step: 0, alt: 0, oct: None, dir: None }
}
fn cs() -> Pitch {
    Pitch { step: 0, alt: 1, oct: None, dir: None }
}
fn cb() -> Pitch {
    Pitch { step: 0, alt: -1, oct: None, dir: None }
}
fn a() -> Pitch {
    Pitch { step: 5, alt: 0, oct: None, dir: None }
}

// Notes
fn c4() -> Pitch {
    Pitch { step: 0, alt: 0, oct: Some(4), dir: None }
}
fn a4() -> Pitch {
    Pitch { step: 5, alt: 0, oct: Some(4), dir: None }
}
fn gs6() -> Pitch {
    Pitch { step: 4, alt: 1, oct: Some(6), dir: None }
}

// Intervals
fn p5() -> Pitch {
    Pitch { step: 4, alt: 0, oct: Some(0), dir: Some(Direction::Up) }
}
fn p_5() -> Pitch {
    Pitch { step: 4, alt: 0, oct: Some(0), dir: Some(Direction::Down) }
}

#[test]
fn test_height() {
    let pcs: Vec<i32> = [c(), cs(), cb(), a()].iter().map(Pitch::height).collect();
    assert_eq!(pcs, [-1200, -1199, -1201, -1191]);

    let notes: Vec<i32> = [c4(), a4(), gs6()].iter().map(Pitch::height).collect();
    assert_eq!(notes, [48, 57, 80]);

    let intervals: Vec<i32> = [p5(), p_5()].iter().map(Pitch::height).collect();
    assert_eq!(intervals, [7, -7]);
}

#[test]
fn test_midi() {
    let pcs: Vec<Option<i32>> = [c(), cs(), cb(), a()].iter().map(Pitch::midi).collect();
    assert_eq!(pcs, [None, None, None, None]);

    let notes: Vec<Option<i32>> = [c4(), a4(), gs6()].iter().map(Pitch::midi).collect();
    assert_eq!(notes, [Some(60), Some(69), Some(92)]);
}

#[test]
fn test_chroma() {
    let pcs: Vec<i32> = [c(), cs(), cb(), a()].iter().map(Pitch::chroma).collect();
    assert_eq!(pcs, [0, 1, 11, 9]);

    let notes: Vec<i32> = [c4(), a4(), gs6()].iter().map(Pitch::chroma).collect();
    assert_eq!(notes, [0, 9, 8]);

    let intervals: Vec<i32> = [p5(), p_5()].iter().map(Pitch::chroma).collect();
    assert_eq!(intervals, [7, 7]);
}

#[test]
fn test_coordinates() {
    // pitch classes
    assert_eq!(c().coordinates(), PitchCoordinates::PitchClass(0));
    assert_eq!(a().coordinates(), PitchCoordinates::PitchClass(3));
    assert_eq!(cs().coordinates(), PitchCoordinates::PitchClass(7));
    assert_eq!(cb().coordinates(), PitchCoordinates::PitchClass(-7));
    // notes
    assert_eq!(c4().coordinates(), PitchCoordinates::Note(0, 4));
    assert_eq!(a4().coordinates(), PitchCoordinates::Note(3, 3));
    // intervals (direction is folded into the sign, like the TS version)
    assert_eq!(p5().coordinates(), PitchCoordinates::Note(1, 0));
    // TS expects [-1, -0]; Rust integers have no negative zero, so this is 0.
    assert_eq!(p_5().coordinates(), PitchCoordinates::Note(-1, 0));
}

#[test]
fn test_from_coordinates() {
    assert_eq!(Pitch::from(PitchCoordinates::PitchClass(0)), c());
    assert_eq!(Pitch::from(PitchCoordinates::PitchClass(7)), cs());
}

#[test]
fn test_accessors() {
    let p = gs6();
    assert_eq!(p.step(), 4);
    assert_eq!(p.alt(), 1);
    assert_eq!(p.oct(), Some(6));
    assert_eq!(p.dir(), None);
    assert_eq!(p.parts(), (4, 1, Some(6), None));

    assert_eq!(p_5().dir(), Some(Direction::Down));
}
