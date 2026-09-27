//! Machine-readable parity values for Python/Rust cross-validation.
//! Contract: `contracts/example-parity-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example parity            # the values document tests/falsify_parity.py compares
//! cargo run --example parity -- --json  # the receipt in evidence/examples/parity.json
//!
//! Every value is also checked here against its closed form, so the Rust side
//! is pinned to the truth, not only to Di Pierro's nlib.py.
use nlib::receipt::{Receipt, json_requested};
use std::f64::consts::{PI, SQRT_2};

/// One value of the parity document, with the closed form it must match.
pub struct Value {
    /// Key in the parity document (read by tests/falsify_parity.py).
    pub key: &'static str,
    /// The `example-parity-v1` equation the value exercises.
    pub equation: &'static str,
    pub got: f64,
    pub want: f64,
    pub tol: f64,
}

/// Iterative results use nlib's default absolute precision (ap = 1e-6).
const AP: f64 = 1e-6;
/// Exact results must match to rounding.
const EXACT: f64 = 1e-12;

/// Every parity value, in document order.
pub fn values() -> Vec<Value> {
    let v = |key, equation, got, want, tol| Value {
        key,
        equation,
        got,
        want,
        tol,
    };
    // cos(x) = x
    let dottie = 0.739_085_133_215_160_6;
    let mut out = vec![
        v(
            "bisection",
            "solver_root",
            nlib::solve::bisection(|x| x * x - 2.0, 1.0, 2.0, AP),
            SQRT_2,
            AP,
        ),
        v(
            "newton",
            "solver_root",
            nlib::solve::newton(|x| x * x - 2.0, |x| 2.0 * x, 1.5, AP),
            SQRT_2,
            AP,
        ),
        v(
            "secant",
            "solver_root",
            nlib::solve::secant(|x| x * x - 2.0, 1.0, 2.0, AP),
            SQRT_2,
            AP,
        ),
        // nlib's fixed_point solves f(x) = 0 via g(x) = f(x) + x; cos(x) = x is g = cos.
        v(
            "fixed_point",
            "solver_root",
            nlib::solve::fixed_point(f64::cos, 1.0, AP),
            dottie,
            4.0 * AP,
        ),
        v(
            "integrate_sin",
            "integral",
            nlib::integrate::adaptive_quadrature(|x| 1.01 * x.sin(), 0.0, PI, AP),
            2.02,
            AP,
        ),
        v(
            "integrate_x2",
            "integral",
            nlib::integrate::adaptive_quadrature(|x| x * x, 0.0, 1.0, AP),
            1.0 / 3.0,
            AP,
        ),
    ];

    let impulse = nlib::fourier::dft(&[(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)]);
    let dc = nlib::fourier::dft(&[(3.0, 0.0); 4]);
    out.push(v("dft_impulse_0_re", "dft", impulse[0].0, 1.0, EXACT));
    out.push(v("dft_impulse_0_im", "dft", impulse[0].1, 0.0, EXACT));
    out.push(v("dft_dc_0_re", "dft", dc[0].0, 12.0, EXACT));

    let a = nlib::matrix::Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = nlib::matrix::Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
    let c = nlib::matrix::matmul(&a, &b);
    out.push(v("matmul_00", "matrix", c.get(0, 0), 19.0, EXACT));
    out.push(v("matmul_01", "matrix", c.get(0, 1), 22.0, EXACT));
    out.push(v("matmul_10", "matrix", c.get(1, 0), 43.0, EXACT));
    out.push(v("matmul_11", "matrix", c.get(1, 1), 50.0, EXACT));
    out.push(v(
        "determinant",
        "matrix",
        nlib::matrix::determinant(&a),
        -2.0,
        EXACT,
    ));

    let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    out.push(v("mean", "stats", nlib::stats::mean(&x), 5.0, EXACT));
    out.push(v(
        "variance",
        "stats",
        nlib::stats::variance(&x),
        4.0,
        EXACT,
    ));
    let va = [1.0, 2.0, 3.0, 4.0, 5.0];
    let vb = [2.0, 4.0, 6.0, 8.0, 10.0];
    out.push(v(
        "correlation",
        "stats",
        nlib::stats::correlation(&va, &vb),
        1.0,
        EXACT,
    ));

    let mut g = nlib::graph::Graph::new(6);
    for (u, w, d) in [
        (0, 1, 4.0),
        (1, 2, 2.0),
        (0, 3, 1.0),
        (1, 4, 3.0),
        (2, 5, 5.0),
        (3, 4, 6.0),
        (4, 5, 1.0),
    ] {
        g.add_undirected_edge(u, w, d);
    }
    const KEYS: [&str; 6] = [
        "dijkstra_0",
        "dijkstra_1",
        "dijkstra_2",
        "dijkstra_3",
        "dijkstra_4",
        "dijkstra_5",
    ];
    let want = [0.0, 4.0, 6.0, 1.0, 7.0, 8.0];
    for (i, d) in nlib::graph::dijkstra(&g, 0).into_iter().enumerate() {
        out.push(v(KEYS[i], "dijkstra", d, want[i], EXACT));
    }
    out
}

/// The values document: one `"key": value` line per value, 15 significant digits.
pub fn document(values: &[Value]) -> String {
    let body: Vec<String> = values
        .iter()
        .map(|v| format!("  \"{}\": {:.15e}", v.key, v.got))
        .collect();
    format!("{{\n{}\n}}\n", body.join(",\n"))
}

/// Every claim this example makes, as a receipt: each value against its closed form.
pub fn receipt(values: &[Value]) -> Receipt {
    let mut r = Receipt::new("parity");
    for v in values {
        r.close(v.equation, v.got, v.want, v.tol);
    }
    r
}

fn main() {
    let vals = values();
    let r = receipt(&vals);
    if json_requested() {
        std::process::exit(r.emit(true));
    }
    print!("{}", document(&vals));
    std::process::exit(i32::from(!r.all_pass()));
}
