//! Matrix algebra — Di Pierro, *Annotated Algorithms in Python*, §4.4.
//! Contract: `contracts/example-matrix-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example matrix            # human-readable
//! cargo run --example matrix -- --json  # the receipt in evidence/examples/matrix.json
use nlib::matrix::{Matrix, cholesky, determinant, inverse, matmul, transpose};
use nlib::receipt::{Receipt, json_requested};

/// Largest entrywise |x - y| of two equally shaped matrices.
pub fn max_abs_diff(x: &Matrix, y: &Matrix) -> f64 {
    assert_eq!((x.rows(), x.cols()), (y.rows(), y.cols()));
    x.data()
        .iter()
        .zip(y.data())
        .map(|(p, q)| (p - q).abs())
        .fold(0.0, f64::max)
}

/// Every claim this example makes, as a receipt. Each check names the
/// `example-matrix-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("matrix");

    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
    let c = matmul(&a, &b);
    say("A = [[1,2],[3,4]]".into());
    say("B = [[5,6],[7,8]]".into());
    say(format!("A*B = {:?}", c.data()));
    r.check("matmul_known", c.data() == [19.0, 22.0, 43.0, 50.0]);

    let at = transpose(&a);
    say(format!("\nA^T = {:?}", at.data()));
    r.check("transpose_known", at.data() == [1.0, 3.0, 2.0, 4.0]);

    let inv = inverse(&a).expect("A is invertible");
    let id = matmul(&a, &inv);
    say(format!("A^-1 = {:?}", inv.data()));
    say(format!("A*A^-1 = {:?}", id.data()));
    r.close(
        "inverse_identity",
        max_abs_diff(&id, &Matrix::identity(2)),
        0.0,
        1e-12,
    );

    let spd = Matrix::from_rows(&[&[4.0, 2.0], &[2.0, 3.0]]);
    let l = cholesky(&spd).expect("positive definite");
    say(format!("\nCholesky of [[4,2],[2,3]]: {:?}", l.data()));
    r.close(
        "cholesky_reconstructs",
        max_abs_diff(&matmul(&l, &transpose(&l)), &spd),
        0.0,
        1e-12,
    );
    r.check("cholesky_lower_triangular", l.get(0, 1) == 0.0);

    let det = determinant(&a);
    say(format!("det(A) = {det:.1}"));
    r.close("determinant_known", det, -2.0, 1e-12);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
