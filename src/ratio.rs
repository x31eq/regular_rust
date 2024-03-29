//! Utilities for dealing with vectors as ratios

use super::{join, ETMap, PrimeLimit};

/// Integers in ratios can get bigger than partials
type Length = i128;

/// Turn the ratio-space vector (typed as a mapping) into a ratio-string
pub fn get_ratio_string(limit: &PrimeLimit, rsvec: &ETMap) -> Option<String> {
    Some(stringify(get_ratio(limit, rsvec)?))
}

/// Turn the ratio-space vector (typed as a mapping) into a ratio
pub fn get_ratio(
    limit: &PrimeLimit,
    rsvec: &ETMap,
) -> Option<(Length, Length)> {
    let mut numerator: Length = 1;
    let mut denominator: Length = 1;
    let harmonics = integer_partials(limit).ok()?;
    for (&harmonic, &el) in harmonics.iter().zip(rsvec.iter()) {
        if el > 0 {
            numerator =
                numerator.checked_mul(harmonic.checked_pow(el as u32)?)?;
        }
        if el < 0 {
            denominator =
                denominator.checked_mul(harmonic.checked_pow(-el as u32)?)?;
        }
    }
    Some((numerator, denominator))
}

/// Turn the ratio-space vector (typed as a mapping) into a ratio-string
/// or a ket if this is not possible
pub fn get_ratio_or_ket_string(limit: &PrimeLimit, rsvec: &ETMap) -> String {
    match get_ratio(limit, rsvec) {
        Some(ratio) => stringify(ratio),
        None => format!("[{}>", join(", ", rsvec)),
    }
}

pub fn stringify(ratio: (Length, Length)) -> String {
    let (numerator, denominator) = ratio;
    format!("{}:{}", numerator, denominator)
}

/// Reverse engineer a prime limit object into a list of integers
fn integer_partials(
    limit: &PrimeLimit,
) -> Result<Vec<Length>, std::num::ParseIntError> {
    limit.headings.iter().map(|m| m.parse()).collect()
}

#[test]
fn get_syntonic_comma() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio, Some((81, 80)));
}

#[test]
fn get_syntonic_comma_string() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio_string = get_ratio_string(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio_string, Some("81:80".to_string()));
}

#[test]
fn get_syntonic_comma_string_or_ket() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio_string = get_ratio_or_ket_string(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio_string, "81:80");
}

#[test]
fn stringify_syntonic_comma() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-4, 4, -1]).expect("ratio overflow");
    assert_eq!(stringify(ratio), "81:80");
}

#[test]
fn get_major_third() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-2, 0, 1]);
    assert_eq!(ratio, Some((5, 4)));
}

#[test]
fn get_huge_interval() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![1000, -1000, 0]);
    assert_eq!(ratio, None);
}

#[test]
fn get_huge_interval_ket() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio_or_ket_string(&limit5, &vec![1000, -1000, 0]);
    assert_eq!(ratio, "[1000, -1000, 0>");
}

#[test]
fn parse_7limit() {
    let limit7 = super::PrimeLimit::new(7);
    assert_eq!(integer_partials(&limit7), Ok(vec![2, 3, 5, 7]));
}
