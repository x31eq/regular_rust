fn main() {
    let args: Vec<String> = std::env::args().collect();
    let limit = match args.len() {
        0 | 1 | 2 | 3 => panic!(
            format!("{} {}",
                "Supply the number of results, badness parameter,",
                "and prime limit as command line arguments")),
        4 => regular::PrimeLimit::new(args[3].parse().unwrap()),
        _ => regular::PrimeLimit::explicit(
            args.iter().skip(3)
            .map(|m| m.parse().unwrap())
            .collect()),
    };
    let n_results: usize = args[1].parse().unwrap();
    let ek: regular::Cents = args[2].parse().unwrap();

    let mappings = regular::cangwu::get_equal_temperaments(
            &limit.pitches, ek, n_results);
    println!("{}-limit",
             limit.numbers[limit.numbers.len() - 1]);
    let badness = regular::cangwu::equal_temperament_badness(
                &limit.pitches,
                ek,
                &mappings[mappings.len() - 1]);
    for et in mappings {
        println!("{:?}", et);
    }
    println!("Badness of worst in the list {:?}", badness);
}
