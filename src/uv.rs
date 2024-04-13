extern crate nalgebra as na;
use na::DMatrix;

use super::{ETMap, ETSlice, Exponent, Mapping};

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

pub fn tempers_out(mapping: &[ETMap], interval: &ETSlice) -> bool {
    for etmap in mapping {
        if dotprod(etmap, interval) != 0 {
            return false;
        }
    }
    true
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
    assert_eq!(Some(expected), only_unison_vector(&mapping));
}

#[test]
fn meantone7() {
    let mapping = vec![vec![31, 49, 72, 87], vec![19, 30, 44, 53]];
    assert_eq!(None, only_unison_vector(&mapping));
}

#[test]
fn meantone7_redundant() {
    let mapping = vec![
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
        vec![12, 19, 28, 34],
    ];
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
