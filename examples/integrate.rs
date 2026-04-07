//! Numerical integration — Di Pierro Ch. 4.10
//! cargo run --example integrate
use nlib::integrate::{adaptive_quadrature, simpson, trapezoid};

fn main() {
    let pi = std::f64::consts::PI;

    // ∫₀^π sin(x) dx = 2
    let t = trapezoid(f64::sin, 0.0, pi, 100);
    let s = simpson(f64::sin, 0.0, pi, 100);
    let a = adaptive_quadrature(f64::sin, 0.0, pi, 1e-10);
    println!("∫₀^π sin(x) dx = 2.0");
    println!("  trapezoid(n=100): {t:.15}  error={:.2e}", (t - 2.0).abs());
    println!("  simpson(n=100):   {s:.15}  error={:.2e}", (s - 2.0).abs());
    println!(
        "  adaptive(tol=1e-10): {a:.15}  error={:.2e}",
        (a - 2.0).abs()
    );

    // ∫₀^1 x² dx = 1/3
    let exact = 1.0 / 3.0;
    let result = simpson(|x| x * x, 0.0, 1.0, 50);
    println!("\n∫₀^1 x² dx = 1/3");
    println!(
        "  simpson(n=50): {result:.15}  error={:.2e}",
        (result - exact).abs()
    );

    // ∫₀^1 e^x dx = e - 1
    let exact = std::f64::consts::E - 1.0;
    let result = adaptive_quadrature(f64::exp, 0.0, 1.0, 1e-12);
    println!("\n∫₀^1 e^x dx = e-1");
    println!(
        "  adaptive: {result:.15}  error={:.2e}",
        (result - exact).abs()
    );
}
