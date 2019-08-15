pub fn prime_limit(n: u16) -> (Vec<f64>, Vec<u16>) {
    let prime_numbers = primes_below(n + 1);
    (prime_numbers.iter()
                    .map(|p| cents(*p as f64))
                    .collect(), prime_numbers)
}

pub fn cents(ratio: f64) -> f64 {
    ratio.log2() * 12e2
}

fn primes_below(n: u16) -> Vec<u16> {
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
