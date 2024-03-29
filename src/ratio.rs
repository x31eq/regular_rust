//! Utilities for dealing with vectors as ratios

use super::{ETMap, PrimeLimit};

/// Integers in ratios can get bigger than partials
type Length = i128;

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
fn get_huge_interval() {
    let limit5 = super::PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![1000, -1000, 0]);
    assert_eq!(ratio, None);
}

#[test]
fn parse_7limit() {
    let limit7 = super::PrimeLimit::new(7);
    assert_eq!(integer_partials(&limit7), Ok(vec![2, 3, 5, 7]));
}
