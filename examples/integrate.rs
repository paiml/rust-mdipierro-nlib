//! Numerical integration — Di Pierro, *Annotated Algorithms in Python*, §4.10.
//! Contract: `contracts/example-integrate-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example integrate            # human-readable
//! cargo run --example integrate -- --json  # the receipt in evidence/examples/integrate.json
use nlib::integrate::{adaptive_quadrature, simpson, trapezoid};
use nlib::receipt::{Receipt, json_requested};
use std::f64::consts::{E, PI};

/// Every claim this example makes, as a receipt. Each check names the
/// `example-integrate-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("integrate");

    // ∫₀^π sin(x) dx = 2
    let t = trapezoid(f64::sin, 0.0, PI, 100);
    let s = simpson(f64::sin, 0.0, PI, 100);
    let a = adaptive_quadrature(f64::sin, 0.0, PI, 1e-10);
    say("∫₀^π sin(x) dx = 2".into());
    say(format!(
        "  trapezoid(n=100):    {t:.15}  error={:.2e}",
        (t - 2.0).abs()
    ));
    say(format!(
        "  simpson(n=100):      {s:.15}  error={:.2e}",
        (s - 2.0).abs()
    ));
    say(format!(
        "  adaptive(tol=1e-10): {a:.15}  error={:.2e}",
        (a - 2.0).abs()
    ));
    // Composite trapezoid error is (b-a)h²/12·|f''| ≤ π³/(12·100²) ≈ 2.6e-4.
    r.close("trapezoid_sin_0_pi", t, 2.0, 2.6e-4);
    // Composite Simpson error is (b-a)h⁴/180·|f⁗| ≤ π⁵/(180·100⁴) ≈ 1.7e-8.
    r.close("simpson_sin_0_pi", s, 2.0, 1.7e-8);
    r.close("adaptive_sin_0_pi", a, 2.0, 1e-10);
    // Simpson is the more accurate rule at the same n.
    r.check("simpson_beats_trapezoid", (s - 2.0).abs() < (t - 2.0).abs());

    // ∫₀^1 x² dx = 1/3 — Simpson is exact for polynomials up to degree 3.
    let q = simpson(|x| x * x, 0.0, 1.0, 50);
    say(format!(
        "\n∫₀^1 x² dx = 1/3\n  simpson(n=50): {q:.15}  error={:.2e}",
        (q - 1.0 / 3.0).abs()
    ));
    r.close("simpson_exact_cubic", q, 1.0 / 3.0, 1e-14);
    // ∫₀^1 x³ dx = 1/4 on a single Simpson panel pair.
    let c = simpson(|x| x * x * x, 0.0, 1.0, 2);
    r.close("simpson_exact_cubic", c, 0.25, 1e-15);

    // ∫₀^1 e^x dx = e - 1
    let e = adaptive_quadrature(f64::exp, 0.0, 1.0, 1e-12);
    say(format!(
        "\n∫₀^1 e^x dx = e-1\n  adaptive: {e:.15}  error={:.2e}",
        (e - (E - 1.0)).abs()
    ));
    r.close("adaptive_exp_0_1", e, E - 1.0, 1e-12);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
