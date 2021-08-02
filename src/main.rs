use regular::PrimeLimit;
use std::io::{self, stdout, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let limit = match args.len() {
        0 | 1 | 2 | 3 => panic!(
            "{} {}",
            "Supply the number of results, badness parameter,",
            "and prime limit as command line arguments",
        ),
        4 => {
            if args[3] == "cents" {
                read_cents()
            } else {
                args[3].parse().unwrap()
            }
        }
        _ => PrimeLimit::explicit(
            args[3..].iter().map(|m| m.parse().unwrap()).collect(),
        ),
    };
    let n_results: usize = args[1].parse().unwrap();
    let ek: regular::Cents = args[2].parse().unwrap();

    let dimension = limit.pitches.len();
    let safety = if dimension < 100 {
        40
    } else {
        4 * (dimension as f64).sqrt().floor() as usize
    };
    let mappings = regular::cangwu::get_equal_temperaments(
        &limit.pitches,
        ek,
        n_results + safety,
    );
    let mut rts = Vec::with_capacity(mappings.len());
    for mapping in mappings.iter() {
        rts.push(vec![mapping.clone()]);
    }
    for rank in 2..dimension {
        let eff_n_results =
            n_results + if rank == dimension - 1 { 0 } else { safety };
        let new_rts = regular::cangwu::higher_rank_search(
            &limit.pitches,
            &mappings,
            &rts,
            ek,
            eff_n_results,
        );
        rts.truncate(n_results);
        if print_return_closed(&rts) {
            // Return silently if stdout is closed
            return;
        }
        rts = new_rts;
    }
    print_return_closed(&rts);
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

/// Print debug to stdout or return true if stdout is closed
fn print_return_closed<T: std::fmt::Debug>(obj: &T) -> bool {
    // This is like println! but without the panic
    stdout()
        .write_all(&format!("{:?}\n", obj).into_bytes())
        .is_err()
}
