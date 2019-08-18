fn main() {
    let limit = regular::PrimeLimit::new(50);
    for (n, x) in limit.partials() {
        println!("{}: {}", n, x);
    }
    let limit11 = &regular::PrimeLimit::new(11).pitches;
    for mapping in regular::cangwu::get_equal_temperaments(
            limit11, 1.0, 10) {
        println!("{:?}", mapping);
    }
    println!("{:}", regular::cangwu::equal_temperament_badness(
            limit11, 1.0, &vec![31, 49, 72, 87, 107]));
}
