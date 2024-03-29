//! Utilities for dealing with vectors as ratios

use super::PrimeLimit;

/// Integers in ratios can get bigger than partials
type Length = i128;

/// Reverse engineer a prime limit object into a list of integers
fn integer_partials(limit: &PrimeLimit) -> Result<Vec<Length>, String> {
    limit
        .headings
        .iter()
        .map(|m| m.parse())
        .collect::<Result<Vec<Length>, _>>()
        .map_err(|_| "Harmonics should all be numbers".to_string())
}

#[test]
fn parse_7limit() {
    let limit7 = super::PrimeLimit::new(7);
    assert_eq!(integer_partials(&limit7), Ok(vec![2, 3, 5, 7]));
}
