use std::io::{self, BufRead};
use regular::PrimeLimit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let limit = match args.len() {
        0 | 1 | 2 | 3 => panic!(
            format!("{} {}",
                "Supply the number of results, badness parameter,",
                "and prime limit as command line arguments")),
        4 => if args[3] == "cents".to_string() {
                read_cents()
            }
            else {
                PrimeLimit::new(args[3].parse().unwrap())
            },
        _ => PrimeLimit::explicit(
            args.iter().skip(3)
            .map(|m| m.parse().unwrap())
            .collect()),
    };
    let n_results: usize = args[1].parse().unwrap();
    let ek: regular::Cents = args[2].parse().unwrap();

    let mappings = regular::cangwu::get_equal_temperaments(
            &limit.pitches, ek, n_results + 10);
    let mut rts = Vec::with_capacity(mappings.len());
    for mapping in mappings.iter() {
        rts.push(vec![mapping.clone()]);
    }
    println!("{:?}", rts[..n_results].to_vec());
    for _rank in 2 .. limit.pitches.len() {
        rts = regular::cangwu::higher_rank_search(
            &limit.pitches, &mappings, &rts, ek, n_results + 10);
        println!("{:?}", rts[..n_results].to_vec());
    }
}

fn read_cents() -> PrimeLimit {
    println!("List your partials in cents, one to a line");
    let mut result = Vec::new();
    for line in io::stdin().lock().lines() {
        let text = line.unwrap();
        match text.parse() {
            Ok(partial) => result.push(partial),
            Err(_) => println!("Failed to parse {}", text),
        };
    }
    PrimeLimit::inharmonic(result)
}
