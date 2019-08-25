fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_results: usize = args[1].parse().unwrap();
    let prime_limit: regular::Harmonic = args[2].parse().unwrap();

    let limit = regular::PrimeLimit::new(prime_limit);
    let mappings = regular::cangwu::get_equal_temperaments(
            &limit.pitches, 0.3, n_results);
    println!("{}-limit",
             limit.numbers[limit.numbers.len() - 1]);
    let badness = regular::cangwu::equal_temperament_badness(
                &limit.pitches,
                0.3,
                &mappings[mappings.len() - 1]);
    for et in mappings {
        println!("{:?}", et);
    }
    println!("Badness of worst in the list {:?}", badness);
}
