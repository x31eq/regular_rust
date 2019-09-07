//! Temperament finding with Cangwu badness

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use super::{Cents, FactorElement, ETMap, PriorityQueue};

pub fn equal_temperament_badness(
        plimit: &Vec<Cents>, ek: Cents, mapping: &ETMap)
        -> Cents {
    assert_eq!(plimit.len(), mapping.len());
    // Put the primes in terms of octaves
    let plimit: Vec<f64> = plimit.into_iter()
        .map(|p| p / 12e2)
        .collect();
    // Get a dimensionless ek
    let ek = ek / 12e2;
    let epsilon = ek / (1.0 + square(ek)).sqrt();
    let weighted_mapping: Vec<f64> = mapping.iter()
        .zip(plimit.into_iter())
        .map(|(m, p)| (*m as f64) / p)
        .collect();
    let mean = |items: &Vec<f64>| {
        let mut sum = 0.0;
        for item in items.into_iter() {
            sum += item;
        }
        sum / (items.len() as f64)
    };
    let mean_w = mean(&weighted_mapping);
    let translation = (1.0 - epsilon) * mean_w;
    let bad2 = mean(&weighted_mapping.into_iter()
        .map(|x| x - translation)
        .map(square)
        .collect());
    bad2.sqrt() * 12e2
}

static NTHREADS: i32 = 3;

/// Get the best equal temperament mappings for the given prime limit
///
/// plimit: Sizes of prime harmonics in cents
///
/// ek: The Cangwu parameter in cents/octave
///
/// n_results: How many to return
pub fn get_equal_temperaments(
        plimit: &Vec<Cents>, ek: Cents, n_results: usize)
        -> Vec<ETMap> {
    // Stop weird things happening for non-standard units
    let plimit: Vec<Cents> = plimit.into_iter()
        .map(|p| 12e2 * (p / plimit[0]))
        .collect();

    let mut results = PriorityQueue::new(n_results);
    let (tx, rx) : (Sender<ETMap>, Receiver<ETMap>)
                   = mpsc::channel();
    let mut children = Vec::new();
    let bmax = preliminary_badness(&plimit, ek, n_results);
    for thread_id in 0..NTHREADS {
        let thread_tx = tx.clone();
        // Each thread needs its one copy of this
        let plimit = plimit.clone();
        let child = thread::spawn(move || {
            let mut n_notes = 1 + thread_id;
            let mut cap = bmax;
            // Keep a local queue to control the cap
            let mut thread_results = PriorityQueue::new(n_results);
            while (n_notes as f64) < cap / ek {
                for mapping in limited_mappings(
                        n_notes, ek, cap, &plimit) {
                    let bad = equal_temperament_badness(
                        &plimit, ek, &mapping);
                    thread_results.push(bad, mapping.clone());
                    thread_tx.send(mapping).expect("Couldn't send");
                }
                n_notes += thread_id;
                cap = cap.min(thread_results.cap);
            }
        });
        children.push(child);
    }
    drop(tx);

    for mapping in rx {
        let bad = equal_temperament_badness(&plimit, ek, &mapping);
        results.push(bad, mapping);
        if results.len() == n_results {
            break;
        }
    }

    for child in children {
        child.join().expect("Threading error");
    }

    debug_assert!(results.len() == n_results);
    results.extract()
}

/// High guess for the worst badness of a search.
/// Must be a reasonable cap, and at least as high
/// as the worst result we want to keep in the real search.
fn preliminary_badness(
        plimit: &Vec<Cents>, ek: Cents, n_results: usize)
        -> Cents {
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
pub fn limited_mappings(n_notes: FactorElement,
                    ek: Cents,
                    bmax: Cents,
                    plimit: &Vec<Cents>,
                    ) -> Vec<ETMap> {
    // Call things Cents but turn them to octaves/dimensionless
    let ek = ek / 12e2;
    let bmax = bmax / 12e2;
    let cap = square(bmax) * (plimit.len() as Cents) / square(plimit[0]);
    let epsilon2 = square(ek) / (1.0 + square(ek));
    let mut mapping = vec![n_notes; plimit.len()];
    let mut results = Vec::new();

    more_limited_mappings(&mut mapping, 1,
                          0.0, 0.0, cap, epsilon2, &plimit, &mut results);
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
fn more_limited_mappings(mut mapping: &mut ETMap,
                            i: usize,
                            tot: Cents,
                            tot2: Cents,
                            cap: Cents,
                            epsilon2: Cents,
                            plimit: &Vec<Cents>,
                            mut results: &mut Vec<ETMap>,
                            ) {
    assert!(mapping.len() == plimit.len());
    let weighted_size = (mapping[i - 1] as f64) / plimit[i - 1];
    let tot = tot + weighted_size;
    let tot2 = tot2 + square(weighted_size);
    let lambda = 1.0 - epsilon2;
    debug_assert!(tot2 <= lambda*square(tot)/(i as f64) + cap);
    if i == plimit.len() {
        // Recursion stops here.
        // Clone the object to save as the one being worked on
        // keeps changing
        results.push(mapping.clone());
    }
    else {
        let toti = tot * lambda / ((i as f64) + epsilon2);
        let error2 = tot2 - tot * toti;
        if error2 < cap {
            let target = plimit[i];
            let deficit = (
                (i + 1) as f64 * (cap - error2) / (i as f64 + epsilon2)
                ).sqrt();
            let xmin = target * (toti - deficit);
            let xmax = target * (toti + deficit);
            for guess in intrange(xmin, xmax) {
                mapping[i] = guess;
                more_limited_mappings(
                    &mut mapping, i + 1, tot, tot2,
                    cap, epsilon2, &plimit, &mut results);
            }
        }
    }
}

fn square(x: f64) -> f64 {
    x.powi(2)
}

/// Range of integers between x and y
fn intrange(x: f64, y: f64) -> std::ops::RangeInclusive<FactorElement> {
    ((x.ceil() as FactorElement) ..= (y.floor() as FactorElement))
}
