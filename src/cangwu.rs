//! Temperament finding with Cangwu badness

extern crate nalgebra as na;
use na::{DMatrix, DVector};

use super::{Cents, ETMap, FactorElement, Mapping, PriorityQueue, Tuning};
use std::collections::HashSet;

pub struct TemperamentClass {
    plimit: DVector<Cents>,
    melody: Mapping,
}

impl TemperamentClass {
    /// Upgrade vectors into a struct of nalgebra objects
    pub fn new(plimit: &[Cents], melody: &[ETMap]) -> Self {
        let plimit = DVector::from_vec(plimit.to_vec());
        let melody = melody.to_vec();
        TemperamentClass { plimit, melody }
    }

    /// Actual rank of the mapping matrix
    pub fn rank(&self) -> usize {
        let mut result = 0;
        for col in self.reduced_mapping().iter() {
            if col.iter().any(|&x| x != 0) {
                result += 1;
            }
        }
        result
    }

    /// Unique identifier for the mapping
    /// (hermite normal form flattened and
    /// with always-zero entries removed)
    pub fn key(&self) -> ETMap {
        let hermite = self.reduced_mapping();
        // This might be simplifiable now
        let cols = hermite.len();
        let rows = if cols == 0 { 0 } else {hermite[0].len()};
        let expected_size = rows * cols - cols * (cols - 1) / 2;
        let mut result: ETMap = Vec::with_capacity(expected_size);
        for (i, col) in hermite.iter().enumerate() {
            for &n in col.iter().skip(i) {
                result.push(n);
            }
        }
        assert!(result.len() == expected_size);
        result
    }

    pub fn reduced_mapping(&self) -> Mapping {
        super::hermite_normal_form(&self.melody)
    }

    pub fn badness(&self, ek: Cents) -> Cents {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let ek = ek / 1200.0;
        let epsilon = ek / (1.0 + square(ek)).sqrt();
        let scaling = 1.0 - epsilon;
        let m = self.weighted_mapping();
        let offset = scaling * m.row_mean();
        let offset_vec: Vec<f64> = offset.iter().cloned().collect();
        let mut translation = DMatrix::from_vec(rank, 1, offset_vec.clone());
        assert!(dimension > 0);
        for _ in 1..dimension {
            translation.extend(offset_vec.clone());
        }
        rms_of_matrix(&(m - translation.transpose())) * 1200.0
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

    fn weighted_mapping(&self) -> DMatrix<f64> {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let flattened = self.melody
            .iter()
            .map(|mapping| mapping.iter())
            .flatten()
            .cloned();
        let melody = DMatrix::from_iterator(dimension, rank, flattened);
        let weighting_vec: Vec<f64> =
            self.plimit.iter().map(|x| 1200.0 / x).collect();
        let mut weighting =
            DMatrix::from_vec(dimension, 1, weighting_vec.clone());
        assert!(rank > 0);
        for _ in 1..rank {
            weighting.extend(weighting_vec.clone());
        }
        melody.map(f64::from).component_mul(&weighting)
    }
}

pub fn higher_rank_search(
    plimit: &[Cents],
    ets: &[ETMap],
    rts: &[Vec<ETMap>],
    ek: Cents,
    n_results: usize,
) -> Vec<Vec<ETMap>> {
    let mut results = PriorityQueue::new(n_results);
    let mut cache = HashSet::new();
    for rt in rts {
        let rank = rt.len() + 1;
        for et in ets {
            let mut new_rt = rt.clone();
            new_rt.push(et.clone());
            let rt_obj = TemperamentClass::new(&plimit, &new_rt);
            if rt_obj.rank() == rank {
                let badness = rt_obj.badness(ek);
                if badness < results.cap {
                    let key = rt_obj.key();
                    if !cache.contains(&key) {
                        cache.insert(key);
                        results.push(badness, new_rt);
                    }
                }
            }
        }
    }
    results.extract()
}

fn rms_of_matrix(a: &DMatrix<f64>) -> f64 {
    let dimension = a.nrows() as f64;
    let gram = a.transpose().clone() * a;
    ((gram / dimension).determinant()).sqrt()
}

/// Get the best equal temperament mappings for the given prime limit
///
/// plimit: Sizes of prime harmonics in cents
///
/// ek: The Cangwu parameter in cents/octave
///
/// n_results: How many to return
pub fn get_equal_temperaments(
    plimit: &[Cents],
    ek: Cents,
    n_results: usize,
) -> Vec<ETMap> {
    // Stop weird things happening for non-standard units
    let plimit: Tuning =
        plimit.iter().map(|p| 12e2 * (p / plimit[0])).collect();

    let mut results = PriorityQueue::new(n_results);
    let bmax = preliminary_badness(&plimit, ek, n_results);
    let mut n_notes = 1;
    let mut cap = bmax;
    while (f64::from(n_notes)) < cap / ek {
        for mapping in limited_mappings(n_notes, ek, cap, &plimit) {
            let bad = equal_temperament_badness(&plimit, ek, &mapping);
            results.push(bad, mapping.clone());
        }
        n_notes += 1;
        cap = cap.min(results.cap);
    }

    debug_assert!(results.len() == n_results);
    results.extract()
}

pub fn equal_temperament_badness(
    plimit: &[Cents],
    ek: Cents,
    mapping: &[FactorElement],
) -> Cents {
    assert_eq!(plimit.len(), mapping.len());
    // Put the primes in terms of octaves
    let plimit: Vec<f64> = plimit.iter().map(|p| p / 12e2).collect();
    // Get a dimensionless ek
    let ek = ek / 12e2;
    let epsilon = ek / (1.0 + square(ek)).sqrt();
    let weighted_mapping: Vec<f64> = mapping
        .iter()
        .zip(plimit.into_iter())
        .map(|(&m, p)| f64::from(m) / p)
        .collect();
    let mean = |items: &Vec<f64>| {
        let mut sum = 0.0;
        for item in items.iter() {
            sum += item;
        }
        sum / (items.len() as f64)
    };
    let mean_w = mean(&weighted_mapping);
    let translation = (1.0 - epsilon) * mean_w;
    let bad2 = mean(
        &weighted_mapping
            .into_iter()
            .map(|x| x - translation)
            .map(square)
            .collect(),
    );
    bad2.sqrt() * 12e2
}

/// High guess for the worst badness of a search.
/// Must be a reasonable cap, and at least as high
/// as the worst result we want to keep in the real search.
fn preliminary_badness(
    plimit: &[Cents],
    ek: Cents,
    n_results: usize,
) -> Cents {
    // Find a large enough badness cap
    let mut results = PriorityQueue::new(n_results);
    for size in 1..=(plimit.len() + n_results) {
        let pmap = super::prime_mapping(&plimit, size as FactorElement);
        let badness = equal_temperament_badness(&plimit, ek, &pmap);
        results.push(badness, pmap);
    }
    results.cap
}

/// All mappings for a given division of the octave (or generalization)
/// within the given badness cutoff.
///
/// n_notes: Steps to the octave (or whatever comes first in plimit)
///
/// ek: Cangwu badness parameter in cents/octave
///
/// bmax: Cangwu badness cutoff in centified units
///
/// plimit: Sizes of prime harmonics in cents
///
/// # Panics
///
/// Probably won't panic but will attempt to generate
/// a huge vector of mappings if "bmax" is set too high.
pub fn limited_mappings(
    n_notes: FactorElement,
    ek: Cents,
    bmax: Cents,
    plimit: &[Cents],
) -> Vec<ETMap> {
    // Call things Cents but turn them to octaves/dimensionless
    let ek = ek / 12e2;
    let bmax = bmax / 12e2;
    let cap = square(bmax) * (plimit.len() as Cents) / square(plimit[0]);
    let epsilon2 = square(ek) / (1.0 + square(ek));
    let mut mapping = vec![n_notes; plimit.len()];
    let mut results = Vec::new();

    more_limited_mappings(
        &mut mapping,
        1,
        0.0,
        0.0,
        cap,
        epsilon2,
        &plimit,
        &mut results,
    );
    results
}

/// Helper function for limited_mappings that can't be a closure
/// because it's recursive and can't use a nested scope because
/// functions don't do that, so may as well be at top level.
///
/// mapping: the ET mapping with entries found so far
///
/// i: the element to choose next
///
/// tot: running total of w
///
/// tot2: running total of w squared
///
/// cap: the highest badness(squared) to keep
///
/// epsilon2: badness parameter
///
/// plimit: sizes of prime intervals in cents
///
/// results: vector to store found mappings in
fn more_limited_mappings(
    mut mapping: &mut ETMap,
    i: usize,
    tot: Cents,
    tot2: Cents,
    cap: Cents,
    epsilon2: Cents,
    plimit: &[Cents],
    mut results: &mut Vec<ETMap>,
) {
    assert!(mapping.len() == plimit.len());
    let weighted_size = f64::from(mapping[i - 1]) / plimit[i - 1];
    let tot = tot + weighted_size;
    let tot2 = tot2 + square(weighted_size);
    let lambda = 1.0 - epsilon2;
    debug_assert!(
        tot2 * (1.0 - 1e-10) <= lambda * square(tot) / (i as f64) + cap
    );
    if i == plimit.len() {
        // Recursion stops here.
        // Clone the object to save as the one being worked on
        // keeps changing
        results.push(mapping.clone());
    } else {
        let toti = tot * lambda / ((i as f64) + epsilon2);
        let error2 = tot2 - tot * toti;
        if error2 < cap {
            let target = plimit[i];
            let deficit = ((i + 1) as f64 * (cap - error2)
                / (i as f64 + epsilon2))
                .sqrt();
            let xmin = target * (toti - deficit);
            let xmax = target * (toti + deficit);
            for guess in intrange(xmin, xmax) {
                mapping[i] = guess;
                more_limited_mappings(
                    &mut mapping,
                    i + 1,
                    tot,
                    tot2,
                    cap,
                    epsilon2,
                    &plimit,
                    &mut results,
                );
            }
        }
    }
}

fn square(x: f64) -> f64 {
    x.powi(2)
}

/// Range of integers between x and y
fn intrange(x: f64, y: f64) -> std::ops::RangeInclusive<FactorElement> {
    ((x.ceil() as FactorElement)..=(y.floor() as FactorElement))
}
