//! Optimization — contract: `optimization-v1.yaml`
//!
//! Di Pierro Ch. 6: golden-section search, Newton for minimization,
//! gradient descent (multi-dimensional).

const MAX_ITER: usize = 10_000;

/// Golden-section search for minimum of unimodal f on [a,b].
pub fn golden_section(f: impl Fn(f64) -> f64, mut a: f64, mut b: f64, tol: f64) -> f64 {
    assert!(a < b, "golden_section: a must be less than b");
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let rho = 2.0 - phi; // ~0.618
    let mut x1 = a + rho * (b - a);
    let mut x2 = b - rho * (b - a);
    let mut f1 = f(x1);
    let mut f2 = f(x2);
    for _ in 0..MAX_ITER {
        if (b - a).abs() < tol {
            break;
        }
        if f1 < f2 {
            b = x2;
            x2 = x1;
            f2 = f1;
            x1 = a + rho * (b - a);
            f1 = f(x1);
        } else {
            a = x1;
            x1 = x2;
            f1 = f2;
            x2 = b - rho * (b - a);
            f2 = f(x2);
        }
    }
    (a + b) / 2.0
}

/// Newton's method for minimization: x_{n+1} = x_n - f'(x_n)/f''(x_n).
pub fn newton_optimize(
    _f: impl Fn(f64) -> f64,
    df: impl Fn(f64) -> f64,
    ddf: impl Fn(f64) -> f64,
    mut x: f64,
    tol: f64,
) -> f64 {
    for _ in 0..MAX_ITER {
        let g = df(x);
        if g.abs() < tol {
            return x;
        }
        let h = ddf(x);
        if h.abs() < 1e-15 {
            // Second derivative zero; cannot proceed with Newton step
            return x;
        }
        x -= g / h;
    }
    x
}

/// Gradient descent for multi-dimensional minimization.
/// `grad` returns the gradient vector at point x.
pub fn gradient_descent(
    _f: impl Fn(&[f64]) -> f64,
    grad: impl Fn(&[f64]) -> Vec<f64>,
    x0: &[f64],
    lr: f64,
    tol: f64,
) -> Vec<f64> {
    assert!(lr > 0.0, "gradient_descent: learning rate must be positive");
    let mut x = x0.to_vec();
    for _ in 0..MAX_ITER {
        let g = grad(&x);
        let gnorm: f64 = g.iter().map(|v| v * v).sum::<f64>().sqrt();
        if gnorm < tol {
            break;
        }
        for (xi, gi) in x.iter_mut().zip(g.iter()) {
            *xi -= lr * gi;
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_section_x_squared() {
        let min = golden_section(|x| x * x, -1.0, 1.0, 1e-10);
        assert!(min.abs() < 1e-8, "minimum of x^2 is at 0");
    }

    #[test]
    fn golden_section_shifted() {
        let min = golden_section(|x| (x - 3.0).powi(2), 0.0, 10.0, 1e-10);
        assert!((min - 3.0).abs() < 1e-8, "minimum of (x-3)^2 is at 3");
    }

    #[test]
    #[should_panic]
    fn golden_section_a_ge_b() {
        golden_section(|x| x * x, 1.0, -1.0, 1e-10);
    }

    #[test]
    fn newton_opt_x_squared() {
        let min = newton_optimize(
            |x| x * x,
            |x| 2.0 * x,
            |_x| 2.0,
            5.0,
            1e-12,
        );
        assert!(min.abs() < 1e-10);
    }

    #[test]
    fn newton_opt_shifted() {
        let min = newton_optimize(
            |x| (x - 3.0).powi(2),
            |x| 2.0 * (x - 3.0),
            |_x| 2.0,
            0.0,
            1e-12,
        );
        assert!((min - 3.0).abs() < 1e-10);
    }

    #[test]
    fn newton_opt_zero_second_deriv() {
        // f''(0) = 0 for f(x) = x^3 at x=0
        let result = newton_optimize(
            |x| x * x * x,
            |x| 3.0 * x * x,
            |x| 6.0 * x,
            0.0,
            1e-12,
        );
        // Should return early without crashing
        assert!(result.is_finite());
    }

    #[test]
    fn gd_1d() {
        let min = gradient_descent(
            |x| x[0] * x[0],
            |x| vec![2.0 * x[0]],
            &[5.0],
            0.1,
            1e-8,
        );
        assert!(min[0].abs() < 1e-6);
    }

    #[test]
    fn gd_shifted() {
        let min = gradient_descent(
            |x| (x[0] - 3.0).powi(2),
            |x| vec![2.0 * (x[0] - 3.0)],
            &[0.0],
            0.1,
            1e-8,
        );
        assert!((min[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn gd_rosenbrock_2d() {
        // Rosenbrock: f(x,y) = (1-x)^2 + 100*(y-x^2)^2, min at (1,1)
        let min = gradient_descent(
            |x| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2),
            |x| {
                let dx = -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]);
                let dy = 200.0 * (x[1] - x[0] * x[0]);
                vec![dx, dy]
            },
            &[0.0, 0.0],
            0.001,
            1e-8,
        );
        assert!((min[0] - 1.0).abs() < 0.1, "x near 1: got {}", min[0]);
        assert!((min[1] - 1.0).abs() < 0.1, "y near 1: got {}", min[1]);
    }

    #[test]
    #[should_panic]
    fn gd_negative_lr() {
        gradient_descent(|x| x[0] * x[0], |x| vec![2.0 * x[0]], &[1.0], -0.1, 1e-8);
    }
}
