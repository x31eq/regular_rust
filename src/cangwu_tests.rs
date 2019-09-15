use super::PrimeLimit;
use super::cangwu;

#[test]
fn complexity() {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = PrimeLimit::new(11);
    let marvel = cangwu::TemperamentClass::new(
        &limit11.pitches, &marvel_vector);
    let complexity = marvel.complexity();
    assert!(0.155663 < complexity);
    assert!(complexity < 0.155664);
    let expected_tuning = vec![3.96487, 17.32226, 14.05909];
    for (expected, calculated) in
            expected_tuning.iter().zip(
                marvel.optimal_tuning().into_iter()) {
        let discrepancy = (expected - calculated).abs();
        assert!(discrepancy < 0.00001);
    }
    println!("{:?}", marvel.optimal_tuning());
}

#[test]
fn expected_limited_mappings() {
    let limit7 = PrimeLimit::new(7).pitches;
    let examples = cangwu::limited_mappings(
            19, 1.0, 1e2, &limit7);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![19, 30, 44, 53]);

    let limit13 = PrimeLimit::new(13).pitches;
    let examples = cangwu::limited_mappings(
            41, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![41, 65, 95, 115, 142, 152]);
    let examples = cangwu::limited_mappings(
            31, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 2);
    assert_eq!(examples[0], vec![31, 49, 72, 87, 107, 114]);
    assert_eq!(examples[1], vec![31, 49, 72, 87, 107, 115]);
}

#[test]
fn big_limit() {
    let sbyte = PrimeLimit::new(127).pitches;
    let mappings = cangwu::get_equal_temperaments(
            &sbyte, 0.3, 10);
    assert_eq!(octaves(&mappings),
               vec![62, 62, 31, 50, 50, 34, 31, 46, 60, 60]);
}

#[test]
fn nonoctave() {
    let limit = PrimeLimit::explicit(vec![3, 5, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches, 10.0, 5);
    assert_eq!(octaves(&mappings), vec![7, 4, 6, 2, 9]);
}

#[test]
fn nofives() {
    let limit = PrimeLimit::explicit(vec![2, 3, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches, 1.0, 5);
    assert_eq!(octaves(&mappings), vec![17, 41, 9, 46, 10]);
}

fn octaves(mappings: &Vec<super::ETMap>) -> super::ETMap {
    mappings.iter().map(|m| m[0]).collect()
}
