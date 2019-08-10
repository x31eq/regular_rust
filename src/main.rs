fn main() {
    let mut primes = primes_below(50);
    for p in &mut primes {
        println!("{} is prime", p);
    }
}

fn primes_below(n: usize) -> Vec<usize> {
    let mut hasfactors: Vec<bool> = Vec::new();
    for _ in 2..n {
        hasfactors.push(false);
    }
    for i in 2..(n/2) {
        if !hasfactors[i - 2] {
            let mut j = 2 * i;
            while j < n {
                hasfactors[j - 2] = true;
                j += i;
            }
        }
    }
    let mut result: Vec<usize> = Vec::new();
    for i in 2..n {
        if !hasfactors[i - 2] {
            result.push(i);
        }
    }
    result
}
