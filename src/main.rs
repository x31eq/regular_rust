fn main() {
    let primes = primes_below(50);
    let mut plimit: Vec<f64> = Vec::new();
    for p in primes.iter() {
        print!(" {}", p);
        plimit.push(cents(*p as f64));
    }
    println!("");
    for x in plimit.iter() {
        println!("{}", x);
    }
}

fn cents(ratio: f64) -> f64 {
    ratio.log2() * 12e2
}

fn primes_below(n: u16) -> Vec<u16> {
    let n = n as usize;
    let mut hasfactors = vec![false; n - 2];
    let mut result: Vec<u16> = Vec::new();
    for i in 2..n {
        if !hasfactors[i - 2] {
            result.push(i as u16);
            let mut j = 2 * i;
            while j < n {
                hasfactors[j - 2] = true;
                j += i;
            }
        }
    }
    result
}
