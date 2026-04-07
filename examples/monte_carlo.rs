//! Monte Carlo simulation — Di Pierro Ch. 7
//! cargo run --example monte_carlo
use nlib::monte_carlo::{bootstrap_error, mc_integrate};
use nlib::stats::mean;

fn main() {
    // MC estimate of ∫₀^1 x² dx = 1/3
    let exact = 1.0 / 3.0;
    for &n in &[100, 1_000, 10_000, 100_000] {
        let estimate = mc_integrate(|x| x * x, 0.0, 1.0, n, 42);
        let error = (estimate - exact).abs();
        println!("MC ∫x²dx (N={n:>7}): {estimate:.6}  error={error:.6}");
    }

    // Bootstrap error estimation
    let data = vec![2.3, 4.1, 3.7, 5.2, 3.9, 4.5, 2.8, 3.3, 4.0, 3.6];
    let se = bootstrap_error(&data, |s| mean(s), 10_000, 123);
    println!("\nBootstrap SE of mean({data:?}):");
    println!("  mean = {:.4}, SE = {:.4}", mean(&data), se);
}
