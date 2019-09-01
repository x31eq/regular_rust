use super::PrimeLimit;
use super::cangwu;

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
    let octaves = mappings.iter()
                        .map(|m| m[0])
                        .collect::<Vec<_>>();
    assert_eq!(octaves,
               vec![62, 62, 31, 50, 50, 34, 31, 46, 60, 60]);
}

#[test]
fn nonoctave() {
    let limit = PrimeLimit::explicit(vec![3, 5, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches, 10.0, 5);
    let octaves = mappings.iter()
                        .map(|m| m[0])
                        .collect::<Vec<_>>();
    assert_eq!(octaves, vec![7, 4, 6, 2, 9]);
}

#[test]
fn nofives() {
    let limit = PrimeLimit::explicit(vec![2, 3, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(
        &limit.pitches, 1.0, 5);
    let octaves = mappings.iter()
                        .map(|m| m[0])
                        .collect::<Vec<_>>();
    assert_eq!(octaves, vec![17, 41, 9, 46, 10]);
}
