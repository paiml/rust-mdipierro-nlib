//! Random number generators — Di Pierro, *Annotated Algorithms in Python*, §6.4.
//! Contract: `contracts/example-random-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example random            # human-readable
//! cargo run --example random -- --json  # the receipt in evidence/examples/random.json
use nlib::random::{Lcg, Mt19937};
use nlib::receipt::{Receipt, json_requested};
use nlib::stats::chi_squared;

/// MINSTD (Park & Miller 1988): x ← 16807·x mod (2³¹ - 1).
pub fn minstd(seed: u64) -> Lcg {
    Lcg::new(seed, 16_807, 0, 2_147_483_647)
}

/// χ² critical value for 9 degrees of freedom at p = 0.01.
pub const CHI2_9DF_P01: f64 = 21.666;

/// χ² of `n` draws from `next` in 10 equal bins of [0, 1).
pub fn chi2_uniform(mut next: impl FnMut() -> f64, n: usize) -> f64 {
    let mut bins = [0.0f64; 10];
    for _ in 0..n {
        let u = next();
        assert!((0.0..1.0).contains(&u), "draw {u} outside [0, 1)");
        bins[((u * 10.0) as usize).min(9)] += 1.0;
    }
    chi_squared(&bins, &[n as f64 / 10.0; 10])
}

/// Every claim this example makes, as a receipt. Each check names the
/// `example-random-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("random");

    // Park & Miller's published check: from seed 1, the 10 000th value is 1043618065.
    let mut lcg = minstd(1);
    let first: Vec<u64> = (0..10).map(|_| lcg.next_val()).collect();
    say(format!("LCG (MINSTD) first 10: {first:?}"));
    let x10000 = (10..10_000).fold(0, |_, _| lcg.next_val());
    say(format!("  x[10000] = {x10000}"));
    r.check("minstd_known_value", x10000 == 1_043_618_065);

    // The reference MT19937 check: from seed 5489, the 10 000th output is 4123659995.
    let mut mt = Mt19937::new(5489);
    let first: Vec<u32> = (0..10).map(|_| mt.next_u32()).collect();
    say(format!("MT19937 (seed 5489) first 10: {first:?}"));
    let y10000 = (10..10_000).fold(0, |_, _| mt.next_u32());
    say(format!("  y[10000] = {y10000}"));
    r.check("mt19937_known_value", y10000 == 4_123_659_995);

    // 10 000 draws in 10 bins pass χ² at p = 0.01.
    let mut lcg = minstd(1);
    let c_lcg = chi2_uniform(|| lcg.next_f64(), 10_000);
    let mut mt = Mt19937::new(42);
    let c_mt = chi2_uniform(|| mt.next_f64(), 10_000);
    say(format!(
        "\nχ² over 10 bins (critical {CHI2_9DF_P01}): LCG {c_lcg:.3}, MT19937 {c_mt:.3}"
    ));
    r.check("uniform_chi2", c_lcg < CHI2_9DF_P01);
    r.check("uniform_chi2", c_mt < CHI2_9DF_P01);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
