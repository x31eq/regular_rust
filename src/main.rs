fn main() {
    let mut primes = primes_below(50);
    for p in &mut primes {
        println!("{} is prime", p);
    }
}

fn primes_below(n: u16) -> Vec<u16> {
    let mut hasfactors: Vec<bool> = Vec::new();
    for _ in 2..n {
        hasfactors.push(false);
    }
    for i in 2..(n/2) {
        if !hasfactors[i as usize - 2] {
            let mut j = 2 * i;
            while j < n {
                hasfactors[j as usize - 2] = true;
                j += i;
            }
        }
    }
    let mut result: Vec<u16> = Vec::new();
    for i in 2..n {
        if !hasfactors[i as usize - 2] {
            result.push(i);
        }
    }
    result
}
