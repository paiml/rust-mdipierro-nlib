//! Monte Carlo methods — contract: `monte-carlo-v1.yaml`
//!
//! Di Pierro Ch. 10: MC integration, bootstrap error estimation.
//! Uses internal LCG for reproducible randomness (no external deps).

use crate::random::Lcg;

/// Monte Carlo integration: I ≈ (b-a)/N * sum f(x_i), x_i ~ Uniform(a,b).
pub fn mc_integrate(f: impl Fn(f64) -> f64, a: f64, b: f64, n: usize, rng_seed: u64) -> f64 {
    assert!(n > 0, "mc_integrate: n must be positive");
    assert!(a < b, "mc_integrate: a must be less than b");
    let m = 2_147_483_647u64; // 2^31 - 1
    let mut rng = Lcg::new(rng_seed % (m - 1) + 1, 16807, 0, m);
    let width = b - a;
    let mut sum = 0.0;
    for _ in 0..n {
        let u = rng.next_f64();
        let x = a + u * width;
        sum += f(x);
    }
    width * sum / n as f64
}

/// Bootstrap standard error estimate for a statistic.
///
/// Resamples `data` with replacement `n_resamples` times, computes
/// `statistic_fn` on each resample, returns the standard deviation
/// of the bootstrap distribution.
pub fn bootstrap_error(
    data: &[f64],
    statistic_fn: fn(&[f64]) -> f64,
    n_resamples: usize,
    rng_seed: u64,
) -> f64 {
    assert!(!data.is_empty(), "bootstrap_error: data must be non-empty");
    assert!(n_resamples > 1, "bootstrap_error: need > 1 resamples");
    let m = 2_147_483_647u64;
    let mut rng = Lcg::new(rng_seed % (m - 1) + 1, 16807, 0, m);
    let n = data.len();
    let mut theta_values = Vec::with_capacity(n_resamples);
    let mut resample = vec![0.0; n];
    for _ in 0..n_resamples {
        for slot in resample.iter_mut() {
            let idx = (rng.next() as usize) % n;
            *slot = data[idx];
        }
        theta_values.push(statistic_fn(&resample));
    }
    // Standard deviation of theta_values
    let mean: f64 = theta_values.iter().sum::<f64>() / n_resamples as f64;
    let var: f64 = theta_values.iter().map(|&t| (t - mean).powi(2)).sum::<f64>()
        / (n_resamples - 1) as f64;
    var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean(x: &[f64]) -> f64 {
        x.iter().sum::<f64>() / x.len() as f64
    }

    #[test]
    fn mc_integrate_x_identity() {
        // integral of x on [0,1] = 0.5
        let r = mc_integrate(|x| x, 0.0, 1.0, 100_000, 42);
        assert!((r - 0.5).abs() < 0.02, "MC integral of x ≈ 0.5, got {r}");
    }

    #[test]
    fn mc_integrate_x_squared() {
        // integral of x^2 on [0,1] = 1/3
        let r = mc_integrate(|x| x * x, 0.0, 1.0, 100_000, 42);
        assert!((r - 1.0 / 3.0).abs() < 0.02, "MC integral of x^2 ≈ 1/3, got {r}");
    }

    #[test]
    fn mc_integrate_constant() {
        // integral of 5 on [0, 2] = 10
        let r = mc_integrate(|_| 5.0, 0.0, 2.0, 10_000, 1);
        assert!((r - 10.0).abs() < 0.1);
    }

    #[test]
    #[should_panic]
    fn mc_integrate_n_zero() {
        mc_integrate(|x| x, 0.0, 1.0, 0, 42);
    }

    #[test]
    fn mc_error_decreases_with_n() {
        let exact = 0.5;
        let err_1k = (mc_integrate(|x| x, 0.0, 1.0, 1_000, 42) - exact).abs();
        let err_100k = (mc_integrate(|x| x, 0.0, 1.0, 100_000, 42) - exact).abs();
        // 100x more samples => ~10x less error (O(1/sqrt(N)))
        assert!(err_100k < err_1k, "error should decrease with N");
    }

    #[test]
    fn mc_deterministic() {
        let r1 = mc_integrate(|x| x * x, 0.0, 1.0, 10_000, 42);
        let r2 = mc_integrate(|x| x * x, 0.0, 1.0, 10_000, 42);
        assert!((r1 - r2).abs() < 1e-15, "same seed => same result");
    }

    #[test]
    fn bootstrap_constant_data() {
        // SE of constant data should be ~0
        let data = vec![5.0; 100];
        let se = bootstrap_error(&data, mean, 500, 42);
        assert!(se < 1e-10, "SE of constant data should be near 0, got {se}");
    }

    #[test]
    fn bootstrap_non_negative() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let se = bootstrap_error(&data, mean, 1000, 42);
        assert!(se >= 0.0, "SE must be non-negative");
    }

    #[test]
    fn bootstrap_reasonable_se() {
        // For uniform 1..10, SE of mean should be ~0.9 (σ/√n ≈ 2.87/√10 ≈ 0.91)
        let data: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let se = bootstrap_error(&data, mean, 2000, 42);
        assert!(se > 0.3 && se < 2.0, "reasonable SE range, got {se}");
    }

    #[test]
    #[should_panic]
    fn bootstrap_empty_data() {
        bootstrap_error(&[], mean, 100, 42);
    }
}
