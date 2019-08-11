pub fn prime_limit(n: u16) -> Vec<(u16, f64)> {
    let mut result: Vec<(u16, f64)> = Vec::new();
    for p in primes_below(n + 1) {
        result.push((p, cents(p as f64)));
    }
    result
}

pub fn cents(ratio: f64) -> f64 {
    ratio.log2() * 12e2
}

fn primes_below(n: u16) -> Vec<u16> {
    let n = n as usize;
    let mut hasfactors = vec![false; n - 2];
    let mut result: Vec<u16> = Vec::new();
    for i in 2..n {
        if !hasfactors[i - 2] {
            result.push(i as u16);
            let mut j = i;
            while {
                j += i;
                j < n
            } {
                hasfactors[j - 2] = true;
            }
        }
    }
    result
}
