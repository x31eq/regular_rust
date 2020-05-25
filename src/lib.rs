//! # Regular Temperament Utilities
//!
//! Utilties for regular temperament finding

use num_integer::div_floor;
use std::fmt;
use std::str::FromStr;

pub type Cents = f64;
// Human hearing covers about 10 octaves,
// which means 11 bits (assuming the root is 1).
/// Integer partial
pub type Harmonic = u16;
/// Member of a "val" or ratio-lattice vector
pub type Exponent = i32;
/// Simplify type declarations, like types are intended for
pub type ETMap = Vec<Exponent>;
pub type ETSlice = [Exponent];
pub type Tuning = Vec<Cents>;
pub type Mapping = Vec<ETMap>;

pub struct PrimeLimit {
    /// Something used for printing
    pub label: String,

    /// Pitch of each partial in cents above the root
    pub pitches: Tuning,

    /// Something used for printing tables
    pub headings: Vec<String>,
}

impl PrimeLimit {
    /// Constructor for consecutive prime limits
    /// given the highest prime
    /// (or a slightly higher composite)
    pub fn new(n: Harmonic) -> Self {
        let mut result = PrimeLimit::explicit(primes_below(n + 1));
        result.label = n.to_string();
        result
    }

    /// Explicit specification for non-consecutive prime limits
    /// (with no check for numbers being prime).
    pub fn explicit(prime_numbers: Vec<Harmonic>) -> Self {
        let pitches =
            prime_numbers.iter().map(|p| cents(f64::from(*p))).collect();
        let label = join(".", &prime_numbers);
        let headings =
            prime_numbers.iter().map(Harmonic::to_string).collect();
        PrimeLimit {
            label,
            pitches,
            headings,
        }
    }

    /// Partials specified in cents
    pub fn inharmonic(pitches: Tuning) -> Self {
        let headings = pitches.iter().map(Cents::to_string).collect();
        PrimeLimit {
            label: "inharmonic".to_string(),
            pitches,
            headings,
        }
    }
}

impl FromStr for PrimeLimit {
    type Err = ParseLimitError;

    fn from_str(src: &str) -> Result<PrimeLimit, ParseLimitError> {
        // FIXME: prime limit of zero causes a runtime error
        if let Ok(limit) = src.parse() {
            Ok(PrimeLimit::new(limit))
        } else if let Ok(primes) = src.split('.').map(str::parse).collect() {
            Ok(PrimeLimit::explicit(primes))
        } else {
            Err(ParseLimitError {})
        }
    }
}

#[derive(Debug)]
pub struct ParseLimitError {}

impl fmt::Display for ParseLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Unrecognized prime limit".fmt(f)
    }
}

fn join<T: ToString + Copy>(joiner: &str, items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(joiner)
}

/// Equal temperament mapping with each prime rounded
/// to the nearest division of the equivalence interval
pub fn prime_mapping(plimit: &[Cents], n_notes: Exponent) -> Vec<Exponent> {
    let multiplier = Cents::from(n_notes) / plimit[0];
    plimit
        .iter()
        .map(|&x| (x * multiplier).round() as Exponent)
        .collect()
}

/// Convert a frequency ratio to cents
pub fn cents(ratio: f64) -> Cents {
    ratio.log2() * 12e2
}

/// Eratosthenes sieve
fn primes_below(n: Harmonic) -> Vec<Harmonic> {
    let mut sieve = vec![true; n as usize - 2];
    (2..n)
        .filter(|&i| {
            let mut multiples = sieve.iter_mut().rev().step_by(i as usize);
            if *multiples.next().unwrap() {
                multiples.for_each(|multiple| *multiple = false);
            }
            sieve.pop().unwrap()
        })
        .collect()
}

/// Convert the matrix to a unique column echelon form
/// with everything as simple as possible,
/// things positive when they can't be zero,
/// and within the same lattice (determinant conserved)
pub fn hermite_normal_form(ets: &[ETMap]) -> Mapping {
    let mut echelon = echelon_form(ets);
    for col in 1..echelon.len() {
        let mut col_iter = echelon[..=col].iter_mut().rev();
        // Getting top_col from the mutable iterator
        // ensures "echelon" is consistently borrowed
        let top_col = col_iter.next().unwrap();
        if let Some((row, &n)) =
            top_col.iter().enumerate().find(|(_i, &n)| n != 0)
        {
            assert!(n > 0);
            for scol in col_iter {
                let s = scol[row];
                if s == 0 {
                    continue;
                }
                for (x, y) in scol.iter_mut().zip(top_col.iter()) {
                    *x -= div_floor(s, n) * y;
                }
                assert!(scol[row] >= 0);
                assert!(scol[row] < n);
            }
        }
    }
    echelon
}

fn echelon_form(ets: &[ETMap]) -> Mapping {
    echelon_rec(ets.to_vec(), 0)
}

fn echelon_rec(mut working: Mapping, row: usize) -> Mapping {
    if working.is_empty() {
        return working;
    }
    let nrows = working[0].len();

    // Normalize so the first nonzero entry in each column is positive
    for column in working.iter_mut() {
        if let Some(first_non_zero) = column.iter().find(|&&n| n != 0) {
            if *first_non_zero < 0 {
                *column = column.iter().map(|&x| -x).collect();
            }
        }
    }

    if row == nrows {
        return working;
    }
    assert!(row < nrows);

    let mut reduced = Vec::new();
    loop {
        for col in working.iter() {
            if col[row] == 0 {
                reduced.push(col.clone());
            }
        }
        working.retain(|col| col[row] != 0);

        if working.len() < 2 {
            working.extend_from_slice(&echelon_rec(reduced, row + 1));
            return working;
        }

        working.sort_unstable_by(|a, b| {
            match a.iter().zip(b.iter()).find(|(x, y)| **x != 0 || **y != 0) {
                Some((p, q)) => p.cmp(q),
                None => std::cmp::Ordering::Equal,
            }
        });
        let mut workings = working.iter_mut();
        let pivot = workings.next().unwrap();
        let pivot_element = pivot[row];
        // pivot_element must be non-zero or it would be in reduced
        assert!(pivot_element != 0);
        for col in workings {
            let n = col[row] / pivot_element;
            for (i, &x) in pivot.iter().enumerate() {
                col[i] -= x * n;
            }
        }
    }
}

/// Container to keep results ordered by badness
/// and throw away the bad ones.
/// Prioritized by badness: low values are preferred.
struct PriorityQueue<T> {
    pub cap: f64,
    size: usize,
    items: Vec<(f64, T)>,
}

impl<T> PriorityQueue<T> {
    pub fn new(size: usize) -> PriorityQueue<T> {
        PriorityQueue {
            cap: std::f64::INFINITY,
            size,
            // over-allocate because we push before we pop
            items: Vec::with_capacity(size + 1),
        }
    }

    pub fn push(&mut self, badness: f64, item: T) {
        if self.len() == self.size {
            if badness < self.cap {
                self.items.push((badness, item));
                self.sort();
                self.items.pop();
                self.set_cap();
            }
        } else {
            self.items.push((badness, item));
            self.sort();
            if self.len() == self.size {
                self.set_cap();
            }
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn extract(self) -> impl Iterator<Item=T> {
        self.items.into_iter().map(|(_, item)| item)
    }

    fn sort(&mut self) {
        self.items.sort_unstable_by(|(bad1, _), (bad2, _)| {
            bad1.partial_cmp(bad2).unwrap()
        });
    }

    fn set_cap(&mut self) {
        if let Some((bad, _)) = self.items.last() {
            self.cap = *bad;
        }
    }
}

pub mod cangwu;

pub mod te;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cangwu_tests;
