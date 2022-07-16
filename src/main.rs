use regular::{Harmonic, PrimeLimit};
use std::io::{self, stdout, BufRead, Write};

fn main() {
    let mut args = std::env::args();

    let (n_results, ek, limit) = {
        if let (Some(_), Some(n_results), Some(ek), Some(limit1)) =
            (args.next(), args.next(), args.next(), args.next())
        {
            let n_results: usize = n_results.parse().unwrap();
            let ek: regular::Cents = ek.parse().unwrap();

            let limit = if limit1 == "cents" {
                assert!(args.next() == None);
                read_cents()
            } else {
                let limit1: Harmonic = limit1.parse().unwrap();
                let mut harmonics: Vec<Harmonic> =
                    args.map(|m| m.parse().unwrap()).collect();
                if harmonics.len() == 0 {
                    PrimeLimit::new(limit1)
                } else {
                    harmonics.insert(0, limit1);
                    PrimeLimit::explicit(harmonics)
                }
            };
            (n_results, ek, limit)
        } else {
            panic!(
                "{} {}",
                "Supply the number of results, badness parameter,",
                "and prime limit as command line arguments",
            )
        }
    };

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
