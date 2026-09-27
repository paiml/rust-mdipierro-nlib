//! Monte Carlo — Di Pierro, *Annotated Algorithms in Python*, Ch. 7.
//! Contract: `contracts/example-monte-carlo-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example monte_carlo            # human-readable
//! cargo run --example monte_carlo -- --json  # the receipt in evidence/examples/monte_carlo.json
use nlib::monte_carlo::{bootstrap_error, mc_integrate};
use nlib::receipt::{Receipt, json_requested};
use nlib::stats::{mean, std_dev};

/// Standard deviation of f(U) = U² for U ~ Uniform(0, 1): sqrt(1/5 - 1/9).
pub fn sigma_x2() -> f64 {
    (1.0f64 / 5.0 - 1.0 / 9.0).sqrt()
}

/// The sample the bootstrap resamples.
pub const DATA: [f64; 10] = [2.3, 4.1, 3.7, 5.2, 3.9, 4.5, 2.8, 3.3, 4.0, 3.6];

/// Every claim this example makes, as a receipt. Each check names the
/// `example-monte-carlo-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("monte_carlo");

    // ∫₀^1 x² dx = 1/3; each estimate lies within 4 standard errors (σ/√N).
    let exact = 1.0 / 3.0;
    for n in [100, 1_000, 10_000, 100_000] {
        let est = mc_integrate(|x| x * x, 0.0, 1.0, n, 42);
        let err = (est - exact).abs();
        say(format!("MC ∫x²dx (N={n:>7}): {est:.6}  error={err:.6}"));
        r.close(
            "mc_clt_bound",
            est,
            exact,
            4.0 * sigma_x2() / (n as f64).sqrt(),
        );
    }

    // The bootstrap SE of the mean agrees with σ/√n to within 10 %.
    let se = bootstrap_error(&DATA, mean, 10_000, 123);
    let analytic = std_dev(&DATA) / (DATA.len() as f64).sqrt();
    say(format!("\nBootstrap SE of mean({DATA:?}):"));
    say(format!(
        "  mean = {:.4}, SE = {se:.4}, σ/√n = {analytic:.4}",
        mean(&DATA)
    ));
    r.close("mean_known", mean(&DATA), 3.74, 1e-12);
    r.close("bootstrap_se_mean", se, analytic, 0.1 * analytic);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
