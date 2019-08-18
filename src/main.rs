fn main() {
    let limit = regular::PrimeLimit::new(50);
    for (n, x) in limit.partials() {
        println!("{}: {}", n, x);
    }
    for mapping in regular::cangwu::get_equal_temperaments(
            &regular::PrimeLimit::new(11).pitches, 1.0, 10) {
        println!("{:?}", mapping);
    }
}
