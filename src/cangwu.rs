//! Temperament finding with Cangwu badness

extern crate nalgebra as na;
use na::DMatrix;

use super::temperament_class::{key_to_mapping, TemperamentClass};
use super::{
    map, prime_mapping, Cents, ETMap, ETSlice, Exponent, Mapping,
    PriorityQueue,
};
use std::collections::HashSet;

pub struct CangwuTemperament<'a> {
    plimit: &'a [Cents],
    melody: Mapping,
}

pub trait TenneyWeighted {
    fn mapping(&self) -> &Mapping;
    fn plimit(&self) -> &[Cents];

    fn weighted_mapping(&self) -> DMatrix<f64> {
        let melody = self.mapping();
        let plimit = self.plimit();
        weight_mapping(melody, plimit)
    }
}

fn weight_mapping(mapping: &[ETMap], plimit: &[Cents]) -> DMatrix<f64> {
    let rank = mapping.len();
    let dimension = plimit.len();
    let flattened = mapping.iter().flat_map(|m| m.iter()).cloned();
    let mapping = DMatrix::from_iterator(dimension, rank, flattened);
    let weighting_vec = map(|x| 1200.0 / x, plimit);
    let mut weighting =
        DMatrix::from_vec(dimension, 1, weighting_vec.clone());
    assert!(rank > 0);
    for _ in 1..rank {
        weighting.extend(weighting_vec.clone());
    }
    mapping.map(f64::from).component_mul(&weighting)
}

impl<'a> CangwuTemperament<'a> {
    pub fn new(plimit: &'a [Cents], melody: &[ETMap]) -> Self {
        let melody = melody.to_vec();
        CangwuTemperament { plimit, melody }
    }

    pub fn from_ets_and_key(
        plimit: &'a [Cents],
        ets: &ETSlice,
        key: &ETSlice,
    ) -> Option<Self> {
        let melody = vec![];
        let mut result = CangwuTemperament { plimit, melody };
        let tclass = Self::new(plimit, &key_to_mapping(plimit.len(), key)?);
        for &et in ets.iter() {
            for etmap in tclass.ets_of_size(et) {
                // This might not give the expected results where the
                // same ET size will work twice,
                // but it is likely to produce something sensible
                if tclass.et_belongs(&etmap) && !result.et_belongs(&etmap) {
                    result.melody.push(etmap.clone())
                }
            }
        }
        if result.rank() == tclass.rank() {
            Some(result)
        } else {
            None
        }
    }

    pub fn badness(&self, ek: Cents) -> Cents {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let ek = ek / 1200.0;
        let epsilon = ek / (1.0 + square(ek)).sqrt();
        let scaling = 1.0 - epsilon;
        let m = self.weighted_mapping();
        let offset = scaling * m.row_mean();
        let offset_vec: Vec<_> = offset.iter().cloned().collect();
        let mut translation = DMatrix::from_vec(rank, 1, offset_vec.clone());
        assert!(dimension > 0);
        for _ in 1..dimension {
            translation.extend(offset_vec.clone());
        }
        rms_of_matrix(&(m - translation.transpose())) * 1200.0
    }

    /// Get equal temperaments of a specific size belonging to the class.
    /// If more than one match is legal, they might not all be found.
    pub fn ets_of_size(&self, size: Exponent) -> Mapping {
        let pet = prime_mapping(self.plimit, size);
        let ek = self.badness(0.0);
        let mut bmax = Self::new(self.plimit, &[pet]).badness(ek);
        for _ in 0..100 {
            let ets = limited_mappings(size, ek, bmax, self.plimit);
            if !ets.is_empty() {
                return ets;
            }
            bmax *= 0.1;
        }
        // Return an empty result if we couldn't find anything
        // in a reasonable amount of time
        vec![]
    }
}

impl TemperamentClass for CangwuTemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }
}

impl TenneyWeighted for CangwuTemperament<'_> {
    fn mapping(&self) -> &Mapping {
        &self.melody
    }

    fn plimit(&self) -> &[Cents] {
        self.plimit
    }
}

pub fn higher_rank_search(
    plimit: &[Cents],
    ets: &[ETMap],
    rts: &[Mapping],
    ek: Cents,
    n_results: usize,
) -> Vec<Mapping> {
    let mut results = PriorityQueue::new(n_results);
    let mut cache = HashSet::new();
    for rt in rts {
        let rank = rt.len() + 1;
        for et in ets {
            let mut new_rt = rt.clone();
            new_rt.push(et.clone());
            let rt_obj = CangwuTemperament::new(plimit, &new_rt);
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
    results.extract().collect()
}

pub fn rms_of_matrix(a: &DMatrix<f64>) -> f64 {
    let dimension = a.nrows() as f64;
    let gram = (a.transpose() * a) / dimension;
    // The LU decomposition stops the determinant overflowing
    // for rank 3 temperament classes defined by big ETs.
    // (For rank above 3 the LU decomposition is used by default.)
    gram.lu().determinant().sqrt()
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
) -> Mapping {
    // Stop weird things happening for non-standard units
    let plimit = map(|p| 12e2 * (p / plimit[0]), plimit);

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
    results.extract().collect()
}

pub fn equal_temperament_badness(
    plimit: &[Cents],
    ek: Cents,
    mapping: &[Exponent],
) -> Cents {
    assert_eq!(plimit.len(), mapping.len());
    // Put the primes in terms of octaves
    let plimit = map(|p| p / 12e2, plimit);
    // Get a dimensionless ek
    let ek = ek / 12e2;
    let epsilon = ek / (1.0 + square(ek)).sqrt();
    let weighted_mapping: Vec<_> = mapping
        .iter()
        .zip(plimit)
        .map(|(&m, p)| f64::from(m) / p)
        .collect();
    let mean = |items: &Vec<_>| {
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
        let pmap = prime_mapping(plimit, size as Exponent);
        let badness = equal_temperament_badness(plimit, ek, &pmap);
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
    n_notes: Exponent,
    ek: Cents,
    bmax: Cents,
    plimit: &[Cents],
) -> Mapping {
    let cap = square(bmax / 12e2) * (plimit.len() as f64) / square(plimit[0]);
    let mut searcher = MoreMappings::new(n_notes, cap, ek / 12e2, plimit);
    searcher.search(1, 0.0, 0.0);
    searcher.results
}

/// Simple struct to hold global data for the mapping search
struct MoreMappings<'a> {
    cap: f64,          // the highest badness (squared) to keep
    epsilon2: f64,     // badness parameter
    plimit: &'a [f64], // sizes of prime intervals
    lambda: f64,       // alternative badness parameter
    mapping: ETMap,    // working result
    results: Mapping,  // final results
}

impl<'a> MoreMappings<'a> {
    fn new(n_notes: Exponent, cap: f64, ek: f64, plimit: &'a [f64]) -> Self {
        let epsilon2 = square(ek) / (1.0 + square(ek));
        let lambda = 1.0 - epsilon2;
        let mapping = vec![n_notes; plimit.len()];
        let results = Vec::new();
        MoreMappings {
            cap,
            epsilon2,
            plimit,
            lambda,
            mapping,
            results,
        }
    }

    /// i: the element to choose next
    ///
    /// tot: running total of w
    ///
    /// tot2: running total of w squared
    fn search(&mut self, i: usize, tot: f64, tot2: f64) {
        assert!(self.mapping.len() == self.plimit.len());
        let weighted_size =
            f64::from(self.mapping[i - 1]) / self.plimit[i - 1];
        let tot = tot + weighted_size;
        let tot2 = tot2 + square(weighted_size);
        debug_assert!(
            tot2 * (1.0 - 1e-10)
                <= self.lambda * square(tot) / (i as f64) + self.cap
        );
        if i == self.plimit.len() {
            // Recursion stops here.
            // Clone the object to save as the one being worked on
            // keeps changing
            self.results.push(self.mapping.clone());
        } else {
            let toti = tot * self.lambda / ((i as f64) + self.epsilon2);
            let error2 = tot2 - tot * toti;
            if error2 < self.cap {
                let target = self.plimit[i];
                let deficit = ((i + 1) as f64 * (self.cap - error2)
                    / (i as f64 + self.epsilon2))
                    .sqrt();
                let xmin = target * (toti - deficit);
                let xmax = target * (toti + deficit);
                for guess in intrange(xmin, xmax) {
                    self.mapping[i] = guess;
                    self.search(i + 1, tot, tot2);
                }
            }
        }
    }
}

fn square(x: f64) -> f64 {
    x.powi(2)
}

/// Range of integers between x and y
fn intrange(x: f64, y: f64) -> std::ops::RangeInclusive<Exponent> {
    (x.ceil() as Exponent)..=(y.floor() as Exponent)
}
