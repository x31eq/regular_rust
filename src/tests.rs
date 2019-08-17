use super::*;

#[test]
fn octave_cents() {
    assert_eq!(cents(2.0), 1200.0);
    assert_eq!(cents(4.0), 2400.0);
}

#[test]
fn seven_limit() {
    assert_eq!(PrimeLimit::new(7).numbers,
                vec![2, 3, 5, 7]);
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
    for (r, p) in rust_generated.iter().cloned()
                    .zip(python_generated.iter().cloned()) {
        assert!(near_enough_equal(r, p));
    }
}

#[test]
fn small_primes() {
    let from_pari = vec![
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41,
        43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97];
    assert_eq!(primes_below(100), from_pari);
}

fn near_enough_equal(x: f64, y: f64) -> bool {
    (x/y - 1.0).abs() < 1e-15
}
