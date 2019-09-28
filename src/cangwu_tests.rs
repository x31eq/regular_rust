use super::cangwu;
use super::PrimeLimit;

extern crate nalgebra as na;
use na::DMatrix;

fn make_marvel() -> cangwu::TemperamentClass {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = PrimeLimit::new(11);
    cangwu::TemperamentClass::new(&limit11.pitches, &marvel_vector)
}

fn make_jove() -> cangwu::TemperamentClass {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = PrimeLimit::new(11);
    cangwu::TemperamentClass::new(&limit11.pitches, &jove_vector)
}

#[test]
fn badness() {
    let marvel = make_marvel();
    assert!(0.16948 < marvel.badness(1.0));
    assert!(marvel.badness(1.0) < 0.16949);
    assert!(0.06882 < marvel.badness(0.1));
    assert!(marvel.badness(0.1) < 0.06883);

    let jove = make_jove();
    assert!(0.18269 < jove.badness(1.0));
    assert!(jove.badness(1.0) < 0.18270);
    assert!(0.05606 < jove.badness(0.1));
    assert!(jove.badness(0.1) < 0.05607);
}

#[test]
fn complexity() {
    let marvel = make_marvel();
    assert!(0.155663 < marvel.complexity());
    assert!(marvel.complexity() < 0.155664);

    let jove = make_jove();
    // Less precision here because it disagrees with Python.
    assert!(0.17475 < jove.complexity());
    assert!(jove.complexity() < 0.174755);
}

#[rustfmt::skip]
#[test]
fn hermite() {
    let marvel = make_marvel();
    let marvel_hermite =
        vec![1, 0, 0, -5, 12,
             0, 1, 0, 2, -1,
             0, 0, 1, 2, -3];
    let marvel_hermite = DMatrix::from_vec(5, 3, marvel_hermite);
    assert_eq!(marvel.reduced_mapping(), marvel_hermite);

    let jove = make_jove();
    let jove_hermite = vec![1, 1, 1, 2, 2,
                            0, 2, 1, 1, 5,
                            0, 0, 2, 1, 0];
    let jove_hermite = DMatrix::from_vec(5, 3, jove_hermite);
    assert_eq!(jove.reduced_mapping(), jove_hermite);
}

#[rustfmt::skip]
#[test]
fn key() {
    assert_eq!(
        make_marvel().key(),
        vec![1, 0, 0, -5, 12,
                1, 0, 2, -1,
                   1, 2, -3]
    );

    assert_eq!(make_jove().key(), vec![1, 1, 1, 2, 2,
                                          2, 1, 1, 5,
                                             2, 1, 0]);
}

#[test]
fn rank() {
    assert_eq!(make_marvel().rank(), 3);
    assert_eq!(make_jove().rank(), 3);
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
        cangwu::TemperamentClass::new(&limit13.pitches, &mystery_vector);
    assert_eq!(mystery.key(), vec![29, 46, 0, 14, 33, 40,
                                        0, 1, 1, 1, 1]);
    assert_eq!(mystery.rank(), 2);
    assert!(4.83894 < mystery.complexity());
    assert!(mystery.complexity() < 4.83895);
    assert!(5.43717 < mystery.badness(1.0));
    assert!(mystery.badness(1.0) < 5.43718);
    assert!(2.52619 < mystery.badness(0.1));
    assert!(mystery.badness(0.1) < 2.52620);
}

#[test]
fn tuning() {
    let marvel = make_marvel();
    let expected_tuning = vec![3.96487, 17.32226, 14.05909];
    for (expected, calculated) in expected_tuning
        .iter()
        .zip(marvel.optimal_tuning().into_iter())
    {
        let discrepancy = (expected - calculated).abs();
        assert!(discrepancy < 0.00001);
    }
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

fn octaves(mappings: &Vec<super::ETMap>) -> super::ETMap {
    mappings.iter().map(|m| m[0]).collect()
}
