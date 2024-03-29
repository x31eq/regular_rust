extern crate nalgebra as na;
use na::{DMatrix};

use super::{ETMap, Exponent, Mapping};

/// Return the commatic unison vector for a mapping with only one dimension short
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
    let det = sq.clone().determinant();
    let adjoint = sq.clone().try_inverse()? * det;
    Some(adjoint.row(0).iter().map(|&x| x as Exponent).collect())
}

#[test]
fn meantone5() {
    let mapping = vec![vec![12, 19, 28], vec![7, 11, 16]];
    let expected = vec![-4, 4, -1];
    assert_eq!(Some(expected), only_unison_vector(mapping));
}
