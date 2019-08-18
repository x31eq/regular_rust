//! Temperament finding with Cangwu badness

use super::{Cents, FactorElement};

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
    let plimit: Vec<Cents> = plimit.iter().cloned().map(|x| x/12e2).collect();
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
    let weighted_size = (mapping[i - 1] as Cents) / plimit[i - 1];
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
            let deficit: Cents = ((
                (i + 1) as Cents * (cap - error2)
                / (i as Cents + epsilon2)
                ) as Cents).sqrt();
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
fn intrange(x: Cents, y: Cents) -> std::ops::RangeInclusive<FactorElement> {
    ((x.ceil() as FactorElement) ..= (y.floor() as FactorElement))
}
