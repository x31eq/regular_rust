//! # Regular Temperament Utilities
//!
//! Utilties for regular temperament finding

pub type Cents = f64;
// Human hearing covers about 10 octaves,
// which means 11 bits (assuming the root is 1).
/// Integer partial
pub type Harmonic = u16;
/// Member of a "val" or ratio-lattice vector
pub type FactorElement = i32;

pub struct PrimeLimit {
    /// Numbers representing partials
    pub numbers: Vec<Harmonic>,

    /// Pitch of each partial in cents above the root
    pub pitches: Vec<Cents>,
}

impl PrimeLimit {
    /// Constructor for consecutive prime limits
    /// given the highest prime
    /// (or a slightly higher composite)
    pub fn new(n: Harmonic) -> PrimeLimit {
        let prime_numbers = primes_below(n + 1);
        let plimit = prime_numbers.iter().cloned()
                        .map(|p| cents(p as f64))
                        .collect();
        PrimeLimit{ numbers: prime_numbers, pitches: plimit }
    }

    pub fn partials(&self) -> Vec<(Harmonic, Cents)> {
        self.numbers.iter().cloned()
            .zip(self.pitches.iter().cloned())
            .collect()
    }
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

pub mod cangwu;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cangwu_tests;
