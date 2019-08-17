fn main() {
    let limit = regular::PrimeLimit::new(50);
    for (n, x) in limit.partials() {
        println!("{}: {}", n, x);
    }
    for mapping in regular::cangwu::limited_mappings(
            19, 1.0, 1e2, &regular::PrimeLimit::new(7).pitches) {
        println!("{:?}", mapping);
    }
}
