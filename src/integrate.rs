//! Numerical integration — contract: `integration-v1.yaml`
//!
//! Di Pierro Ch. 7: trapezoid, Simpson, adaptive quadrature.
//! Uses `aprender::Vector<f32>` for storing quadrature node values.

use aprender::Vector as AprVector;

/// Composite trapezoid rule: I = h/2 * [f(a) + 2*sum + f(b)]
pub fn trapezoid(f: impl Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64 {
    assert!(a < b, "trapezoid: a must be less than b");
    assert!(n > 0, "trapezoid: n must be positive");
    let h = (b - a) / n as f64;
    // Store quadrature nodes in aprender Vector for diagnostics
    let nodes: Vec<f32> = (0..=n).map(|i| (a + i as f64 * h) as f32).collect();
    let _apr_nodes = AprVector::from_vec(nodes);
    let mut sum = (f(a) + f(b)) / 2.0;
    for i in 1..n {
        sum += f(a + i as f64 * h);
    }
    sum * h
}

/// Composite Simpson's 1/3 rule. Requires n even.
pub fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64 {
    assert!(a < b, "simpson: a must be less than b");
    assert!(n > 0, "simpson: n must be positive");
    assert!(n % 2 == 0, "simpson: n must be even");
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        if i % 2 == 0 {
            sum += 2.0 * f(x);
        } else {
            sum += 4.0 * f(x);
        }
    }
    sum * h / 3.0
}

/// Adaptive Simpson quadrature with Richardson extrapolation.
pub fn adaptive_quadrature(f: impl Fn(f64) -> f64, a: f64, b: f64, tol: f64) -> f64 {
    assert!(a < b, "adaptive_quadrature: a must be less than b");
    assert!(tol > 0.0, "adaptive_quadrature: tolerance must be positive");
    adaptive_rec(&f, a, b, tol, 20)
}

fn adaptive_rec(f: &impl Fn(f64) -> f64, a: f64, b: f64, tol: f64, depth: usize) -> f64 {
    let mid = (a + b) / 2.0;
    let whole = simpson_raw(f, a, b);
    let left = simpson_raw(f, a, mid);
    let right = simpson_raw(f, mid, b);
    let refined = left + right;
    if depth == 0 || (refined - whole).abs() < 15.0 * tol {
        return refined + (refined - whole) / 15.0;
    }
    adaptive_rec(f, a, mid, tol / 2.0, depth - 1)
        + adaptive_rec(f, mid, b, tol / 2.0, depth - 1)
}

/// Raw Simpson on 2 panels (3 points).
fn simpson_raw(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let mid = (a + b) / 2.0;
    let h = (b - a) / 6.0;
    h * (f(a) + 4.0 * f(mid) + f(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trapezoid_linear() {
        // Exact for f(x) = 3x+1 on [0,1], integral = 2.5
        let r = trapezoid(|x| 3.0 * x + 1.0, 0.0, 1.0, 1);
        assert!((r - 2.5).abs() < 1e-14, "trapezoid exact for linear");
    }

    #[test]
    fn trapezoid_x_squared() {
        let r = trapezoid(|x| x * x, 0.0, 1.0, 1000);
        assert!((r - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn trapezoid_sin() {
        let r = trapezoid(f64::sin, 0.0, std::f64::consts::PI, 1000);
        assert!((r - 2.0).abs() < 1e-5, "integral of sin from 0 to pi = 2");
    }

    #[test]
    #[should_panic]
    fn trapezoid_a_ge_b() {
        trapezoid(|x| x, 1.0, 0.0, 10);
    }

    #[test]
    fn simpson_x_cubed() {
        // Simpson exact for cubics. integral of x^3 on [0,1] = 0.25
        let r = simpson(|x| x.powi(3), 0.0, 1.0, 2);
        assert!((r - 0.25).abs() < 1e-14, "Simpson exact for cubic");
    }

    #[test]
    fn simpson_sin() {
        let r = simpson(f64::sin, 0.0, std::f64::consts::PI, 100);
        assert!((r - 2.0).abs() < 1e-7, "integral of sin = 2");
    }

    #[test]
    #[should_panic]
    fn simpson_odd_n() {
        simpson(|x| x * x, 0.0, 1.0, 3);
    }

    #[test]
    fn adaptive_sin() {
        let r = adaptive_quadrature(f64::sin, 0.0, std::f64::consts::PI, 1e-10);
        assert!((r - 2.0).abs() < 1e-8, "adaptive integral of sin = 2");
    }

    #[test]
    fn adaptive_x_squared() {
        let r = adaptive_quadrature(|x| x * x, 0.0, 1.0, 1e-10);
        assert!((r - 1.0 / 3.0).abs() < 1e-8);
    }

    #[test]
    fn adaptive_near_singularity() {
        // 1/sqrt(x) on [0.001, 1] = 2*sqrt(1) - 2*sqrt(0.001) ~ 1.93675
        let r = adaptive_quadrature(|x| 1.0 / x.sqrt(), 0.001, 1.0, 1e-6);
        let exact = 2.0 * 1.0_f64.sqrt() - 2.0 * 0.001_f64.sqrt();
        assert!((r - exact).abs() < 1e-4);
    }

    #[test]
    fn trapezoid_convergence_order() {
        // Error should decrease by ~4x when n doubles (O(h^2))
        let exact = 1.0 / 3.0;
        let e1 = (trapezoid(|x| x * x, 0.0, 1.0, 100) - exact).abs();
        let e2 = (trapezoid(|x| x * x, 0.0, 1.0, 200) - exact).abs();
        let ratio = e1 / e2;
        assert!(ratio > 3.0 && ratio < 5.0, "trapezoid O(h^2): ratio={ratio}");
    }

    #[test]
    fn simpson_more_accurate_than_trapezoid() {
        let exact = 1.0 / 3.0;
        let e_trap = (trapezoid(|x| x * x, 0.0, 1.0, 10) - exact).abs();
        let e_simp = (simpson(|x| x * x, 0.0, 1.0, 10) - exact).abs();
        assert!(e_simp < e_trap, "Simpson should be more accurate");
    }
}
