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
/// Simplify type declarations, like types are intended for
pub type ETMap = Vec<FactorElement>;

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
        let plimit = prime_numbers.iter()
                        .map(|p| cents(*p as f64))
                        .collect();
        PrimeLimit{ numbers: prime_numbers, pitches: plimit }
    }

    pub fn partials(&self) -> Vec<(Harmonic, Cents)> {
        self.numbers.iter().cloned()
            .zip(self.pitches.iter().cloned())
            .collect()
    }
}

/// Equal temperament mapping with each prime rounded
/// to the nearest division of the equivalence interval
pub fn prime_mapping(plimit: &Vec<Cents>, n_notes: FactorElement)
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
            items: Vec::new(),
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

    pub fn extract(mut self) -> Vec<T> {
        // Could return the iterator but that's
        // harder to get the type of
        self.items.drain(..).map(|(_, item)| item).collect()
    }

    fn sort(&mut self) {
        self.items.sort_by(
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
