//! # Regular Temperament Utilities
//!
//! Utilties for regular temperament finding

extern crate nalgebra as na;
use na::{DMatrix, DVector};

pub type Cents = f64;
// Human hearing covers about 10 octaves,
// which means 11 bits (assuming the root is 1).
/// Integer partial
pub type Harmonic = u16;
/// Member of a "val" or ratio-lattice vector
pub type FactorElement = i32;
/// Simplify type declarations, like types are intended for
pub type ETMap = Vec<FactorElement>;
pub type Tuning = Vec<Cents>;

pub struct PrimeLimit {
    /// Something used for printing
    pub label: String,

    /// Pitch of each partial in cents above the root
    pub pitches: Tuning,
}

impl PrimeLimit {
    /// Constructor for consecutive prime limits
    /// given the highest prime
    /// (or a slightly higher composite)
    pub fn new(n: Harmonic) -> PrimeLimit {
        let mut result = PrimeLimit::explicit(primes_below(n + 1));
        result.label = format!("{}-limit", n.to_string());
        result
    }

    /// Explicit specification for non-consecutive prime limits
    /// (with no check for numbers being prime).
    pub fn explicit(prime_numbers: Vec<Harmonic>) -> PrimeLimit {
        let pitches = prime_numbers.iter()
                        .map(|p| cents(*p as f64))
                        .collect();
        let label = format!("{}-limit", join(".", &prime_numbers));
        PrimeLimit{ label, pitches }
    }

    /// Partials specified in cents
    pub fn inharmonic(pitches: Tuning) -> PrimeLimit {
        PrimeLimit{ label: "inharmonic".to_string(), pitches }
    }
}

fn join<T: ToString + Copy>(joiner: &str, items: &Vec<T>) -> String {
    let mut items = items.iter();
    let mut result = match items.next() {
        Some(item) => item.to_string(),
        None => "".to_string(),
    };
    for item in items {
        result.push_str(joiner);
        result.push_str(&item.to_string());
    }
    result
}

/// Equal temperament mapping with each prime rounded
/// to the nearest division of the equivalence interval
pub fn prime_mapping(plimit: &Tuning, n_notes: FactorElement)
        -> Vec<FactorElement> {
    let multiplier = n_notes as Cents / plimit[0];
    plimit.iter()
        .map(|x| (*x * multiplier).round() as FactorElement)
        .collect()
}

/// Convert a frequency ratio to cents
pub fn cents(ratio: f64) -> Cents {
    ratio.log2() * 12e2
}

/// Eratosthenes sieve
fn primes_below(n: Harmonic) -> Vec<Harmonic> {
    let top = n as usize;
    let mut hasfactors = vec![false; top - 2];
    (2..n).filter(|i| {
        let i = *i as usize;
        if !hasfactors[i - 2] {
            let mut j = i;
            while { j += i; j < top } {
                hasfactors[j - 2] = true;
            }
        }
        !hasfactors[i - 2]
    })
    .collect()
}


pub fn hermite_normal_form(ets: &DMatrix<FactorElement>)
        -> DMatrix<FactorElement> {
    let mut echelon = echelon_form(ets);
    // Workaround for borrowing restriction on nested iterators
    let echelon_copy = echelon.clone();
    for (col, ncol) in echelon_copy
                        .column_iter()
                        .enumerate()
                        .skip(1) {
        if let Some((row, n)) = ncol.iter().enumerate().skip(1)
                                .find(|(_i, n)| **n != 0) {
            assert!(*n > 0);
            for mut scol in echelon.column_iter_mut().take(col) {
                let m = scol[row] / *n;
                let col_copy = DVector::from_iterator(
                    ncol.nrows(), ncol.iter().cloned());
                scol -= m * col_copy.clone();
                // correct for round-towards-zero
                if scol[row] < 0 {
                    scol += col_copy;
                }
                assert!(scol[row] >= 0);
            }
        }
    }
    echelon
}

fn echelon_form(ets: &DMatrix<FactorElement>)
        -> DMatrix<FactorElement> {
    let (nrows, ncols) = ets.shape();
    if nrows == 0 {
        return ets.clone();
    }
    let mut working = Vec::with_capacity(ncols);
    for row in ets.column_iter() {
        working.push(DVector::from_iterator(
                nrows, row.iter().cloned()));
    }
    DMatrix::from_columns(&echelon_rec(working, 0))
}

fn echelon_rec(mut working: Vec<DVector<FactorElement>>, row: usize)
        -> Vec<DVector<FactorElement>> {
    // Normalize so the first nonzero entry in each column is positive
    for column in working.iter_mut() {
        if let Some(first_non_zero) = column.iter().find(|&&n| n != 0) {
            if *first_non_zero < 0 {
                *column *= -1;
            }
        }
    }

    if working.len() == 0 {
        return working;
    }
    let nrows = working[0].len();

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

        working.sort_unstable_by(|a, b|
                    match a.iter().zip(b.iter())
                            .find(|(x, y)| **x != 0 || **y != 0) {
                        Some((p, q)) => p.cmp(q),
                        None => std::cmp::Ordering::Equal,
                    });
        let mut workings = working.iter_mut();
        let pivot = workings.next().unwrap();
        let pivot_element = pivot[row];
        // pivot_element must be non-zero or it would be in reduced
        for col in workings {
            let n = col[row] / pivot_element;
            *col -= pivot.clone() * n;
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

impl <T> PriorityQueue<T> {
    pub fn new(size: usize) -> PriorityQueue<T> {
        PriorityQueue{
            cap: std::f64::INFINITY,
            size: size,
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
        }
        else {
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

    pub fn extract(&mut self) -> Vec<T> {
        // Could return the iterator but that's
        // harder to get the type of
        self.items.drain(..).map(|(_, item)| item).collect()
    }

    fn sort(&mut self) {
        self.items.sort_unstable_by(
            |(bad1, _), (bad2, _)|
            bad1.partial_cmp(&bad2).unwrap()
        );
    }

    fn set_cap(&mut self) {
        if let Some((bad, _)) = self.items.last() {
            self.cap = *bad;
        }
    }
}

pub mod cangwu;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cangwu_tests;
