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
fn ratio_limit() {
    let label = "2.3.7/5";
    let limit: Result<PrimeLimit, _> = label.parse();
    assert!(limit.is_ok());
    let limit = limit.unwrap();
    assert_eq!(&limit.label, "2.3.7/5");
    assert_eq!(limit.pitches[0], 1200.0);
    assert!(1901.9550008 < limit.pitches[1]);
    assert!(limit.pitches[1] < 1901.9550009);
    assert!(582.512192 < limit.pitches[2]);
    assert!(limit.pitches[2] < 582.512193);
}

#[test]
fn ratio_limit_from_str() {
    let primes = vec!["2", "3", "7/5"];
    let limit = PrimeLimit::from_labels(&primes);
    assert!(limit.is_some());
    let limit = limit.unwrap();
    assert_eq!(&limit.label, "2.3.7/5");
    assert_eq!(limit.pitches[0], 1200.0);
    assert!(1901.9550008 < limit.pitches[1]);
    assert!(limit.pitches[1] < 1901.9550009);
    assert!(582.512192 < limit.pitches[2]);
    assert!(limit.pitches[2] < 582.512193);
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
fn comma_size() {
    let limit = PrimeLimit::new(5);
    let size = limit.interval_size(&[-4, 4, -1]);
    assert!(21.506285967 < size);
    assert!(size < 21.5062895968);
}

#[test]
fn size_13() {
    let limit = PrimeLimit::new(13);
    // 1001:1000
    let size = limit.interval_size(&[-3, 0, -3, 1, 1, 1]);
    assert!(1.7303690086 < size);
    assert!(size < 1.7303690087);
}

#[test]
fn name_12p() {
    let limit = PrimeLimit::new(13);
    let et = prime_mapping(&limit.pitches, 12);
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "12p");
    assert_eq!(Some(et.clone()), et_from_name(&limit, &name));
    // Check that the same result comes back without the "p"
    assert_eq!(Some(et), et_from_name(&limit, "12"));
}

#[test]
fn name_38de() {
    let limit = PrimeLimit::new(11);
    let et = vec![38, 60, 88, 106, 132];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "38de");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_b4() {
    let limit = PrimeLimit::explicit(vec![3, 5, 7]);
    let et = vec![4, 6, 7];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "b4p");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_q15e() {
    let limit = PrimeLimit::explicit(vec![10, 11, 12]);
    let et = vec![15, 15, 16];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "q15e");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_q32r() {
    let limit = PrimeLimit::explicit(vec![10, 11, 12]);
    let et = vec![32, 33, 34];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "q32r");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_q22() {
    let limit = PrimeLimit::explicit(vec![10, 11, 12]);
    let et = vec![22, 23, 24];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "q22p");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_q11() {
    let limit = PrimeLimit::explicit(vec![8, 10, 12, 14]);
    let et = vec![11, 12, 13, 14];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "q11p");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_q2t() {
    let limit = PrimeLimit::explicit(vec![8, 10, 12, 14]);
    let et = vec![2, 2, 2, 2]; // contorted
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "q2t");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_8dee() {
    let limit = PrimeLimit::new(11);
    let et = vec![8, 13, 19, 23, 29];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "8dee");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_2egg() {
    let limit = PrimeLimit::new(17);
    let et = vec![2, 3, 5, 6, 6, 7, 7];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "2egg");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_4ef() {
    let limit = PrimeLimit::new(17);
    let et = vec![4, 6, 9, 11, 13, 14, 16];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "4ef");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_4efgg() {
    let limit = PrimeLimit::new(17);
    let et = vec![4, 6, 9, 11, 13, 14, 15];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "4efgg");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_4efgggg() {
    let limit = PrimeLimit::new(17);
    let et = vec![4, 6, 9, 11, 13, 14, 14];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "4efgggg");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_4efg() {
    let limit = PrimeLimit::new(17);
    let et = vec![4, 6, 9, 11, 13, 14, 17];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "4efg");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn name_4efggg() {
    let limit = PrimeLimit::new(17);
    let et = vec![4, 6, 9, 11, 13, 14, 18];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "4efggg");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn rt12_from_name() {
    let limit = PrimeLimit::new(7);
    assert_eq!(
        Some(vec![vec![12, 19, 28, 34]]),
        mapping_from_name(&limit, "12p"),
    );
    assert_eq!(
        Some(vec![vec![12, 19, 28, 34]]),
        mapping_from_name(&limit, "12"),
    );
}

#[test]
fn meantone_from_name() {
    let limit = PrimeLimit::new(7);
    let expected = vec![vec![12, 19, 28, 34], vec![19, 30, 44, 53]];
    assert_eq!(
        Some(expected.clone()),
        mapping_from_name(&limit, "12 & 19"),
    );
    // The & is optional
    assert_eq!(
        Some(expected.clone()),
        mapping_from_name(&limit, "12 19"),
    );
    assert_eq!(
        Some(expected.clone()),
        mapping_from_name(&limit, "12 + 19"),
    );
    // + is also supported
    // Extra whitespace should be ignored
    assert_eq!(
        Some(expected.clone()),
        mapping_from_name(&limit, "   12  &  19  &&&& "),
    );
}

#[test]
fn bad_rt_from_name() {
    let limit = PrimeLimit::new(7);
    assert_eq!(None, mapping_from_name(&limit, "bad name"));
}

/// Test a case where a Chinese character is used for the wart
#[test]
fn name_chinese_wart() {
    let limit = PrimeLimit::new(101);
    let et = vec![
        62, 98, 144, 174, 214, 229, 253, 263, 280, 301, 307, 323, 332, 336,
        344, 355, 364, 367, 376, 381, 383, 390, 395, 401, 409, 412,
    ];
    let name = warted_et_name(&limit, &et);
    assert_eq!(name, "62rsvw一");
    assert_eq!(Some(et), et_from_name(&limit, &name));
}

#[test]
fn chinese_rollover() {
    assert_eq!(next_char('z'), '一');
}

#[test]
fn chinese_continuation() {
    assert_eq!(next_char('一'), '丁');
}

#[test]
fn nonoctave_prime_mapping() {
    let limit = PrimeLimit::explicit(vec![3, 5, 7, 11, 13]);
    let et = prime_mapping(&limit.pitches, 19);
    assert_eq!(et, vec![19, 28, 34, 41, 44]);
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

#[test]
fn normalize_already_positive() {
    let limit5 = PrimeLimit::new(5);
    let comma = vec![-4, 4, -1];
    let expected = comma.clone();
    assert_eq!(expected, super::normalize_positive(&limit5.pitches, comma));
}

#[test]
fn normalize_negative() {
    let limit5 = PrimeLimit::new(5);
    let comma = vec![4, -4, 1];
    let expected = vec![-4, 4, -1];
    assert_eq!(expected, super::normalize_positive(&limit5.pitches, comma));
}

fn near_enough_equal(x: f64, y: f64) -> bool {
    (x / y - 1.0).abs() < 1e-15
}
