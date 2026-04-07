//! Fourier transforms — contract: `fourier-transform-v1.yaml`
//!
//! Di Pierro Ch. 8: DFT, FFT (Cooley-Tukey radix-2), inverse DFT.
//! Complex numbers represented as (re, im) f64 pairs.
//! Uses `aprender::Matrix<f64>` to represent DFT matrix for validation.

use aprender::Matrix as AprMatrix;
use std::f64::consts::PI;

/// Discrete Fourier Transform (O(N^2)).
/// X_k = sum_{n=0}^{N-1} x_n * exp(-2*pi*i*k*n/N)
pub fn dft(x: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = x.len();
    assert!(n > 0, "dft: input must be non-empty");
    let mut result = Vec::with_capacity(n);
    for k in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for (idx, &(xr, xi)) in x.iter().enumerate() {
            let angle = -2.0 * PI * (k as f64) * (idx as f64) / (n as f64);
            let (sin_a, cos_a) = angle.sin_cos();
            // (xr + i*xi) * (cos + i*sin) = (xr*cos - xi*sin) + i*(xr*sin + xi*cos)
            re += xr * cos_a - xi * sin_a;
            im += xr * sin_a + xi * cos_a;
        }
        result.push((re, im));
    }
    result
}

/// Build the DFT matrix W where W[k,n] = exp(-2*pi*i*k*n/N).
/// Returns (real_part, imag_part) as two aprender matrices.
pub fn dft_matrix(n: usize) -> (AprMatrix<f64>, AprMatrix<f64>) {
    let mut re_data = vec![0.0; n * n];
    let mut im_data = vec![0.0; n * n];
    for k in 0..n {
        for j in 0..n {
            let angle = -2.0 * PI * (k as f64) * (j as f64) / (n as f64);
            let (sin_a, cos_a) = angle.sin_cos();
            re_data[k * n + j] = cos_a;
            im_data[k * n + j] = sin_a;
        }
    }
    (
        AprMatrix::from_vec(n, n, re_data).expect("valid"),
        AprMatrix::from_vec(n, n, im_data).expect("valid"),
    )
}

/// Cooley-Tukey radix-2 FFT (O(N log N)).
/// Input length must be a power of 2.
pub fn fft(x: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = x.len();
    assert!(n > 0, "fft: input must be non-empty");
    assert!(n & (n - 1) == 0, "fft: input length must be a power of 2");
    fft_recursive(x)
}

fn fft_recursive(x: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = x.len();
    if n == 1 {
        return vec![x[0]];
    }
    let even: Vec<(f64, f64)> = x.iter().step_by(2).copied().collect();
    let odd: Vec<(f64, f64)> = x.iter().skip(1).step_by(2).copied().collect();
    let e = fft_recursive(&even);
    let o = fft_recursive(&odd);
    let mut result = vec![(0.0, 0.0); n];
    for k in 0..n / 2 {
        let angle = -2.0 * PI * (k as f64) / (n as f64);
        let (sin_a, cos_a) = angle.sin_cos();
        // twiddle * O[k]
        let (or, oi) = o[k];
        let tr = or * cos_a - oi * sin_a;
        let ti = or * sin_a + oi * cos_a;
        result[k] = (e[k].0 + tr, e[k].1 + ti);
        result[k + n / 2] = (e[k].0 - tr, e[k].1 - ti);
    }
    result
}

/// Inverse DFT: x_n = (1/N) sum_{k=0}^{N-1} X_k * exp(+2*pi*i*k*n/N)
pub fn inverse_dft(x: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = x.len();
    assert!(n > 0, "inverse_dft: input must be non-empty");
    let mut result = Vec::with_capacity(n);
    let nf = n as f64;
    for idx in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for (k, &(xr, xi)) in x.iter().enumerate() {
            let angle = 2.0 * PI * (k as f64) * (idx as f64) / nf;
            let (sin_a, cos_a) = angle.sin_cos();
            re += xr * cos_a - xi * sin_a;
            im += xr * sin_a + xi * cos_a;
        }
        result.push((re / nf, im / nf));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn cpx_approx_eq(a: &[(f64, f64)], b: &[(f64, f64)], tol: f64) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .all(|((ar, ai), (br, bi))| approx_eq(*ar, *br, tol) && approx_eq(*ai, *bi, tol))
    }

    #[test]
    fn idft_roundtrip() {
        let signal = vec![(1.0, 0.0), (2.0, -1.0), (0.0, 3.0), (-1.0, 0.5)];
        let spectrum = dft(&signal);
        let recovered = inverse_dft(&spectrum);
        assert!(
            cpx_approx_eq(&signal, &recovered, 1e-10),
            "idft(dft(x)) == x"
        );
    }

    #[test]
    fn fft_matches_dft() {
        let signal: Vec<(f64, f64)> = (0..16)
            .map(|i| ((i as f64 * 0.3).sin(), (i as f64 * 0.7).cos()))
            .collect();
        let dft_result = dft(&signal);
        let fft_result = fft(&signal);
        assert!(cpx_approx_eq(&dft_result, &fft_result, 1e-10), "fft == dft");
    }

    #[test]
    fn dc_component() {
        // DFT of constant signal: X[0] = c*N, X[k>0] ~ 0
        let c = 3.0;
        let n = 8;
        let signal: Vec<(f64, f64)> = vec![(c, 0.0); n];
        let result = dft(&signal);
        assert!(approx_eq(result[0].0, c * n as f64, 1e-10));
        for r in &result[1..] {
            assert!(approx_eq(r.0, 0.0, 1e-10));
            assert!(approx_eq(r.1, 0.0, 1e-10));
        }
    }

    #[test]
    fn parseval_theorem() {
        let signal: Vec<(f64, f64)> = vec![(1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (4.0, 0.0)];
        let spectrum = dft(&signal);
        let time_energy: f64 = signal.iter().map(|(r, i)| r * r + i * i).sum();
        let freq_energy: f64 = spectrum.iter().map(|(r, i)| r * r + i * i).sum();
        let n = signal.len() as f64;
        assert!(approx_eq(time_energy, freq_energy / n, 1e-8), "Parseval");
    }

    #[test]
    fn fft_single_element() {
        let result = fft(&[(5.0, 3.0)]);
        assert!(approx_eq(result[0].0, 5.0, 1e-10));
        assert!(approx_eq(result[0].1, 3.0, 1e-10));
    }

    #[test]
    #[should_panic]
    fn fft_non_power_of_2() {
        let signal: Vec<(f64, f64)> = vec![(1.0, 0.0); 7];
        fft(&signal);
    }

    #[test]
    fn ifft_fft_roundtrip() {
        let signal: Vec<(f64, f64)> = vec![(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
        let spectrum = fft(&signal);
        let recovered = inverse_dft(&spectrum);
        assert!(cpx_approx_eq(&signal, &recovered, 1e-10));
    }

    #[test]
    fn dft_real_signal() {
        // Real signal: x = [1, 0, -1, 0]
        let signal = vec![(1.0, 0.0), (0.0, 0.0), (-1.0, 0.0), (0.0, 0.0)];
        let result = dft(&signal);
        assert!(approx_eq(result[0].0, 0.0, 1e-10), "DC = 0");
        assert!(approx_eq(result[1].0, 2.0, 1e-10), "X[1] = 2");
        assert!(approx_eq(result[2].0, 0.0, 1e-10), "X[2] = 0");
    }

    #[test]
    fn dft_matrix_correct_size() {
        let (re, im) = dft_matrix(4);
        assert_eq!(re.shape(), (4, 4));
        assert_eq!(im.shape(), (4, 4));
    }
}
