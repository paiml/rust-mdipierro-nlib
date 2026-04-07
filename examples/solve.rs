//! Nonlinear solvers — Di Pierro Ch. 4.6
//! cargo run --example solve
use nlib::solve::{bisection, fixed_point, newton, secant};

fn main() {
    // Find sqrt(2) by solving x^2 - 2 = 0
    let root = bisection(|x| x * x - 2.0, 1.0, 2.0, 1e-12);
    println!("bisection: sqrt(2) ≈ {root:.15}");
    println!("  error: {:.2e}", (root - std::f64::consts::SQRT_2).abs());

    // Newton's method: x^3 - x - 2 = 0, root near x=1.5
    let root = newton(|x| x * x * x - x - 2.0, |x| 3.0 * x * x - 1.0, 1.5, 1e-12);
    println!("\nnewton: x^3-x-2=0 → x ≈ {root:.15}");
    println!("  f(root) = {:.2e}", root * root * root - root - 2.0);

    // Secant method
    let root = secant(|x| x * x - 2.0, 1.0, 2.0, 1e-12);
    println!("\nsecant: sqrt(2) ≈ {root:.15}");

    // Fixed point: cos(x) = x
    let fp = fixed_point(|x| x.cos(), 1.0, 1e-12);
    println!("\nfixed_point: cos(x)=x → x ≈ {fp:.15}");
    println!("  |cos(x)-x| = {:.2e}", (fp.cos() - fp).abs());
}
