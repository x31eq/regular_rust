fn main() {
    let limit = regular::PrimeLimit::new(50);
    for (n, x) in limit.partials() {
        println!("{}: {}", n, x);
    }
}
