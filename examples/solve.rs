//! Nonlinear solvers — Di Pierro, *Annotated Algorithms in Python*, §4.6.
//! Contract: `contracts/example-solve-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example solve            # human-readable
//! cargo run --example solve -- --json  # the receipt in evidence/examples/solve.json
use nlib::receipt::{Receipt, json_requested};
use nlib::solve::{bisection, fixed_point, newton, secant};
use std::f64::consts::SQRT_2;

/// The root of cos(x) = x (the Dottie number), to 16 digits.
pub const DOTTIE: f64 = 0.739_085_133_215_160_6;

/// Every claim this example makes, as a receipt. Each check names the
/// `example-solve-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("solve");

    // sqrt(2) as the root of x^2 - 2 on [1, 2].
    let b = bisection(|x| x * x - 2.0, 1.0, 2.0, 1e-12);
    say(format!("bisection: sqrt(2) ≈ {b:.15}"));
    say(format!("  error: {:.2e}", (b - SQRT_2).abs()));
    r.close("bisection_sqrt2", b, SQRT_2, 1e-12);

    // x^3 - x - 2 = 0 from x0 = 1.5: the residual is below the tolerance.
    let f = |x: f64| x * x * x - x - 2.0;
    let n = newton(f, |x| 3.0 * x * x - 1.0, 1.5, 1e-12);
    say(format!("\nnewton: x^3-x-2=0 → x ≈ {n:.15}"));
    say(format!("  f(root) = {:.2e}", f(n)));
    r.close("newton_cubic_residual", f(n), 0.0, 1e-12);

    let s = secant(|x| x * x - 2.0, 1.0, 2.0, 1e-12);
    say(format!("\nsecant: sqrt(2) ≈ {s:.15}"));
    r.close("secant_sqrt2", s, SQRT_2, 1e-12);
    r.close("solvers_agree", b, s, 1e-12);

    // cos(x) = x
    let fp = fixed_point(f64::cos, 1.0, 1e-12);
    say(format!("\nfixed_point: cos(x)=x → x ≈ {fp:.15}"));
    say(format!("  |cos(x)-x| = {:.2e}", (fp.cos() - fp).abs()));
    r.close("fixed_point_dottie", fp, DOTTIE, 1e-11);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
