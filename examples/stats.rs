//! Statistics — Di Pierro, *Annotated Algorithms in Python*, Ch. 5.
//! Contract: `contracts/example-stats-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example stats            # human-readable
//! cargo run --example stats -- --json  # the receipt in evidence/examples/stats.json
use nlib::receipt::{Receipt, json_requested};
use nlib::stats::{chi_squared, correlation, covariance, mean, std_dev, variance};

/// Every claim this example makes, as a receipt. Each check names the
/// `example-stats-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("stats");

    // Σx = 40 over n = 8; Σ(x-5)² = 32, so the population variance is 4.
    let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let (mu, var, sd) = (mean(&x), variance(&x), std_dev(&x));
    say(format!("Data: {x:?}"));
    say(format!("  mean     = {mu:.4}"));
    say(format!("  variance = {var:.4}"));
    say(format!("  std_dev  = {sd:.4}"));
    r.close("mean_known", mu, 5.0, 1e-12);
    r.close("variance_population", var, 4.0, 1e-12);
    r.close("std_dev_known", sd, 2.0, 1e-12);

    // b = 2a: cov(a, b) = 2 var(a) = 4 and corr(a, b) = 1.
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [2.0, 4.0, 6.0, 8.0, 10.0];
    let (cov, corr) = (covariance(&a, &b), correlation(&a, &b));
    say(format!("\nCorrelation({a:?}, {b:?}):"));
    say(format!("  cov  = {cov:.4}"));
    say(format!("  corr = {corr:.4} (perfect positive)"));
    r.close("covariance_known", cov, 4.0, 1e-12);
    r.close("correlation_perfect", corr, 1.0, 1e-12);

    // Σ(O-E)²/E = 0 + 4/16 + 0 + 4/16 + 16/16 + 16/8 = 3.5
    let obs = [16.0, 18.0, 16.0, 14.0, 12.0, 12.0];
    let exp = [16.0, 16.0, 16.0, 16.0, 16.0, 8.0];
    let chi2 = chi_squared(&obs, &exp);
    say(format!("\nChi-squared: {chi2:.4}"));
    r.close("chi_squared_known", chi2, 3.5, 1e-12);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
