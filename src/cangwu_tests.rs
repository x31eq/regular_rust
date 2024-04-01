use super::temperament_class::TemperamentClass;
use super::{cangwu, PrimeLimit};
use cangwu::CangwuTemperament;

fn make_marvel(limit11: &PrimeLimit) -> CangwuTemperament {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    CangwuTemperament::new(&limit11.pitches, &marvel_vector)
}

fn make_jove(limit11: &PrimeLimit) -> CangwuTemperament {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    CangwuTemperament::new(&limit11.pitches, &jove_vector)
}

#[test]
fn et_from_marvel() {
    let limit11 = PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let ets31 = marvel.ets_of_size(31);
    let expected = vec![vec![31, 49, 72, 87, 107]];
    assert_eq!(ets31, expected);
}

#[test]
fn et_from_jove() {
    let limit11 = PrimeLimit::new(11);
    let jove = make_jove(&limit11);
    let ets31 = jove.ets_of_size(31);
    let expected = vec![vec![31, 49, 72, 87, 107]];
    assert_eq!(ets31, expected);
}

#[test]
fn badness() {
    let limit11 = PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    assert!(0.16948 < marvel.badness(1.0));
    assert!(marvel.badness(1.0) < 0.16949);
    assert!(0.06882 < marvel.badness(0.1));
    assert!(marvel.badness(0.1) < 0.06883);

    let jove = make_jove(&limit11);
    assert!(0.18269 < jove.badness(1.0));
    assert!(jove.badness(1.0) < 0.18270);
    assert!(0.05606 < jove.badness(0.1));
    assert!(jove.badness(0.1) < 0.05607);
}

#[rustfmt::skip]
#[test]
fn mystery() {
    let mystery_vector = vec![
        vec![29, 46, 67, 81, 100, 107],
        vec![58, 92, 135, 163, 201, 215],
    ];
    let limit13 = PrimeLimit::new(13);
    let mystery =
        CangwuTemperament::new(&limit13.pitches, &mystery_vector);
    assert_eq!(mystery.key(), vec![0, 1, 1, 1, 1,
                             29, 46, 0, 14, 33, 40]);
    assert_eq!(mystery.rank(), 2);
    assert!(5.43717 < mystery.badness(1.0));
    assert!(mystery.badness(1.0) < 5.43718);
    assert!(2.52619 < mystery.badness(0.1));
    assert!(mystery.badness(0.1) < 2.52620);
}

#[rustfmt::skip]
#[test]
fn ragismic() {
    let ragismic_vector = vec![
        vec![171, 271, 397, 480],
        vec![270, 428, 627, 758],
        vec![494, 783, 1147, 1387],
    ];
    let limit7 = PrimeLimit::new(7);
    let ragismic =
        CangwuTemperament::new(&limit7.pitches, &ragismic_vector);
    assert_eq!(ragismic.key(), vec![
                                 1, -4,
                              1, 0, 7,
                           1, 0, 0, 1,
    ]);
    assert_eq!(ragismic.rank(), 3);
    assert!(0.17 < ragismic.badness(1.0));
    assert!(ragismic.badness(1.0) < 0.18);
    assert!(0.01 < ragismic.badness(0.1));
    assert!(ragismic.badness(0.1) < 0.02);
    let ragismic =
        super::te::TETemperament::new(&limit7.pitches, &ragismic_vector);
    assert!(0.17 < ragismic.complexity());
    assert!(ragismic.complexity() < 1.8);
}

#[test]
fn expected_limited_mappings() {
    let limit7 = PrimeLimit::new(7).pitches;
    let examples = cangwu::limited_mappings(19, 1.0, 1e2, &limit7);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![19, 30, 44, 53]);

    let limit13 = PrimeLimit::new(13).pitches;
    let examples = cangwu::limited_mappings(41, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![41, 65, 95, 115, 142, 152]);
    let examples = cangwu::limited_mappings(31, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 2);
    assert_eq!(examples[0], vec![31, 49, 72, 87, 107, 114]);
    assert_eq!(examples[1], vec![31, 49, 72, 87, 107, 115]);
}

#[test]
fn big_limit() {
    let sbyte = PrimeLimit::new(127).pitches;
    let mappings = cangwu::get_equal_temperaments(&sbyte, 0.3, 10);
    assert_eq!(
        octaves(&mappings),
        vec![62, 62, 31, 50, 50, 34, 31, 46, 60, 60]
    );
}

#[test]
fn nonoctave() {
    let limit = PrimeLimit::explicit(vec![3, 5, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(&limit.pitches, 10.0, 5);
    assert_eq!(octaves(&mappings), vec![7, 4, 6, 2, 9]);
}

#[test]
fn nofives() {
    let limit = PrimeLimit::explicit(vec![2, 3, 7, 11, 13]);
    let mappings = cangwu::get_equal_temperaments(&limit.pitches, 1.0, 5);
    assert_eq!(octaves(&mappings), vec![17, 41, 9, 46, 10]);
}

#[test]
fn marvel_from_key() {
    let limit11 = PrimeLimit::new(11);
    let original = make_marvel(&limit11);
    let clone = CangwuTemperament::from_ets_and_key(
        &limit11.pitches,
        &[22, 31, 41],
        &original.key(),
    );
    assert!(clone.is_some());
    if let Some(clone) = clone {
        assert_eq!(original.mapping(), clone.mapping());
    }
}

#[test]
fn jove_from_key() {
    let limit11 = PrimeLimit::new(11);
    let original = make_jove(&limit11);
    let clone = CangwuTemperament::from_ets_and_key(
        &limit11.pitches,
        &[27, 31, 41],
        &original.key(),
    );
    assert!(clone.is_some());
    if let Some(clone) = clone {
        assert_eq!(original.mapping(), clone.mapping());
    }
}

fn octaves(mappings: &Vec<super::ETMap>) -> super::ETMap {
    mappings.iter().map(|m| m[0]).collect()
}
