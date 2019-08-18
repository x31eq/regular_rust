fn main() {
    let limit = regular::PrimeLimit::new(50);
    for (n, x) in limit.partials() {
        println!("{}: {}", n, x);
    }
    println!("11-limit");
    let limit11 = regular::PrimeLimit::new(11).pitches;
    for mapping in regular::cangwu::get_equal_temperaments(
            &limit11, 1.0, 10) {
        println!("{:?}", mapping);
    }
    println!("11-limit 1-cent badness of 31-equal: {}",
             regular::cangwu::equal_temperament_badness(
                &limit11, 1.0, &vec![31, 49, 72, 87, 107]));
    let big_limit = regular::PrimeLimit::new(100);
    let mappings = regular::cangwu::get_equal_temperaments(
            &big_limit.pitches, 0.3, 10);
    println!("{}-limit",
             big_limit.numbers[big_limit.numbers.len() - 1]);
    println!("{:?}", mappings.iter()
                        .map(|m| m[0])
                        .collect::<Vec<_>>());
    println!("Badness of worst in the list {:?}",
             regular::cangwu::equal_temperament_badness(
                &big_limit.pitches,
                0.3,
                &mappings[mappings.len() - 1]));
}
