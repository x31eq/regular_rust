extern crate nalgebra as na;
use na::DMatrix;

use super::{ETMap, Exponent, Mapping};

/// Return the commatic unison vector for a mapping with
/// only one dimension short
pub fn only_unison_vector(mapping: Mapping) -> Option<ETMap> {
    let rank = mapping.len();
    if rank == 0 {
        return None;
    }
    let dimension = mapping[0].len();
    let fiter = mapping.iter().flat_map(|m| m.iter()).map(|&x| x as f64);
    let fmap = DMatrix::from_iterator(dimension, rank, fiter);
    let mut sq = fmap.insert_column(0, 0.0);
    sq[(0, 0)] = 1.0;
    if !sq.is_square() {
        return None;
    }
    let det = sq.clone().determinant();
    let adjoint = sq.try_inverse()? * det;
    Some(
        adjoint
            .row(0)
            .iter()
            .map(|&x| x.round() as Exponent)
            .collect(),
    )
}

#[test]
fn meantone5() {
    let mapping = vec![vec![12, 19, 28], vec![7, 11, 16]];
    let expected = vec![-4, 4, -1];
    assert_eq!(Some(expected), only_unison_vector(mapping));
}

#[test]
fn marvel7() {
    let mapping = vec![
        vec![41, 65, 95, 115],
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
    ];
    let expected = vec![-5, 2, 2, -1];
    assert_eq!(Some(expected), only_unison_vector(mapping));
}

#[test]
fn marvel7_from_reduced() {
    let mapping = vec![vec![1, 0, 0, -5], vec![0, 1, 0, 2], vec![0, 0, 1, 2]];
    let expected = vec![-5, 2, 2, -1];
    assert_eq!(Some(expected), only_unison_vector(mapping));
}

#[test]
fn meantone7() {
    let mapping = vec![vec![31, 49, 72, 87], vec![19, 30, 44, 53]];
    assert_eq!(None, only_unison_vector(mapping));
}

#[test]
fn meantone7_redundant() {
    let mapping = vec![
        vec![31, 49, 72, 87],
        vec![19, 30, 44, 53],
        vec![12, 19, 28, 34],
    ];
    assert_eq!(None, only_unison_vector(mapping));
}
