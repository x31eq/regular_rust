pub fn prime_limit(n: u16) -> Vec<(u16, f64)> {
    primes_below(n + 1).iter()
        .map(|p| (*p, cents(*p as f64)))
        .collect()
}

pub fn cents(ratio: f64) -> f64 {
    ratio.log2() * 12e2
}

fn primes_below(n: u16) -> Vec<u16> {
    let top = n as usize;
    let mut hasfactors = vec![false; top - 2];
    (2..n).filter(|i| {
        let i = *i as usize;
        if hasfactors[i - 2] {
            false
        }
        else {
            let mut j = i;
            while {
                j += i;
                j < top
            } {
                hasfactors[j - 2] = true;
            }
            true
        }
    })
    .collect()
}

#[cfg(test)]
mod tests;
