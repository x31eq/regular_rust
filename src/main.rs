fn main() {
    let (plimit, nums) = regular::prime_limit(50);
    for (n, x) in nums.iter().zip(plimit) {
        println!("{}: {}", n, x);
    }
}
