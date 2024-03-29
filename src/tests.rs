use super::*;

#[test]
fn octave_cents() {
    assert_eq!(cents(2.0), 1200.0);
    assert_eq!(cents(4.0), 2400.0);
}

#[test]
fn seven_limit() {
    assert_eq!(&PrimeLimit::new(7).label, "7");
}

#[test]
fn non_consecutive_limit() {
    let primes = vec![2, 3, 7];
    let limit = PrimeLimit::explicit(primes);
    assert_eq!(&limit.label, "2.3.7");
}

#[test]
fn limit_from_str() {
    let label = "7";
    let limit: PrimeLimit =
        label.parse().expect("Failed to parse integer prime limit");
    assert_eq!(&limit.label, label);
    assert_eq!(limit.headings, vec!["2", "3", "5", "7"]);
}

#[test]
fn non_consecutive_limit_from_str() {
    let label = "2.3.7";
    let limit: PrimeLimit =
        label.parse().expect("Failed to parse dotted prime limit");
    assert_eq!(&limit.label, label);
    assert_eq!(limit.headings, vec!["2", "3", "7"]);
}

#[test]
fn bad_limit_from_str() {
    let label = "foo";
    let limit: Result<PrimeLimit, _> = label.parse();
    assert!(limit.is_err());
}

#[test]
fn test_join() {
    assert_eq!(&join(", ", &vec![1, 2, 3]), "1, 2, 3");
    let mut tokens = Vec::new();
    assert_eq!(&join(" ", &tokens), "");
    tokens.push("foo");
    assert_eq!(&join(" ", &tokens), "foo");
    tokens.push("bar");
    assert_eq!(&join(" ", &tokens), "foo bar");
}

#[test]
fn cents50() {
    let python_generated = vec![
        1200.0,
        1901.9550008653875,
        2786.3137138648344,
        3368.825906469125,
        4151.317942364757,
        4440.527661769311,
        4904.955409500408,
        5097.513016132302,
        5428.274347268416,
        5829.5771941530875,
        5945.035572464251,
        6251.34403875474,
        6429.0624055417,
        6511.517705642517,
        6665.506622013165,
    ];
    let rust_generated = PrimeLimit::new(50).pitches;
    for (r, p) in rust_generated.into_iter().zip(python_generated.into_iter())
    {
        assert!(near_enough_equal(r, p));
    }
}

#[test]
fn suboptimal_prime_mapping() {
    let limit13 = PrimeLimit::new(13).pitches;
    let p12 = prime_mapping(&limit13, 12);
    assert_eq!(p12, vec![12, 19, 28, 34, 42, 44]);
}

#[test]
fn small_primes() {
    let from_pari = vec![
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61,
        67, 71, 73, 79, 83, 89, 97,
    ];
    assert_eq!(primes_below(100), from_pari);
}

#[test]
fn hermite_reduction() {
    let jove = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let jove_hermite = vec![
        vec![1, 1, 1, 2, 2],
        vec![0, 2, 1, 1, 5],
        vec![0, 0, 2, 1, 0],
    ];
    assert_eq!(hermite_normal_form(&jove), jove_hermite);
    let jove_neg_hermite = vec![
        vec![1, -1, 0, 1, -3],
        vec![0, 2, -1, 0, 5],
        vec![0, 0, 2, 1, 0],
    ];
    assert_eq!(hermite_normal_form(&jove_neg_hermite), jove_hermite);
}

fn near_enough_equal(x: f64, y: f64) -> bool {
    (x / y - 1.0).abs() < 1e-15
}
