//! Optimization — Di Pierro, *Annotated Algorithms in Python*, §4.7–4.8.
//! Contract: `contracts/example-optimize-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example optimize            # human-readable
//! cargo run --example optimize -- --json  # the receipt in evidence/examples/optimize.json
use nlib::optimize::{golden_section, gradient_descent, newton_optimize};
use nlib::receipt::{Receipt, json_requested};

/// f(x) = x⁴ - 3x² + 2 and its first two derivatives.
pub fn quartic(x: f64) -> f64 {
    x.powi(4) - 3.0 * x * x + 2.0
}
pub fn quartic_d1(x: f64) -> f64 {
    4.0 * x * x * x - 6.0 * x
}
pub fn quartic_d2(x: f64) -> f64 {
    12.0 * x * x - 6.0
}

/// Every claim this example makes, as a receipt. Each check names the
/// `example-optimize-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("optimize");

    // (x-3)² on [0, 10] has its minimum at x = 3.
    let x = golden_section(|x| (x - 3.0).powi(2), 0.0, 10.0, 1e-10);
    say(format!("golden_section: min of (x-3)^2 at x = {x:.10}"));
    r.close("golden_section_min", x, 3.0, 1e-9);

    // From x0 = 1, Newton reaches the local minimum x = sqrt(3/2) of the quartic.
    // (From x0 = 0.5 it converges to the local maximum x = 0; FALSIFY-EXOPT-004.)
    let x = newton_optimize(quartic, quartic_d1, quartic_d2, 1.0, 1e-10);
    say(format!("\nnewton_optimize: critical point at x = {x:.10}"));
    say(format!(
        "  f'(x) = {:.2e}, f''(x) = {:.2}",
        quartic_d1(x),
        quartic_d2(x)
    ));
    r.close("newton_critical_point", quartic_d1(x), 0.0, 1e-10);
    r.close("newton_critical_point", x, 1.5f64.sqrt(), 1e-10);
    r.check("newton_is_minimum", quartic_d2(x) > 0.0);

    // x² + y² from (5, 3): the minimum is the origin (steps run in f32).
    let p = gradient_descent(
        |v| v[0] * v[0] + v[1] * v[1],
        |v| vec![2.0 * v[0], 2.0 * v[1]],
        &[5.0, 3.0],
        0.1,
        1e-6,
    );
    say(format!(
        "\ngradient_descent: min of x^2+y^2 at ({:.6}, {:.6})",
        p[0], p[1]
    ));
    r.close("gradient_descent_min", p[0].hypot(p[1]), 0.0, 1e-6);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
