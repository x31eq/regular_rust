fn main() {
    let limit = regular::PrimeLimit::new(50);
    let partials = limit.numbers.iter().zip(limit.pitches);
    for (n, x) in partials {
        println!("{}: {}", n, x);
    }
}
