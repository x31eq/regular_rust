//! # Regular Temperament Utilities
//!
//! Utilities for regular temperament finding

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
        let pitches = map(|p| cents(f64::from(*p)), &prime_numbers);
        let label = join(".", &prime_numbers);
        let headings = map(Harmonic::to_string, &prime_numbers);
        PrimeLimit {
            label,
            pitches,
            headings,
        }
    }

    /// Partials specified in cents
    pub fn inharmonic(pitches: Tuning) -> Self {
        let headings = map(Cents::to_string, &pitches);
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

/// Standard name, based on Herman Miller's suggestion.
/// The basic name is the number of steps to the octave.
/// If it has the best approximation to each prime
/// (it is the patent val) a 'p' is appended (not in Herman's suggestion).
/// Otherwise, a letter is appended for each prime that differs from
/// its best approximation.
/// Ambiguity is not considered here.
/// The letters the start of the alphabet for prime limits
/// or letters from q for non-prime harmonics.
pub fn warted_et_name(plimit: &PrimeLimit, et: &ETSlice) -> String {
    let mut next_inharmonic_wart = 'q';
    let mut warts = vec![];
    for harmonic in &plimit.headings {
        if let Some(c) = wart_for_prime(harmonic) {
            warts.push(c);
        } else {
            warts.push(next_inharmonic_wart);
            next_inharmonic_wart = next_char(next_inharmonic_wart);
        }
    }
    let mut name = et[0].to_string();
    if plimit.headings[0] != "2" {
        name.insert(0, warts[0]);
    }
    let prime_et = prime_mapping(&plimit.pitches, et[0]);
    if prime_et == et {
        return name + "p";
    }
    let n_notes_scale = et[0] as f64 / plimit.pitches[0];
    for (&et_i, (&pet_i, (&pitch, &wart))) in et
        .iter()
        .zip(prime_et.iter().zip(plimit.pitches.iter().zip(warts.iter())))
    {
        if et_i != pet_i {
            let delta = et_i.abs_diff(pet_i) as isize;
            let sign = if (pet_i as f64) < pitch * n_notes_scale {
                1
            } else {
                -1
            };
            let freq = 2 * delta
                - (if et_i as isize == pet_i as isize - sign * delta {
                    0
                } else {
                    1
                });
            for _ in 0..freq {
                name = name + &wart.to_string();
            }
        }
    }
    name
}

fn wart_for_prime(heading: &str) -> Option<char> {
    if let Ok(n) = heading.parse::<Harmonic>() {
        if n > 47 {
            // 47 is the highest prime we have a letter for
            return None;
        }
        let mut next_wart = 'a';
        for p in primes_below(48).into_iter() {
            if n == p {
                return Some(next_wart);
            }
            next_wart = next_char(next_wart);
        }
    }
    None
}

fn next_char(current: char) -> char {
    (current as u8 + 1) as char
}

#[derive(Debug)]
pub struct ParseLimitError {}

impl fmt::Display for ParseLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Unrecognized prime limit".fmt(f)
    }
}

pub fn normalize_positive(limit: &PrimeLimit, rsvec: ETMap) -> ETMap {
    let pitch_width = limit
        .pitches
        .iter()
        .zip(rsvec.iter())
        .fold(0.0, |acc, (&x, &y)| acc + x * (y as Cents));
    if pitch_width < 0.0 {
        rsvec.iter().map(|x| -x).collect()
    } else {
        rsvec.clone()
    }
}

/// Some generic utilities
fn map<T, U>(f: impl FnMut(&T) -> U, v: &[T]) -> Vec<U> {
    v.iter().map(f).collect()
}

fn join<T>(joiner: &str, items: &[T]) -> String
where
    T: ToString + Copy,
{
    map(ToString::to_string, items).join(joiner)
}

/// Equal temperament mapping with each prime rounded
/// to the nearest division of the equivalence interval
pub fn prime_mapping(plimit: &[Cents], n_notes: Exponent) -> Vec<Exponent> {
    let multiplier = Cents::from(n_notes) / plimit[0];
    map(|&x| (x * multiplier).round() as Exponent, plimit)
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
            if *multiples.next().expect("Eratosthenes sieve underrun") {
                multiples.for_each(|multiple| *multiple = false);
            }
            sieve.pop().expect("Leaky Eratosthenes sieve")
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
        let top_col = col_iter.next().expect("No columns for hermite");
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
                *column = map(|&x| -x, column);
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
        let pivot = workings.next().expect("No pivot for echelon reduction");
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

    pub fn extract(self) -> impl Iterator<Item = T> {
        self.items.into_iter().map(|(_, item)| item)
    }

    fn sort(&mut self) {
        self.items.sort_unstable_by(|(bad1, _), (bad2, _)| {
            bad1.partial_cmp(bad2)
                .expect("Bad comparison: NaN or something")
        });
    }

    fn set_cap(&mut self) {
        if let Some((bad, _)) = self.items.last() {
            self.cap = *bad;
        }
    }
}

pub mod cangwu;
pub mod names;
pub mod ratio;
pub mod te;
pub mod temperament_class;
pub mod uv;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
mod tests;
