//! Temperament finding with Cangwu badness

use super::{Cents, FactorElement};

/// Get the best equal temperament mappings for the given prime limit
///
/// plimit: Sizes of prime harmonics in cents
///
/// ek: The Cangwu parameter in cents/octave
///
/// n_results: How many to return
pub fn get_equal_temperaments(
        plimit: &Vec<Cents>, ek: Cents, n_results: usize)
        -> Vec<Vec<FactorElement>> {
    // Stop weird things happening for non-standard units
    let plimit: Vec<Cents> = plimit.iter().cloned()
        .map(|p| 12e2 * (p / plimit[0]))
        .collect();

    // Low initial guess
    let mut bmax = ek * plimit.len() as f64;
    // the trait `std::cmp::Ord` is not implemented for `f64`
    if bmax > 12.0 {
        bmax = 12.0;
    }
    let mut results = Vec::new();
    // Stop search getting out of control
    for _ in 0..100 {
        results.truncate(0);
        let mut n_notes = 1;
        while (n_notes as f64) < bmax / ek {
            for mapping in limited_mappings(n_notes, ek, bmax, &plimit) {
                results.push(mapping);
            }
            n_notes += 1;
        }
        // This should be sorted by badness but
        // we don't have that calculation yet
        if results.len() >= n_results {
            return results;
        }
        bmax *= 1.5;
    }
    // Couldn't find enough, return whatever we have
    results
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
pub fn limited_mappings(n_notes: FactorElement,
                    ek: Cents,
                    bmax: Cents,
                    plimit: &Vec<Cents>,
                    ) -> Vec<Vec<FactorElement>> {
    // Call things Cents but turn them to octaves/dimensionless
    let ek = ek / 12e2;
    let bmax = bmax / 12e2;
    let cap = bmax.powi(2) * (plimit.len() as Cents) / (plimit[0].powi(2));
    let epsilon2 = ek.powi(2) / (1.0 + ek.powi(2));

    more_limited_mappings(vec![n_notes], 0.0, 0.0, cap, epsilon2, &plimit)
}

/// Helper function for limited_mappings that can't be a closure
/// because it's recursive and can't use a nested scope because
/// functions don't do that, so may as well be at top level.
///
/// mapping: the ET mapping with a new entry
///
/// tot: running total of w
///
/// tot2: running total of w squared
fn more_limited_mappings(mapping: Vec<FactorElement>,
                            tot: Cents,
                            tot2: Cents,
                            cap: Cents,
                            epsilon2: Cents,
                            plimit: &Vec<Cents>,
                            ) -> Vec<Vec<FactorElement>> {
    let mut result = Vec::new();
    let i = mapping.len();
    let weighted_size = (mapping[i - 1] as f64) / plimit[i - 1];
    let tot = tot + weighted_size;
    let tot2 = tot2 + weighted_size.powi(2);
    let lambda = 1.0 - epsilon2;
    if i == plimit.len() {
        // recursion stops here
        result.push(mapping);
    }
    else {
        let toti = tot * lambda / ((i as Cents) + epsilon2);
        let error2 = tot2 - tot * toti;
        if error2 < cap {
            let target = plimit[i];
            let deficit: f64 = (
                (i + 1) as f64 * (cap - error2) / (i as f64 + epsilon2)
                ).sqrt();
            let xmin = target * (toti - deficit);
            let xmax = target * (toti + deficit);
            for guess in intrange(xmin, xmax) {
                let mut next_mapping = mapping.clone();
                next_mapping.push(guess);
                let results = more_limited_mappings(
                    next_mapping, tot, tot2,
                    cap, epsilon2, &plimit);
                for new_result in results {
                    result.push(new_result);
                }
            }
        }
    }
    result
}

/// Range of integers between x and y
fn intrange(x: f64, y: f64) -> std::ops::RangeInclusive<FactorElement> {
    ((x.ceil() as FactorElement) ..= (y.floor() as FactorElement))
}
