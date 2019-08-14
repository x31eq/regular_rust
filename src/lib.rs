pub fn prime_limit(n: u16) -> Vec<(u16, f64)> {
    primes_below(n + 1).iter()
        .map(|p| (*p, cents(*p as f64)))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octave_cents() {
        assert_eq!(cents(2.0), 1200.0);
        assert_eq!(cents(4.0), 2400.0);
    }

    #[test]
    fn seven_limit() {
        let primes: Vec<u16> =
                prime_limit(7).iter()
                .map(|(p, _size)| *p)
                .collect();
        assert_eq!(primes, vec![2, 3, 5, 7]);
    }

    #[test]
    fn small_primes() {
        let from_pari = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31,
            37, 41, 43, 47, 53, 59, 61, 67, 71, 73,
            79, 83, 89, 97];
        assert_eq!(primes_below(100), from_pari);
    }
}
