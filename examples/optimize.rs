//! Optimization — Di Pierro Ch. 4.7-4.8
//! cargo run --example optimize
use nlib::optimize::{golden_section, newton_optimize, gradient_descent};

fn main() {
    // Minimize (x-3)^2 on [0, 10]
    let x = golden_section(|x| (x - 3.0).powi(2), 0.0, 10.0, 1e-10);
    println!("golden_section: min of (x-3)^2 at x = {x:.10}");
    println!("  f(x) = {:.2e}", (x - 3.0).powi(2));

    // Newton: minimize x^4 - 3x^2 + 2 near x=1
    let x = newton_optimize(
        |x| x.powi(4) - 3.0*x*x + 2.0,  // f
        |x| 4.0*x*x*x - 6.0*x,          // f'
        |x| 12.0*x*x - 6.0,             // f''
        0.5, 1e-10
    );
    println!("\nnewton_optimize: critical point at x = {x:.10}");
    println!("  f'(x) = {:.2e}", 4.0*x*x*x - 6.0*x);

    // Gradient descent: minimize f(x,y) = x^2 + y^2
    let result = gradient_descent(
        |x| x[0]*x[0] + x[1]*x[1],
        |x| vec![2.0 * x[0], 2.0 * x[1]],
        &[5.0, 3.0],
        0.1,
        1e-6,
    );
    println!("\ngradient_descent: min of x^2+y^2 at ({:.6}, {:.6})", result[0], result[1]);
    println!("  f(x,y) = {:.2e}", result[0]*result[0] + result[1]*result[1]);
}
