extern crate nalgebra as na;
use na::{DMatrix, DVector};

use super::cangwu;
use super::{Cents, ETMap, Mapping, Tuning};
use cangwu::{rms_of_matrix, TenneyWeighted};

pub struct TETemperament {
    plimit: DVector<Cents>,
    melody: Mapping,
}

impl cangwu::TemperamentClass for TETemperament {
    fn mapping(&'_ self) -> &'_ Mapping {
        &self.melody
    }
}

impl cangwu::TenneyWeighted for TETemperament {
    fn mapping(&'_ self) -> &'_ Mapping {
        &self.melody
    }

    fn plimit(&'_ self) -> &'_ DVector<Cents> {
        &self.plimit
    }
}

impl TETemperament {
    /// Upgrade vectors into a struct of nalgebra objects
    pub fn new(plimit: &[Cents], melody: &[ETMap]) -> Self {
        let plimit = DVector::from_vec(plimit.to_vec());
        let melody = melody.to_vec();
        TETemperament { plimit, melody }
    }

    pub fn complexity(&self) -> f64 {
        rms_of_matrix(&self.weighted_mapping())
    }

    /// This shouldn't really be here, but it's easy
    pub fn optimal_tuning(&self) -> Tuning {
        let wmap = self.weighted_mapping();
        let pinv = wmap.pseudo_inverse(0.0).expect("no pseudoinverse");
        let tuning = pinv.column_sum() * 1200.0;
        tuning.iter().cloned().collect()
    }

    pub fn badness(&self) -> Cents {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let m = self.weighted_mapping();
        let offset_vec: Vec<f64> = m.row_mean().iter().cloned().collect();
        let mut translation = DMatrix::from_vec(rank, 1, offset_vec.clone());
        assert!(dimension > 0);
        for _ in 1..dimension {
            translation.extend(offset_vec.clone());
        }
        rms_of_matrix(&(m - translation.transpose())) * 1200.0
    }
}

#[cfg(test)]
fn make_marvel() -> TETemperament {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = super::PrimeLimit::new(11);
    TETemperament::new(&limit11.pitches, &marvel_vector)
}

#[cfg(test)]
fn make_jove() -> TETemperament {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    let limit11 = super::PrimeLimit::new(11);
    TETemperament::new(&limit11.pitches, &jove_vector)
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

#[rustfmt::skip]
#[test]
fn mystery() {
    let mystery_vector = vec![
        vec![29, 46, 67, 81, 100, 107],
        vec![58, 92, 135, 163, 201, 215],
    ];
    let limit13 = super::PrimeLimit::new(13);
    let mystery = TETemperament::new(&limit13.pitches, &mystery_vector);
    assert!(4.83894 < mystery.complexity());
    assert!(mystery.complexity() < 4.83895);
}
