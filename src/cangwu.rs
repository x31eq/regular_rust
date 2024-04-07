//! Temperament finding with Cangwu badness

extern crate nalgebra as na;
use na::DMatrix;

use super::temperament_class::{key_to_mapping, TemperamentClass};
use super::uv::only_unison_vector;
use super::{
    et_from_name, map, normalize_positive, prime_mapping, Cents, ETMap,
    ETSlice, Exponent, Mapping, PrimeLimit, PriorityQueue,
};
use std::collections::HashSet;

pub struct CangwuTemperament<'a> {
    plimit: &'a [Cents],
    pub melody: Mapping,
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

    pub fn from_et_names(
        plimit: &'a PrimeLimit,
        ets: &[String],
    ) -> Option<Self> {
        if let Some(melody) =
            ets.iter().map(|name| et_from_name(plimit, name)).collect()
        {
            let plimit = &plimit.pitches;
            Some(CangwuTemperament { plimit, melody })
        } else {
            None
        }
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
            bmax *= 1.1;
        }
        // Return an empty result if we couldn't find anything
        // in a reasonable amount of time
        vec![]
    }

    /// Find unison vectors that this temperament class tempers out.
    /// Might not find as many as you ask for, but will do its best
    pub fn unison_vectors(&self, n_results: usize) -> Mapping {
        let rank = self.melody.len();
        let dimension = self.plimit.len();
        let n_ets = n_results * 2;
        let n_lts = n_results + n_results / 2;
        let ek = self.badness(0.0) * 10.0;
        let seed_ets: Vec<ETMap> =
            get_equal_temperaments(self.plimit, ek, n_ets)
                .drain(..)
                .filter(|et| !self.et_belongs(et))
                .collect();
        let mut rts = vec![self.melody.clone()];
        for new_rank in (rank + 1)..dimension {
            rts = higher_rank_search(
                self.plimit,
                &seed_ets,
                &rts,
                ek,
                if new_rank == dimension - 1 {
                    n_results
                } else {
                    n_lts
                },
            );
        }
        rts.iter()
            .filter_map(|rt| only_unison_vector(rt))
            .map(|uv| normalize_positive(self.plimit, uv))
            .collect()
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

/// Decide if this is unambiguously the best mapping of
/// this number of notes in the prime limit.
/// Really a TE error function, but here because we have the search.
pub fn ambiguous_et(plimit: &[Cents], et: &ETMap) -> bool {
    let n_notes = if let Some(&n) = et.first() {
        n
    } else {
        // Say an equal temperament with no mappings is unambiguous
        return false;
    };
    if et != &prime_mapping(plimit, et[0]) {
        return true;
    }
    // As this is the prime mapping, check that there are
    // no other mappings within 20% of its error
    let error = equal_temperament_badness(plimit, 0.0, et);
    let others = limited_mappings(n_notes, 0.0, error * 1.2, plimit);
    others.len() > 1
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

#[cfg(test)]
fn make_marvel(limit11: &super::PrimeLimit) -> CangwuTemperament {
    let marvel_vector = vec![
        vec![22, 35, 51, 62, 76],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    CangwuTemperament::new(&limit11.pitches, &marvel_vector)
}

#[cfg(test)]
fn make_jove(limit11: &super::PrimeLimit) -> CangwuTemperament {
    let jove_vector = vec![
        vec![27, 43, 63, 76, 94],
        vec![31, 49, 72, 87, 107],
        vec![41, 65, 95, 115, 142],
    ];
    CangwuTemperament::new(&limit11.pitches, &jove_vector)
}

#[cfg(test)]
fn make_porcupine(limit11: &super::PrimeLimit) -> CangwuTemperament {
    let porcupine_vector =
        vec![vec![22, 35, 51, 62, 76], vec![15, 24, 35, 42, 52]];
    CangwuTemperament::new(&limit11.pitches, &porcupine_vector)
}

#[test]
fn et_from_marvel() {
    let limit11 = super::PrimeLimit::new(11);
    let marvel = make_marvel(&limit11);
    let ets31 = marvel.ets_of_size(31);
    let expected = vec![vec![31, 49, 72, 87, 107]];
    assert_eq!(ets31, expected);
}

#[test]
fn et_from_jove() {
    let limit11 = super::PrimeLimit::new(11);
    let jove = make_jove(&limit11);
    let ets31 = jove.ets_of_size(31);
    let expected = vec![vec![31, 49, 72, 87, 107]];
    assert_eq!(ets31, expected);
}

#[test]
fn et22_from_porcupine() {
    let limit11 = super::PrimeLimit::new(11);
    let porcupine = make_porcupine(&limit11);
    let ets22 = porcupine.ets_of_size(22);
    let expected = vec![vec![22, 35, 51, 62, 76]];
    assert_eq!(ets22, expected);
}

#[test]
fn et15_from_porcupine() {
    let limit11 = super::PrimeLimit::new(11);
    let porcupine = make_porcupine(&limit11);
    let ets15 = porcupine.ets_of_size(15);
    let expected = vec![vec![15, 24, 35, 42, 52]];
    assert_eq!(ets15, expected);
}

#[test]
fn badness() {
    let limit11 = super::PrimeLimit::new(11);
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
    let limit13 = super::PrimeLimit::new(13);
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
    let limit7 = super::PrimeLimit::new(7);
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
    let limit7 = super::PrimeLimit::new(7).pitches;
    let examples = limited_mappings(19, 1.0, 1e2, &limit7);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![19, 30, 44, 53]);

    let limit13 = super::PrimeLimit::new(13).pitches;
    let examples = limited_mappings(41, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0], vec![41, 65, 95, 115, 142, 152]);
    let examples = limited_mappings(31, 1.0, 1e2, &limit13);
    assert_eq!(examples.len(), 2);
    assert_eq!(examples[0], vec![31, 49, 72, 87, 107, 114]);
    assert_eq!(examples[1], vec![31, 49, 72, 87, 107, 115]);
}

#[test]
fn big_limit() {
    let sbyte = super::PrimeLimit::new(127).pitches;
    let mappings = get_equal_temperaments(&sbyte, 0.3, 10);
    assert_eq!(
        octaves(&mappings),
        vec![62, 62, 31, 50, 50, 34, 31, 46, 60, 60]
    );
}

#[test]
fn nonoctave() {
    let limit = super::PrimeLimit::explicit(vec![3, 5, 7, 11, 13]);
    let mappings = get_equal_temperaments(&limit.pitches, 10.0, 5);
    assert_eq!(octaves(&mappings), vec![7, 4, 6, 2, 9]);
}

#[test]
fn nofives() {
    let limit = super::PrimeLimit::explicit(vec![2, 3, 7, 11, 13]);
    let mappings = get_equal_temperaments(&limit.pitches, 1.0, 5);
    assert_eq!(octaves(&mappings), vec![17, 41, 9, 46, 10]);
}

#[test]
fn marvel_unison_vectors() {
    let limit = super::PrimeLimit::new(11);
    let lt = make_marvel(&limit);
    let n_results = 5;
    let uvs = lt.unison_vectors(n_results);
    assert!(uvs.len() <= n_results);
    assert!(uvs.contains(&vec![2, 3, 1, -2, -1]));
    assert!(uvs.contains(&vec![-5, 2, 2, -1, 0]));
    assert!(uvs.contains(&vec![-7, -1, 1, 1, 1]));
}

#[test]
fn porcupine_unison_vectors() {
    let limit = super::PrimeLimit::new(11);
    let lt = make_porcupine(&limit);
    let n_results = 5;
    let uvs = lt.unison_vectors(n_results);
    assert!(uvs.len() <= n_results);
    assert!(uvs.contains(&vec![-1, -3, 1, 0, 1]));
    assert!(uvs.contains(&vec![6, -2, 0, -1, 0]));
    assert!(uvs.contains(&vec![2, -2, 2, 0, -1]));
}

#[test]
fn test_ambiguous_et() {
    let limit = super::PrimeLimit::new(11).pitches;
    assert!(ambiguous_et(&limit, &vec![1, 2, 3, 4, 5]));
}

#[test]
fn test_ambiguity_31_11() {
    let limit = super::PrimeLimit::new(11).pitches;
    let et = prime_mapping(&limit, 31);
    assert!(!ambiguous_et(&limit, &et));
}

#[test]
fn test_ambiguity_41_11() {
    let limit = super::PrimeLimit::new(11).pitches;
    let et = prime_mapping(&limit, 41);
    assert!(!ambiguous_et(&limit, &et));
}

#[test]
fn test_ambiguity_12_11() {
    let limit = super::PrimeLimit::new(11).pitches;
    let et = prime_mapping(&limit, 12);
    assert!(!ambiguous_et(&limit, &et));
}

#[test]
fn test_ambiguity_12_13() {
    let limit = super::PrimeLimit::new(13).pitches;
    let et = prime_mapping(&limit, 12);
    assert!(ambiguous_et(&limit, &et));
}

#[test]
fn test_ambiguity_19_13() {
    let limit = super::PrimeLimit::new(13).pitches;
    let et = prime_mapping(&limit, 19);
    assert!(ambiguous_et(&limit, &et));
}

#[test]
fn test_ambiguity_31_13() {
    let limit = super::PrimeLimit::new(13).pitches;
    let et = prime_mapping(&limit, 31);
    assert!(!ambiguous_et(&limit, &et));
}

#[test]
fn marvel_from_et_names() {
    let limit = super::PrimeLimit::new(11);
    let original = make_marvel(&limit);
    let named = CangwuTemperament::from_et_names(
        &limit,
        &vec!["22".to_string(), "31".to_string(), "41".to_string()],
    );
    assert!(named.is_some());
    if let Some(rt) = named {
        assert_eq!(original.melody, rt.melody);
    }
}

#[test]
fn meantone_from_et_names() {
    let limit = super::PrimeLimit::new(13);
    let expected =
        vec![vec![31, 49, 72, 87, 107, 115], vec![12, 19, 28, 34, 42, 45]];
    let named = CangwuTemperament::from_et_names(
        &limit,
        &vec!["31".to_string(), "12f".to_string()],
    );
    assert!(named.is_some());
    if let Some(rt) = named {
        assert_eq!(rt.melody, expected);
    }
}

#[test]
fn marvel_from_key() {
    let limit11 = super::PrimeLimit::new(11);
    let original = make_marvel(&limit11);
    let clone = CangwuTemperament::from_ets_and_key(
        &limit11.pitches,
        &[22, 31, 41],
        &original.key(),
    );
    assert!(clone.is_some());
    if let Some(clone) = clone {
        assert_eq!(original.melody, clone.melody);
    }
}

#[test]
fn jove_from_key() {
    let limit11 = super::PrimeLimit::new(11);
    let original = make_jove(&limit11);
    let clone = CangwuTemperament::from_ets_and_key(
        &limit11.pitches,
        &[27, 31, 41],
        &original.key(),
    );
    assert!(clone.is_some());
    if let Some(clone) = clone {
        assert_eq!(original.melody, clone.melody);
    }
}

/// Regression test from the web interface
#[test]
fn porcupine_from_key() {
    let limit11 = super::PrimeLimit::new(11);
    let key = vec![3, 5, -6, 4, 1, 2, 3, 2, 4];
    let mapping = key_to_mapping(limit11.pitches.len(), &key);
    assert_eq!(
        Some(vec![vec![1, 2, 3, 2, 4], vec![0, 3, 5, -6, 4]]),
        mapping,
    );
    match CangwuTemperament::from_ets_and_key(
        &limit11.pitches,
        &[15, 22],
        &key,
    ) {
        Some(_) => (),
        None => assert!(false),
    }
}

#[cfg(test)]
fn octaves(mappings: &Vec<super::ETMap>) -> super::ETMap {
    mappings.iter().map(|m| m[0]).collect()
}
