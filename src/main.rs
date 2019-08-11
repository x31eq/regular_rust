use regular::prime_limit;

fn main() {
    for (n, x) in prime_limit(50) {
        println!("{}: {}", n, x);
    }
}
