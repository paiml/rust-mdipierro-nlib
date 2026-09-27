//! Fourier transforms — Di Pierro, *Annotated Algorithms in Python*, §4.11.
//! Contract: `contracts/example-fourier-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example fourier            # human-readable
//! cargo run --example fourier -- --json  # the receipt in evidence/examples/fourier.json
use nlib::fourier::{dft, fft, inverse_dft};
use nlib::receipt::{Receipt, json_requested};

/// `n` samples of a sine with `cycles` periods over the window.
pub fn sine(n: usize, cycles: f64) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            ((2.0 * std::f64::consts::PI * cycles * t).sin(), 0.0)
        })
        .collect()
}

/// |z| of a complex (re, im) pair.
pub fn modulus(z: (f64, f64)) -> f64 {
    z.0.hypot(z.1)
}

/// Largest |x_k - y_k| over two equal-length complex sequences.
pub fn max_diff(x: &[(f64, f64)], y: &[(f64, f64)]) -> f64 {
    assert_eq!(x.len(), y.len());
    x.iter()
        .zip(y)
        .map(|(p, q)| modulus((p.0 - q.0, p.1 - q.1)))
        .fold(0.0, f64::max)
}

/// Every claim this example makes, as a receipt. Each check names the
/// `example-fourier-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("fourier");

    // One period of a sine over N = 8 samples: all energy in bins 1 and N-1,
    // each of magnitude N/2.
    let n = 8;
    let signal = sine(n, 1.0);
    say(format!("Signal (1 cycle of sine, {n} samples):"));
    for (i, (re, _)) in signal.iter().enumerate() {
        say(format!("  x[{i}] = {re:.4}"));
    }
    let spectrum = dft(&signal);
    say("\nDFT spectrum (magnitude):".into());
    for (k, &z) in spectrum.iter().enumerate() {
        say(format!("  X[{k}] = {:.4}", modulus(z)));
    }
    let half = n as f64 / 2.0;
    r.close("dft_sine_peak", modulus(spectrum[1]), half, 1e-12);
    r.close("dft_sine_peak", modulus(spectrum[n - 1]), half, 1e-12);
    let leak = (0..n)
        .filter(|&k| k != 1 && k != n - 1)
        .map(|k| modulus(spectrum[k]))
        .fold(0.0, f64::max);
    r.close("dft_sine_no_leakage", leak, 0.0, 1e-12);

    let fast = fft(&signal);
    let fft_err = max_diff(&spectrum, &fast);
    say(format!("\nFFT vs DFT max error: {fft_err:.2e}"));
    r.close("fft_matches_dft", fft_err, 0.0, 1e-12);

    let recovered = inverse_dft(&fast);
    let rt_err = max_diff(&signal, &recovered);
    say(format!("IDFT(FFT(x)) roundtrip error: {rt_err:.2e}"));
    r.close("idft_roundtrip", rt_err, 0.0, 1e-12);

    // Parseval: Σ|x|² = (1/N) Σ|X|²
    let e_t: f64 = signal.iter().map(|&z| modulus(z).powi(2)).sum();
    let e_f: f64 = spectrum.iter().map(|&z| modulus(z).powi(2)).sum();
    r.close("parseval", e_t, e_f / n as f64, 1e-12);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
