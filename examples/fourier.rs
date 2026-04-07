//! Fourier transforms — Di Pierro Ch. 4.11
//! cargo run --example fourier
use nlib::fourier::{dft, fft, inverse_dft};

fn main() {
    // Signal: 1 Hz sine wave, 8 samples
    let n = 8;
    let signal: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            ((2.0 * std::f64::consts::PI * t).sin(), 0.0)
        })
        .collect();

    println!("Signal (1 Hz sine, {n} samples):");
    for (i, (re, _)) in signal.iter().enumerate() {
        println!("  x[{i}] = {re:.4}");
    }

    let spectrum = dft(&signal);
    println!("\nDFT spectrum (magnitude):");
    for (k, (re, im)) in spectrum.iter().enumerate() {
        let mag = (re * re + im * im).sqrt();
        println!("  X[{k}] = {mag:.4}");
    }

    // FFT should match DFT
    let fft_result = fft(&signal);
    let mut max_diff = 0.0f64;
    for (d, f) in spectrum.iter().zip(fft_result.iter()) {
        let diff = ((d.0 - f.0).powi(2) + (d.1 - f.1).powi(2)).sqrt();
        max_diff = max_diff.max(diff);
    }
    println!("\nFFT vs DFT max error: {max_diff:.2e}");

    // Roundtrip: ifft(fft(x)) ≈ x
    let recovered = inverse_dft(&fft_result);
    let mut roundtrip_err = 0.0f64;
    for (orig, rec) in signal.iter().zip(recovered.iter()) {
        roundtrip_err += (orig.0 - rec.0).powi(2) + (orig.1 - rec.1).powi(2);
    }
    println!("IFFT(FFT(x)) roundtrip error: {:.2e}", roundtrip_err.sqrt());
}
