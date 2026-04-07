//! Random number generators — Di Pierro Ch. 6
//! cargo run --example random
use nlib::random::{Lcg, Mt19937};

fn main() {
    // MINSTD LCG
    let mut lcg = Lcg::new(1, 16807, 0, 2_147_483_647);
    print!("LCG (MINSTD) first 10: ");
    for _ in 0..10 {
        print!("{} ", lcg.next_val());
    }
    println!();

    // MT19937
    let mut mt = Mt19937::new(42);
    print!("MT19937 first 10: ");
    for _ in 0..10 {
        print!("{} ", mt.next_u32());
    }
    println!();

    // Uniformity check
    let mut lcg = Lcg::new(1, 16807, 0, 2_147_483_647);
    let n = 10_000;
    let mut buckets = [0u32; 10];
    for _ in 0..n {
        let idx = (lcg.next_f64() * 10.0) as usize;
        buckets[idx.min(9)] += 1;
    }
    println!("\nLCG uniformity ({n} samples, 10 bins):");
    for (i, &count) in buckets.iter().enumerate() {
        let bar: String = std::iter::repeat_n('█', (count / 20) as usize).collect();
        println!("  [{i}] {count:>5} {bar}");
    }
}
