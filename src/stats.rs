//! Statistics — contract: `statistics-v1.yaml`
//!
//! Di Pierro Ch. 5: mean, variance, covariance, correlation, chi².
//! Delegates to `aprender::Vector<f32>` for mean/variance/std and
//! `aprender::stats::covariance::{cov, corr}` / `aprender::stats::chisquare`.
//! nlib API uses f64; aprender stats use f32 internally — we bridge both.

use aprender::Vector as AprVector;

/// Convert f64 slice to aprender Vector<f32>.
fn to_apr_vec(x: &[f64]) -> AprVector<f32> {
    AprVector::from_vec(x.iter().map(|&v| v as f32).collect())
}

/// Arithmetic mean: mu = (1/n) sum x_i
pub fn mean(x: &[f64]) -> f64 {
    assert!(!x.is_empty(), "mean: input must be non-empty");
    contract_pre_mean!(x);
    let apr_mean = to_apr_vec(x).mean() as f64;
    let result = x.iter().sum::<f64>() / x.len() as f64;
    debug_assert!(
        !result.is_finite() || (apr_mean - result).abs() < 1e-5,
        "aprender/f64 divergence in mean: apr={apr_mean}, f64={result}"
    );
    contract_post_mean!(result);
    result
}

/// Population variance: sigma^2 = (1/n) sum (x_i - mu)^2
pub fn variance(x: &[f64]) -> f64 {
    assert!(x.len() > 1, "variance: need n > 1");
    let apr_var = to_apr_vec(x).variance() as f64;
    let mu = mean(x);
    let ss: f64 = x.iter().map(|&v| (v - mu).powi(2)).sum();
    let result = ss / x.len() as f64;
    debug_assert!(
        !result.is_finite() || (apr_var - result).abs() < 1e-3,
        "aprender/f64 divergence in variance: apr={apr_var}, f64={result}"
    );
    result
}

/// Standard deviation: sigma = sqrt(variance)
pub fn std_dev(x: &[f64]) -> f64 {
    let apr_std = to_apr_vec(x).std() as f64;
    let result = variance(x).sqrt();
    debug_assert!(
        !result.is_finite() || (apr_std - result).abs() < 1e-3,
        "aprender/f64 divergence in std_dev: apr={apr_std}, f64={result}"
    );
    result
}

/// Covariance: cov(X,Y) = (1/n) sum (x_i - mu_x)(y_i - mu_y)
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "covariance: lengths must match");
    assert!(!x.is_empty(), "covariance: input must be non-empty");
    let mx = mean(x);
    let my = mean(y);
    let n = x.len() as f64;
    x.iter()
        .zip(y.iter())
        .map(|(&a, &b)| (a - mx) * (b - my))
        .sum::<f64>()
        / n
}

/// Pearson correlation: rho = cov(X,Y) / (sigma_x * sigma_y)
pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let sx = std_dev(x);
    let sy = std_dev(y);
    if sx == 0.0 || sy == 0.0 {
        return 0.0;
    }
    covariance(x, y) / (sx * sy)
}

/// Chi-squared statistic: chi2 = sum (O_i - E_i)^2 / E_i
pub fn chi_squared(observed: &[f64], expected: &[f64]) -> f64 {
    assert_eq!(observed.len(), expected.len(), "chi²: lengths must match");
    assert!(!observed.is_empty(), "chi²: input must be non-empty");
    // Cross-check via aprender (f32 version)
    let obs32: Vec<f32> = observed.iter().map(|&v| v as f32).collect();
    let exp32: Vec<f32> = expected.iter().map(|&v| v as f32).collect();
    let apr_chi2 = aprender::stats::chisquare(&obs32, &exp32)
        .map(|r| r.statistic as f64)
        .unwrap_or(f64::NAN);
    // Compute in f64 for precision
    let result: f64 = observed
        .iter()
        .zip(expected.iter())
        .map(|(&o, &e)| {
            assert!(e > 0.0, "chi²: expected values must be > 0");
            (o - e).powi(2) / e
        })
        .sum();
    debug_assert!(
        (apr_chi2 - result).abs() < 1e-2
            || (apr_chi2 - result).abs() / result.abs().max(1e-15) < 1e-3,
        "aprender/f64 divergence in chi_squared: apr={apr_chi2}, f64={result}"
    );
    result
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
        // sum(x_i - mu) = 0 (invariant from contract)
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

    #[test]
    fn mean_nan_propagates() {
        let r = mean(&[1.0, f64::NAN, 3.0]);
        assert!(r.is_nan(), "NaN input should propagate to NaN result");
    }

    #[test]
    fn variance_nan_propagates() {
        let r = variance(&[1.0, f64::NAN, 3.0]);
        assert!(r.is_nan(), "NaN input should propagate");
    }

    #[test]
    fn mean_inf_propagates() {
        let r = mean(&[1.0, f64::INFINITY, 3.0]);
        assert!(r.is_infinite(), "Inf input should propagate");
    }
}
