//! Falsification tests for every example-as-entity contract (`contracts/example-*-v1.yaml`).
//!
//! Each example is compiled here from its own source file, so these tests exercise
//! exactly the code `cargo run --example <name>` runs. The `falsify_ex*` names are
//! the `test:` bindings of the contracts' `falsification_tests`.

/// The tracked receipt for `name`, as committed under `evidence/examples/`.
fn evidence(name: &str) -> String {
    let path = format!(
        "{}/evidence/examples/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

// =========================================================================
// integrate — contracts/example-integrate-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/integrate.rs"]
mod integrate;

#[test]
fn falsify_exintg_001_all_checks_hold() {
    let r = integrate::receipt(true);
    assert_eq!(r.checks.len(), 7);
    assert!(r.all_pass(), "{r:?}");
}

#[test]
fn falsify_exintg_002_receipt_matches_evidence() {
    assert_eq!(integrate::receipt(true).to_json(), evidence("integrate"));
}

#[test]
fn falsify_exintg_003_simpson_bound_discriminates() {
    let pi = std::f64::consts::PI;
    let t = nlib::integrate::trapezoid(f64::sin, 0.0, pi, 100);
    assert!(
        !nlib::receipt::within(t, 2.0, 1.7e-8),
        "trapezoid passed the Simpson bound"
    );
}

#[test]
fn falsify_exintg_004_wrong_weights_miss_cubic() {
    // Simpson's 3/8 weights applied to a 1/3-rule panel pair: (1, 3, 3, 1) over 2 panels.
    let wrong = |f: fn(f64) -> f64| (f(0.0) + 3.0 * f(0.5) + 3.0 * f(0.5) + f(1.0)) * 0.5 / 3.0;
    let cube = |x: f64| x * x * x;
    assert!(!nlib::receipt::within(wrong(cube), 0.25, 1e-15));
    assert!(nlib::receipt::within(
        nlib::integrate::simpson(cube, 0.0, 1.0, 2),
        0.25,
        1e-15
    ));
}

/// The two tests every example contract binds first: all checks hold, and the
/// tracked receipt is the example's current output.
macro_rules! receipt_tests {
    ($m:ident, $n:expr, $all:ident, $evidence:ident) => {
        #[test]
        fn $all() {
            let r = $m::receipt(true);
            assert_eq!(r.checks.len(), $n);
            assert!(r.all_pass(), "{r:?}");
        }

        #[test]
        fn $evidence() {
            assert_eq!($m::receipt(true).to_json(), evidence(stringify!($m)));
        }
    };
}

// =========================================================================
// solve — contracts/example-solve-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/solve.rs"]
mod solve;
receipt_tests!(
    solve,
    5,
    falsify_exsolve_001_all_checks_hold,
    falsify_exsolve_002_receipt_matches_evidence
);

#[test]
fn falsify_exsolve_003_loose_tolerance_fails() {
    let b = nlib::solve::bisection(|x| x * x - 2.0, 1.0, 2.0, 1e-3);
    assert!(!nlib::receipt::within(b, std::f64::consts::SQRT_2, 1e-12));
}

#[test]
fn falsify_exsolve_004_residual_discriminates() {
    let f = |x: f64| x * x * x - x - 2.0;
    assert!(!nlib::receipt::within(f(1.5), 0.0, 1e-12));
}

// =========================================================================
// sort — contracts/example-sort-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/sort.rs"]
mod sort;
receipt_tests!(
    sort,
    7,
    falsify_exsort_001_all_checks_hold,
    falsify_exsort_002_receipt_matches_evidence
);

#[test]
fn falsify_exsort_003_unsorted_input_rejected() {
    assert!(!nlib::sort::is_sorted(&sort::DATA));
}

#[test]
fn falsify_exsort_004_lossy_output_rejected() {
    // Sorted, right length, but 82 was lost and 43 duplicated.
    let lossy = [3, 9, 10, 27, 38, 43, 43];
    assert!(nlib::sort::is_sorted(&lossy));
    assert!(!nlib::sort::is_permutation(&sort::DATA, &lossy));
}

// =========================================================================
// stats — contracts/example-stats-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/stats.rs"]
mod stats;
receipt_tests!(
    stats,
    6,
    falsify_exstats_001_all_checks_hold,
    falsify_exstats_002_receipt_matches_evidence
);

#[test]
fn falsify_exstats_003_sample_variance_fails() {
    let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let sample = nlib::stats::variance(&x) * 8.0 / 7.0;
    assert!(!nlib::receipt::within(sample, 4.0, 1e-12));
}

#[test]
fn falsify_exstats_004_negative_correlation_fails() {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [10.0, 8.0, 6.0, 4.0, 2.0];
    assert!(!nlib::receipt::within(
        nlib::stats::correlation(&a, &b),
        1.0,
        1e-12
    ));
}

// =========================================================================
// matrix — contracts/example-matrix-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/matrix.rs"]
mod matrix;
receipt_tests!(
    matrix,
    6,
    falsify_exmatrix_001_all_checks_hold,
    falsify_exmatrix_002_receipt_matches_evidence
);

#[test]
fn falsify_exmatrix_003_swapped_operands_fail() {
    use nlib::matrix::{Matrix, matmul};
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
    assert_ne!(matmul(&b, &a).data(), [19.0, 22.0, 43.0, 50.0]);
}

#[test]
fn falsify_exmatrix_004_perturbed_inverse_fails() {
    use nlib::matrix::{Matrix, inverse, matmul};
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let mut v = inverse(&a).expect("invertible").data().to_vec();
    v[0] += 1e-9;
    let inv = Matrix::new(2, 2, v);
    let err = matrix::max_abs_diff(&matmul(&a, &inv), &Matrix::identity(2));
    assert!(!nlib::receipt::within(err, 0.0, 1e-12));
}

// =========================================================================
// fourier — contracts/example-fourier-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/fourier.rs"]
mod fourier;
receipt_tests!(
    fourier,
    6,
    falsify_exfourier_001_all_checks_hold,
    falsify_exfourier_002_receipt_matches_evidence
);

#[test]
fn falsify_exfourier_003_wrong_frequency_fails() {
    let x = nlib::fourier::dft(&fourier::sine(8, 2.0));
    assert!(!nlib::receipt::within(fourier::modulus(x[1]), 4.0, 1e-12));
}

#[test]
fn falsify_exfourier_004_unnormalized_inverse_fails() {
    let signal = fourier::sine(8, 1.0);
    let unnormalized: Vec<(f64, f64)> = nlib::fourier::inverse_dft(&nlib::fourier::fft(&signal))
        .into_iter()
        .map(|(re, im)| (re * 8.0, im * 8.0))
        .collect();
    let err = fourier::max_diff(&signal, &unnormalized);
    assert!(!nlib::receipt::within(err, 0.0, 1e-12));
}

// =========================================================================
// graph — contracts/example-graph-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/graph.rs"]
mod graph;
receipt_tests!(
    graph,
    6,
    falsify_exgraph_001_all_checks_hold,
    falsify_exgraph_002_receipt_matches_evidence
);

#[test]
fn falsify_exgraph_003_unrelaxed_distances_fail() {
    assert!(graph::relaxed(&graph::DIST_FROM_0, &graph::EDGES));
    let long_way = [0.0, 4.0, 6.0, 1.0, 7.0, 11.0];
    assert!(!graph::relaxed(&long_way, &graph::EDGES));
}

#[test]
fn falsify_exgraph_004_non_minimal_tree_fails() {
    // 0-3, 4-5, 1-2, 1-4 and then 3-4 (6) instead of 0-1 (4): spanning, not minimal.
    let tree = [1.0, 1.0, 2.0, 3.0, 6.0];
    assert!(!nlib::receipt::within(
        tree.iter().sum(),
        graph::MST_WEIGHT,
        0.0
    ));
}

// =========================================================================
// monte_carlo — contracts/example-monte-carlo-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/monte_carlo.rs"]
mod monte_carlo;
receipt_tests!(
    monte_carlo,
    6,
    falsify_exmc_001_all_checks_hold,
    falsify_exmc_002_receipt_matches_evidence
);

#[test]
fn falsify_exmc_003_biased_estimate_fails() {
    let n = 100_000;
    let est = nlib::monte_carlo::mc_integrate(|x| x, 0.0, 1.0, n, 42);
    let band = 4.0 * monte_carlo::sigma_x2() / (n as f64).sqrt();
    assert!(!nlib::receipt::within(est, 1.0 / 3.0, band));
}

#[test]
fn falsify_exmc_004_unscaled_se_fails() {
    let d = &monte_carlo::DATA;
    let sigma = nlib::stats::std_dev(d);
    let analytic = sigma / (d.len() as f64).sqrt();
    assert!(!nlib::receipt::within(sigma, analytic, 0.1 * analytic));
}

// =========================================================================
// optimize — contracts/example-optimize-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/optimize.rs"]
mod optimize;
receipt_tests!(
    optimize,
    5,
    falsify_exopt_001_all_checks_hold,
    falsify_exopt_002_receipt_matches_evidence
);

#[test]
fn falsify_exopt_003_wrong_bracket_fails() {
    let x = nlib::optimize::golden_section(|x| (x - 3.0).powi(2), 4.0, 10.0, 1e-10);
    assert!(!nlib::receipt::within(x, 3.0, 1e-9));
}

#[test]
fn falsify_exopt_004_newton_from_half_finds_maximum() {
    use optimize::{quartic, quartic_d1, quartic_d2};
    let x = nlib::optimize::newton_optimize(quartic, quartic_d1, quartic_d2, 0.5, 1e-10);
    assert!(quartic_d1(x).abs() <= 1e-10, "still a critical point");
    assert!(quartic_d2(x) <= 0.0, "but a maximum, not a minimum");
}

// =========================================================================
// random — contracts/example-random-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/random.rs"]
mod random;
receipt_tests!(
    random,
    4,
    falsify_exrandom_001_all_checks_hold,
    falsify_exrandom_002_receipt_matches_evidence
);

#[test]
fn falsify_exrandom_003_wrong_multiplier_fails() {
    let mut lcg = nlib::random::Lcg::new(1, 16_808, 0, 2_147_483_647);
    let x10000 = (0..10_000).fold(0, |_, _| lcg.next_val());
    assert_ne!(x10000, 1_043_618_065);
}

#[test]
fn falsify_exrandom_004_degenerate_generator_fails() {
    let mut lcg = nlib::random::Lcg::new(1, 1, 1, 2_147_483_647);
    let chi2 = random::chi2_uniform(|| lcg.next_f64(), 10_000);
    assert!(chi2 >= random::CHI2_9DF_P01, "chi2 = {chi2}");
}

// =========================================================================
// parity — contracts/example-parity-v1.yaml
// =========================================================================
#[allow(dead_code)]
#[path = "../examples/parity.rs"]
mod parity;

#[test]
fn falsify_exparity_001_all_checks_hold() {
    let r = parity::receipt(&parity::values());
    assert_eq!(r.checks.len(), 23);
    assert!(r.all_pass(), "{r:?}");
}

#[test]
fn falsify_exparity_002_receipt_matches_evidence() {
    assert_eq!(
        parity::receipt(&parity::values()).to_json(),
        evidence("parity")
    );
}

#[test]
fn falsify_exparity_003_loose_solver_fails() {
    let b = nlib::solve::bisection(|x| x * x - 2.0, 1.0, 2.0, 1e-2);
    assert!(!nlib::receipt::within(b, std::f64::consts::SQRT_2, 1e-6));
}

#[test]
fn falsify_exparity_004_document_keys() {
    // The keys tests/falsify_parity.py compares, in document order.
    const KEYS: [&str; 23] = [
        "bisection",
        "newton",
        "secant",
        "fixed_point",
        "integrate_sin",
        "integrate_x2",
        "dft_impulse_0_re",
        "dft_impulse_0_im",
        "dft_dc_0_re",
        "matmul_00",
        "matmul_01",
        "matmul_10",
        "matmul_11",
        "determinant",
        "mean",
        "variance",
        "correlation",
        "dijkstra_0",
        "dijkstra_1",
        "dijkstra_2",
        "dijkstra_3",
        "dijkstra_4",
        "dijkstra_5",
    ];
    let vals = parity::values();
    let keys: Vec<&str> = vals.iter().map(|v| v.key).collect();
    assert_eq!(keys, KEYS);
    let doc = parity::document(&vals);
    assert!(doc.starts_with("{\n") && doc.ends_with("\n}\n"));
    assert_eq!(doc.lines().count(), 25);
}
