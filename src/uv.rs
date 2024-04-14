extern crate nalgebra as na;
use na::DMatrix;
use std::ops::Add;

use super::cangwu::filtered_equal_temperaments;
use super::{Cents, ETMap, ETSlice, Exponent, Mapping};

/// Return the commatic unison vector for a mapping with
/// only one dimension short
pub fn only_unison_vector(mapping: &Mapping) -> Option<ETMap> {
    let rank = mapping.len();
    if rank == 0 {
        return None;
    }
    let dimension = mapping[0].len();
    if rank + 1 != dimension {
        return None;
    }
    let fiter = mapping.iter().flat_map(|m| m.iter()).map(|&x| x as f64);
    let fmap = DMatrix::from_iterator(dimension, rank, fiter);
    let mut sq = fmap.insert_column(0, 0.0);
    for i in 0..dimension {
        sq[(i, 0)] = 1.0;
        if let Some(inverse) = sq.clone().try_inverse() {
            let adjoint = inverse * sq.clone().determinant();
            return Some(
                adjoint
                    .row(0)
                    .iter()
                    .map(|&x| x.round() as Exponent)
                    .collect(),
            );
        }
        sq[(i, 0)] = 0.0;
    }
    None
}

pub fn get_ets_tempering_out(
    plimit: &[Cents],
    ek: Cents,
    unison_vectors: &[ETMap],
    n_results: usize,
) -> Mapping {
    filtered_equal_temperaments(
        plimit,
        |et| tempers_out(unison_vectors, et),
        ek,
        n_results,
    )
}

fn tempers_out(mapping: &[ETMap], interval: &ETSlice) -> bool {
    mapping.iter().all(|et| dotprod(et, interval) == 0)
}

/// Choose a value for the cangwu badness parameter
/// for a search based on these unison vectors.
/// This is a rough guess that has to be precise for
/// backwards compatibility reasons
pub fn ek_for_search(limit: &[Cents], uvs: &[ETMap]) -> Cents {
    uvs.iter()
        .map(|uv| inherent_error(limit, uv))
        .reduce(Cents::max)
        .expect("no max")
}

fn inherent_error(limit: &[Cents], uv: &ETSlice) -> Cents {
    if uv.is_empty() {
        // senseless question, return something to avoid panics
        return 0.0;
    }
    let q = limit
        .iter()
        .zip(uv.iter())
        .map(|(&x, &y)| x / 12e2 * y as Cents);
    let len = limit.len() as Cents;
    let mean = q.clone().fold(0.0, Cents::add) / len;
    let rms = (q.fold(0.0, |tot, x| tot + x * x) / len).sqrt();
    (mean / rms).abs()
}

fn dotprod(a: &[Exponent], b: &[Exponent]) -> i64 {
    a.iter()
        .zip(b.iter())
        // multiply as i64 to avoid overflows
        .fold(0, |tot, (&m, &n)| tot + (m as i64) * (n as i64))
}

#[test]
fn meantone5() {
    let mapping = vec![vec![12, 19, 28], vec![7, 11, 16]];
    let expected = vec![-4, 4, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[4, -1, -1]));
    assert!(!tempers_out(&mapping, &[-3, -1, 1]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv =
        super::normalize_positive(&super::PrimeLimit::new(5).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7() {
    let mapping = vec![
        vec![41, 65, 95, 115],
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
    ];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv =
        super::normalize_positive(&super::PrimeLimit::new(7).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7_reordered() {
    let mapping = vec![
        vec![41, 65, 95, 115],
        vec![19, 30, 44, 53],
        vec![31, 49, 72, 87],
    ];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv =
        super::normalize_positive(&super::PrimeLimit::new(7).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn marvel7_from_reduced() {
    let mapping = vec![vec![1, 0, 0, -5], vec![0, 1, 0, 2], vec![0, 0, 1, 2]];
    let expected = vec![-5, 2, 2, -1];
    assert!(tempers_out(&mapping, &expected));
    assert!(!tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(Some(expected), only_unison_vector(&mapping));
}

#[test]
fn meantone7() {
    let mapping = vec![vec![31, 49, 72, 87], vec![19, 30, 44, 53]];
    assert!(tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(None, only_unison_vector(&mapping));
}

#[test]
fn meantone7_redundant() {
    let mapping = vec![
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
        vec![12, 19, 28, 34],
    ];
    assert!(tempers_out(&mapping, &[-4, 4, -1, 0]));
    assert_eq!(None, only_unison_vector(&mapping));
}

#[test]
fn uv_1575_1573() {
    // This failed with the initial implementation
    let mapping = vec![
        vec![72, 114, 167, 202, 249, 266],
        vec![58, 92, 135, 163, 201, 215],
        vec![87, 138, 202, 244, 301, 322],
        vec![31, 49, 72, 87, 107, 115],
        vec![121, 192, 281, 340, 419, 448],
    ];
    let expected = vec![0, 2, 2, 1, -2, -1];
    assert!(tempers_out(&mapping, &expected));
    let uv = only_unison_vector(&mapping).expect("no UV");
    let uv =
        super::normalize_positive(&super::PrimeLimit::new(13).pitches, uv);
    assert_eq!(expected, uv);
}

#[test]
fn meantone_ets() {
    let limit = super::PrimeLimit::new(5);
    let comma = vec![-4, 4, -1];
    let ets =
        get_ets_tempering_out(&limit.pitches, 3.0, &[comma.clone()], 10);
    assert!(tempers_out(&ets, &comma));
}

#[test]
fn syntonic11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma = vec![-4, 4, -1, 0, 0];
    let ets =
        get_ets_tempering_out(&limit.pitches, 3.0, &[comma.clone()], 10);
    assert!(tempers_out(&ets, &comma));
}

#[test]
fn marvel11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma1 = vec![-5, 2, 2, -1, 0];
    let comma2 = vec![2, 3, 1, -2, -1];
    let ets = get_ets_tempering_out(
        &limit.pitches,
        3.0,
        &[comma1.clone(), comma2.clone()],
        10,
    );
    assert!(tempers_out(&ets, &comma1));
    assert!(tempers_out(&ets, &comma2));
}

#[test]
fn porcupine11_ets() {
    let limit = super::PrimeLimit::new(11);
    let comma1 = vec![-1, -3, 1, 0, 1];
    let comma2 = vec![6, -2, 0, -1, 0];
    let comma3 = vec![2, -2, 2, 0, -1];
    let ets = get_ets_tempering_out(
        &limit.pitches,
        3.0,
        &[comma1.clone(), comma2.clone(), comma3.clone()],
        5,
    );
    assert!(tempers_out(&ets, &comma1));
    assert!(tempers_out(&ets, &comma2));
    assert!(tempers_out(&ets, &comma3));
}

#[test]
fn inherent_errors() {
    let limit = super::PrimeLimit::new(11).pitches;
    let comma = vec![2, -2, 2, 0, -1];
    let ek = inherent_error(&limit, &comma);
    assert!(0.0009400 < ek);
    assert!(ek < 0.0009401);

    let comma = vec![-5, 2, 2, -1, 0];
    let ek = inherent_error(&limit, &comma);
    assert!(0.000357 < ek);
    assert!(ek < 0.000358);

    // The limit does matter
    let limit = super::PrimeLimit::new(7).pitches;
    let comma = vec![-5, 2, 2, -1];
    let ek = inherent_error(&limit, &comma);
    assert!(0.000400 < ek);
    assert!(ek < 0.000401);
}
