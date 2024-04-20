//! Utilities for dealing with vectors as ratios

use super::{join, ETMap, Mapping, PrimeLimit};

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

/// Turn the ratio encoded as a string into a vector
/// in the given prime limit.
/// Eventually, should work with vectors-as-strings as well
pub fn parse_as_vector(limit: &PrimeLimit, input: &str) -> Option<ETMap> {
    let input = input.trim();
    let (n, d): Ratio = if let Ok(n) = input.parse() {
        (n, 1)
    } else {
        let (sn, sd) = input.split_once([':', '/'])?;
        (sn.parse().ok()?, sd.parse().ok()?)
    };
    factorize_ratio(limit, (n, d))
}

fn factorize(limit: &PrimeLimit, n: Length) -> Option<ETMap> {
    if n == 0 {
        return None;
    }
    let partials = integer_partials(limit).ok()?;
    let mut result = vec![0; partials.len()];
    let mut remainder = n;
    for (i, &p) in partials.iter().enumerate() {
        while remainder.checked_rem(p) == Some(0) {
            remainder /= p;
            result[i] += 1;
        }
    }
    if remainder == 1 {
        Some(result)
    } else {
        None
    }
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

pub fn factorize_ratios_in_simplest_limit(
    ratios: &[Ratio],
) -> Option<(PrimeLimit, Mapping)> {
    // For now, primes over 100 will not be recognized
    let mut limit = PrimeLimit::new(100);
    let mut vectors = ratios
        .iter()
        .map(|&r| factorize_ratio(&limit, r))
        .collect::<Option<Mapping>>()?;
    let limit_size = limit.pitches.len();
    let trim_point = vectors
        .iter()
        .map(|interval| {
            interval.iter().rposition(|&n| n != 0).unwrap_or(limit_size)
        })
        .max()?;
    for vector in &mut vectors {
        vector.truncate(trim_point + 1);
    }
    limit.pitches.truncate(trim_point + 1);
    limit.headings.truncate(trim_point + 1);
    limit.label = limit
        .headings
        .get(trim_point)
        .expect("Over-truncated headings")
        .to_string();
    Some((limit, vectors))
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
fn parse_7_limit() {
    let limit7 = PrimeLimit::new(7);
    assert_eq!(integer_partials(&limit7), Ok(vec![2, 3, 5, 7]));
}

#[test]
fn parse_7_limit_ratios() {
    let limit = PrimeLimit::new(7);
    assert_eq!(parse_as_vector(&limit, "225:224"), Some(vec![-5, 2, 2, -1]),);
    assert_eq!(
        parse_as_vector(&limit, "2401:2400"),
        Some(vec![-5, -1, -2, 4]),
    );
    assert_eq!(parse_as_vector(&limit, "7:4"), Some(vec![-2, 0, 0, 1]),);
    assert_eq!(parse_as_vector(&limit, "7/4"), Some(vec![-2, 0, 0, 1]),);
    assert_eq!(
        parse_as_vector(&limit, "   7/4    "),
        Some(vec![-2, 0, 0, 1]),
    );
    assert_eq!(parse_as_vector(&limit, "-7:4"), None);
    assert_eq!(parse_as_vector(&limit, "99:100"), None);
    assert_eq!(parse_as_vector(&limit, "foo"), None);
}

#[test]
fn factorize_5_limit() {
    let limit = PrimeLimit::new(5);
    assert_eq!(factorize(&limit, 10), Some(vec![1, 0, 1]));
    assert_eq!(factorize(&limit, 60), Some(vec![2, 1, 1]));
    assert_eq!(factorize(&limit, 1), Some(vec![0, 0, 0]));
    assert_eq!(factorize(&limit, 0), None);
    assert_eq!(factorize(&limit, 7), None);
}

#[test]
fn factorize_13_limit() {
    let limit = PrimeLimit::new(13);
    assert_eq!(factorize(&limit, 1), Some(vec![0, 0, 0, 0, 0, 0]));
    assert_eq!(factorize(&limit, 26), Some(vec![1, 0, 0, 0, 0, 1]));
    assert_eq!(factorize(&limit, 60), Some(vec![2, 1, 1, 0, 0, 0]));
    assert_eq!(factorize(&limit, 30030), Some(vec![1, 1, 1, 1, 1, 1]));
    assert_eq!(factorize(&limit, 0), None);
    assert_eq!(factorize(&limit, 17), None);
}

#[test]
fn test_5_limit_ratios() {
    let limit = PrimeLimit::new(5);
    assert_eq!(factorize_ratio(&limit, (1, 1)), Some(vec![0, 0, 0]));
    assert_eq!(factorize_ratio(&limit, (3, 2)), Some(vec![-1, 1, 0]));
    assert_eq!(factorize_ratio(&limit, (5, 4)), Some(vec![-2, 0, 1]));
    assert_eq!(factorize_ratio(&limit, (81, 80)), Some(vec![-4, 4, -1]));
    assert_eq!(factorize_ratio(&limit, (225, 224)), None);
    assert_eq!(factorize_ratio(&limit, (1, 0)), None);
    assert_eq!(factorize_ratio(&limit, (0, 1)), None);
    assert_eq!(factorize_ratio(&limit, (0, 0)), None);
}

#[test]
fn test_13_limit_ratios() {
    let limit = PrimeLimit::new(13);
    assert_eq!(
        factorize_ratio(&limit, (1, 1)),
        Some(vec![0, 0, 0, 0, 0, 0]),
    );
    assert_eq!(
        factorize_ratio(&limit, (144, 143)),
        Some(vec![4, 2, 0, 0, -1, -1]),
    );
    assert_eq!(
        factorize_ratio(&limit, (143, 144)),
        Some(vec![-4, -2, 0, 0, 1, 1]),
    );
    assert_eq!(
        factorize_ratio(&limit, (225, 224)),
        Some(vec![-5, 2, 2, -1, 0, 0]),
    );
    assert_eq!(
        factorize_ratio(&limit, (100, 99)),
        Some(vec![2, -2, 2, 0, -1, 0]),
    );
    assert_eq!(factorize_ratio(&limit, (256, 255)), None);
    assert_eq!(factorize_ratio(&limit, (1, 0)), None);
    assert_eq!(factorize_ratio(&limit, (0, 1)), None);
    assert_eq!(factorize_ratio(&limit, (0, 0)), None);
}

#[test]
fn detect_5_limit() {
    let result = factorize_ratios_in_simplest_limit(&[(81, 80)]);
    assert!(!result.is_none());
    if let Some((limit, intervals)) = result {
        assert_eq!(limit.label, "5");
        assert_eq!(limit.headings, vec!["2", "3", "5"]);
        assert_eq!(intervals, vec![vec![-4, 4, -1]]);
    }
}

#[test]
fn detect_7_limit() {
    let result = factorize_ratios_in_simplest_limit(&[(7, 6)]);
    assert!(!result.is_none());
    if let Some((limit, intervals)) = result {
        assert_eq!(limit.label, "7");
        assert_eq!(limit.headings, vec!["2", "3", "5", "7"]);
        assert_eq!(intervals, vec![vec![-1, -1, 0, 1]]);
    }
}

#[test]
fn detect_13_limit() {
    let result = factorize_ratios_in_simplest_limit(&[
        (1001, 1000),
        (100, 99),
        (351, 350),
        (540, 539),
    ]);
    assert!(!result.is_none());
    if let Some((limit, intervals)) = result {
        assert_eq!(limit.label, "13");
        assert_eq!(limit.headings, vec!["2", "3", "5", "7", "11", "13"]);
        assert_eq!(
            intervals,
            vec![
                vec![-3, 0, -3, 1, 1, 1],
                vec![2, -2, 2, 0, -1, 0],
                vec![-1, 3, -2, -1, 0, 1],
                vec![2, 3, 1, -2, -1, 0],
            ],
        )
    }
}

/// Corner case: highest detectable limit
#[test]
fn detect_97_limit() {
    let result = factorize_ratios_in_simplest_limit(&[(100, 97)]);
    assert!(!result.is_none());
    if let Some((limit, intervals)) = result {
        assert_eq!(limit.label, "97");
        assert_eq!(
            intervals,
            vec![vec![
                2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, -1,
            ]],
        );
    }
}

/// Corner case: lowest undetectable limit
#[test]
fn detect_101_limit() {
    let result = factorize_ratios_in_simplest_limit(&[(100, 101)]);
    assert!(result.is_none());
}

#[test]
fn detect_silly_limit() {
    let result = factorize_ratios_in_simplest_limit(&[(65536, 65535)]);
    assert!(result.is_none());
}
