fn main() {
    for (n, x) in regular::prime_limit(50) {
        println!("{}: {}", n, x);
    }
}
