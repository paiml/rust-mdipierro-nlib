//! Statistics — contract: `statistics-v1.yaml`
//!
//! Di Pierro Ch. 5: mean, variance, covariance, correlation, chi².
//! Postconditions: variance >= 0, |correlation| <= 1.

/// Arithmetic mean: μ = (1/n) Σ x_i
pub fn mean(x: &[f64]) -> f64 {
    assert!(!x.is_empty(), "mean: input must be non-empty");
    x.iter().sum::<f64>() / x.len() as f64
}

/// Population variance: σ² = (1/n) Σ (x_i - μ)²
pub fn variance(x: &[f64]) -> f64 {
    assert!(x.len() > 1, "variance: need n > 1");
    let mu = mean(x);
    let ss: f64 = x.iter().map(|&v| (v - mu).powi(2)).sum();
    ss / x.len() as f64
}

/// Standard deviation: σ = √variance
pub fn std_dev(x: &[f64]) -> f64 {
    variance(x).sqrt()
}

/// Covariance: cov(X,Y) = (1/n) Σ (x_i - μ_x)(y_i - μ_y)
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "covariance: lengths must match");
    assert!(!x.is_empty(), "covariance: input must be non-empty");
    let mx = mean(x);
    let my = mean(y);
    let n = x.len() as f64;
    x.iter().zip(y.iter()).map(|(&a, &b)| (a - mx) * (b - my)).sum::<f64>() / n
}

/// Pearson correlation: ρ = cov(X,Y) / (σ_x * σ_y)
pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let sx = std_dev(x);
    let sy = std_dev(y);
    if sx == 0.0 || sy == 0.0 {
        return 0.0;
    }
    covariance(x, y) / (sx * sy)
}

/// Chi-squared statistic: χ² = Σ (O_i - E_i)² / E_i
pub fn chi_squared(observed: &[f64], expected: &[f64]) -> f64 {
    assert_eq!(observed.len(), expected.len(), "chi²: lengths must match");
    assert!(!observed.is_empty(), "chi²: input must be non-empty");
    observed.iter().zip(expected.iter())
        .map(|(&o, &e)| {
            assert!(e > 0.0, "chi²: expected values must be > 0");
            (o - e).powi(2) / e
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_basic() {
        assert!((mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn mean_single() {
        assert!((mean(&[42.0]) - 42.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic]
    fn mean_empty() {
        mean(&[]);
    }

    #[test]
    fn variance_basic() {
        let v = variance(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!(v >= 0.0, "variance must be non-negative");
        assert!((v - 4.0).abs() < 1e-10);
    }

    #[test]
    fn variance_constant() {
        let v = variance(&[5.0, 5.0, 5.0, 5.0]);
        assert!((v - 0.0).abs() < 1e-10, "constant data has zero variance");
    }

    #[test]
    fn covariance_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!(covariance(&x, &y) > 0.0);
    }

    #[test]
    fn covariance_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        assert!(covariance(&x, &y) < 0.0);
    }

    #[test]
    fn correlation_bounds() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = correlation(&x, &y);
        assert!((r - 1.0).abs() < 1e-10, "perfect positive correlation");
    }

    #[test]
    fn correlation_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let r = correlation(&x, &y);
        assert!((r - (-1.0)).abs() < 1e-10, "perfect negative correlation");
    }

    #[test]
    fn correlation_bounded() {
        let x = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let y = vec![2.0, 1.0, 5.0, 3.0, 4.0];
        let r = correlation(&x, &y);
        assert!(r.abs() <= 1.0, "|correlation| must be <= 1");
    }

    #[test]
    fn chi_squared_perfect_fit() {
        let obs = vec![10.0, 20.0, 30.0];
        let exp = vec![10.0, 20.0, 30.0];
        assert!((chi_squared(&obs, &exp) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn chi_squared_non_negative() {
        let obs = vec![12.0, 18.0, 30.0];
        let exp = vec![10.0, 20.0, 30.0];
        assert!(chi_squared(&obs, &exp) >= 0.0);
    }

    #[test]
    fn sum_deviations_zero() {
        // Σ(x_i - μ) = 0 (invariant from contract)
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mu = mean(&x);
        let sum_dev: f64 = x.iter().map(|&v| v - mu).sum();
        assert!(sum_dev.abs() < 1e-10, "sum of deviations must be zero");
    }

    #[test]
    fn correlation_constant_is_zero() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        assert!((correlation(&x, &y) - 0.0).abs() < 1e-10);
    }
}
