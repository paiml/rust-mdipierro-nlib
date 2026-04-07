//! Matrix algebra — Di Pierro Ch. 4.4
//! cargo run --example matrix
use nlib::matrix::{Matrix, cholesky, determinant, inverse, matmul, transpose};

fn main() {
    let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
    let c = matmul(&a, &b);
    println!("A = [[1,2],[3,4]]");
    println!("B = [[5,6],[7,8]]");
    println!("A*B = {:?}", c.data());

    let at = transpose(&a);
    println!("\nA^T = {:?}", at.data());

    let inv = inverse(&a).expect("invertible");
    let check = matmul(&a, &inv);
    println!("A^-1 = {:?}", inv.data());
    println!(
        "A*A^-1 ≈ I: [{:.4}, {:.4}; {:.4}, {:.4}]",
        check.get(0, 0),
        check.get(0, 1),
        check.get(1, 0),
        check.get(1, 1)
    );

    let spd = Matrix::from_rows(&[&[4.0, 2.0], &[2.0, 3.0]]);
    let l = cholesky(&spd).expect("positive definite");
    println!("\nCholesky of [[4,2],[2,3]]: {:?}", l.data());

    println!("det(A) = {:.1}", determinant(&a));
}
