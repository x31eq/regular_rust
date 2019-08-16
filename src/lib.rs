type Cents = f64;
// Irrational ratio
type Irratio = f64;
// Human hearing covers about 10 octaves,
// which means 11 bits.
type Harmonic = u16;

pub struct PrimeLimit {
    pub numbers: Vec<Harmonic>,
    pub pitches: Vec<Cents>,
}

impl PrimeLimit {
    pub fn new(n: Harmonic) -> PrimeLimit {
        let prime_numbers = primes_below(n + 1);
        let plimit = prime_numbers.iter()
                        .map(|p| cents(*p as Irratio))
                        .collect();
        PrimeLimit{ numbers: prime_numbers, pitches: plimit }
    }

    pub fn partials(&self) -> Vec<(Harmonic, Cents)> {
        self.numbers.iter().cloned()
            .zip(self.pitches.iter().cloned())
            .collect()
    }
}

pub fn cents(ratio: Irratio) -> Cents {
    ratio.log2() * 12e2
}

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

#[cfg(test)]
mod tests;
