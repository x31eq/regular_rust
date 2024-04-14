//! Utilities for dealing with vectors as ratios

use super::{join, ETMap, PrimeLimit};

/// Integers in ratios can get bigger than partials
type Length = u128;
type Ratio = (Length, Length);

/// Turn the ratio-space vector (typed as a mapping) into a ratio-string
pub fn get_ratio_string(limit: &PrimeLimit, rsvec: &ETMap) -> Option<String> {
    Some(stringify(get_ratio(limit, rsvec)?))
}

/// Turn the ratio-space vector (typed as a mapping) into a ratio
pub fn get_ratio(limit: &PrimeLimit, rsvec: &ETMap) -> Option<Ratio> {
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
        None => format!("[{}⟩", join(", ", rsvec)),
    }
}

pub fn stringify(ratio: Ratio) -> String {
    let (numerator, denominator) = ratio;
    format!("{}:{}", numerator, denominator)
}

fn factorize(limit: &PrimeLimit, n: Length) -> Option<ETMap> {
    let partials = integer_partials(limit).ok()?;
    let mut result = vec![0; partials.len()];
    let mut remainder = n;
    for (i, &p) in partials.iter().enumerate() {
        while remainder.checked_rem(p) == Some(0) {
            remainder /= p;
            result[i] += 1;
        }
    }
    Some(result)
}

pub fn factorize_ratio(limit: &PrimeLimit, (n, d): Ratio) -> Option<ETMap> {
    let numerator = factorize(limit, n)?;
    let denominator = factorize(limit, d)?;
    Some(
        numerator
            .iter()
            .zip(denominator.iter())
            .map(|(&a, &b)| a - b)
            .collect(),
    )
}

/// Turn the ratio encoded as a string into a vector
/// in the given prime limit.
/// Eventually, should work with vectors-as-strings as well
pub fn parse_as_vector(limit: &PrimeLimit, input: &str) -> Option<ETMap> {
    let input = input.trim();
    let (n, d): Ratio = if let Ok(n) = input.parse() {
        (n, 1)
    } else {
        let (sn, sd) = input.split_once(&[':', '/'])?;
        (sn.parse().ok()?, sd.parse().ok()?)
    };
    factorize_ratio(&limit, (n, d))
}

/// Reverse engineer a prime limit object into a list of integers
fn integer_partials(
    limit: &PrimeLimit,
) -> Result<Vec<Length>, std::num::ParseIntError> {
    limit.headings.iter().map(|m| m.parse()).collect()
}

#[test]
fn get_syntonic_comma() {
    let limit5 = PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio, Some((81, 80)));
}

#[test]
fn get_syntonic_comma_string() {
    let limit5 = PrimeLimit::new(5);
    let ratio_string = get_ratio_string(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio_string, Some("81:80".to_string()));
}

#[test]
fn get_syntonic_comma_string_or_ket() {
    let limit5 = PrimeLimit::new(5);
    let ratio_string = get_ratio_or_ket_string(&limit5, &vec![-4, 4, -1]);
    assert_eq!(ratio_string, "81:80");
}

#[test]
fn stringify_syntonic_comma() {
    let limit5 = PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-4, 4, -1]).expect("ratio overflow");
    assert_eq!(stringify(ratio), "81:80");
}

#[test]
fn get_major_third() {
    let limit5 = PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![-2, 0, 1]);
    assert_eq!(ratio, Some((5, 4)));
}

#[test]
fn get_huge_interval() {
    let limit5 = PrimeLimit::new(5);
    let ratio = get_ratio(&limit5, &vec![1000, -1000, 0]);
    assert_eq!(ratio, None);
}

#[test]
fn get_huge_interval_ket() {
    let limit5 = PrimeLimit::new(5);
    let ratio = get_ratio_or_ket_string(&limit5, &vec![1000, -1000, 0]);
    assert_eq!(ratio, "[1000, -1000, 0⟩");
}

#[test]
fn parse_7limit() {
    let limit7 = PrimeLimit::new(7);
    assert_eq!(integer_partials(&limit7), Ok(vec![2, 3, 5, 7]));
}

#[test]
fn factorize_5_limit() {
    let limit = PrimeLimit::new(5);
    assert_eq!(factorize(&limit, 10), Some(vec![1, 0, 1]));
}
