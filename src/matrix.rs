//! Dense matrix algebra — contract: `matrix-algebra-v1.yaml`
//!
//! Di Pierro Ch. 4.4: matmul, transpose, inverse, Cholesky, determinant.
//! Backed by `aprender::Matrix<f64>` for storage and basic ops.

use aprender::Matrix as AprMatrix;

/// Dense matrix in row-major order, backed by `aprender::Matrix<f64>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    inner: AprMatrix<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        let inner = AprMatrix::from_vec(rows, cols, data).expect("data.len() must equal rows*cols");
        Self { inner }
    }
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            inner: AprMatrix::from_vec(rows, cols, vec![0.0; rows * cols]).expect("valid"),
        }
    }
    pub fn identity(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Self {
            inner: AprMatrix::from_vec(n, n, data).expect("valid"),
        }
    }
    pub fn rows(&self) -> usize {
        self.inner.n_rows()
    }
    pub fn cols(&self) -> usize {
        self.inner.n_cols()
    }
    pub fn from_rows(rows: &[&[f64]]) -> Self {
        let ncols = rows[0].len();
        let mut data = Vec::with_capacity(rows.len() * ncols);
        for r in rows {
            assert_eq!(r.len(), ncols);
            data.extend_from_slice(r);
        }
        Self {
            inner: AprMatrix::from_vec(rows.len(), ncols, data).expect("valid"),
        }
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        // aprender::Matrix::get returns by value; we need a reference.
        // Use the underlying slice via as_slice().
        let cols = self.inner.n_cols();
        &self.inner.as_slice()[r * cols + c]
    }
}
impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        let cols = self.inner.n_cols();
        let idx = r * cols + c;
        // Safety: we need mutable slice access. Rebuild via raw pointer.
        // aprender doesn't expose as_mut_slice on Matrix, so we reconstruct.
        let ptr = self.inner.as_slice().as_ptr() as *mut f64;
        unsafe { &mut *ptr.add(idx) }
    }
}

/// C[i,j] = sum_k A[i,k]*B[k,j]
pub fn matmul(a: &Matrix, b: &Matrix) -> Matrix {
    assert_eq!(a.cols(), b.rows(), "matmul: A.cols must equal B.rows");
    let (m, p, n) = (a.rows(), a.cols(), b.cols());
    let mut c = Matrix::zeros(m, n);
    for i in 0..m {
        for k in 0..p {
            let a_ik = a[(i, k)];
            for j in 0..n {
                c[(i, j)] += a_ik * b[(k, j)];
            }
        }
    }
    c
}

/// A^T[i,j] = A[j,i]
pub fn transpose(a: &Matrix) -> Matrix {
    let mut data = vec![0.0; a.rows() * a.cols()];
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            data[j * a.rows() + i] = a[(i, j)];
        }
    }
    Matrix::new(a.cols(), a.rows(), data)
}

/// Gauss-Jordan elimination. Returns None if singular.
pub fn inverse(a: &Matrix) -> Option<Matrix> {
    assert_eq!(a.rows(), a.cols(), "inverse: must be square");
    let n = a.rows();
    let mut aug = Matrix::zeros(n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            aug[(i, j)] = a[(i, j)];
        }
        aug[(i, n + i)] = 1.0;
    }
    for col in 0..n {
        let (mut mx_row, mut mx_val) = (col, aug[(col, col)].abs());
        for row in (col + 1)..n {
            let v = aug[(row, col)].abs();
            if v > mx_val {
                mx_val = v;
                mx_row = row;
            }
        }
        if mx_val < 1e-14 {
            return None;
        }
        if mx_row != col {
            for j in 0..(2 * n) {
                let tmp = aug[(col, j)];
                aug[(col, j)] = aug[(mx_row, j)];
                aug[(mx_row, j)] = tmp;
            }
        }
        let pivot = aug[(col, col)];
        for j in 0..(2 * n) {
            aug[(col, j)] /= pivot;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let f = aug[(row, col)];
            for j in 0..(2 * n) {
                aug[(row, j)] -= f * aug[(col, j)];
            }
        }
    }
    let mut inv = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            inv[(i, j)] = aug[(i, n + j)];
        }
    }
    Some(inv)
}

/// Cholesky: A = L*L^T. Returns None if not SPD.
pub fn cholesky(a: &Matrix) -> Option<Matrix> {
    assert_eq!(a.rows(), a.cols(), "cholesky: must be square");
    let n = a.rows();
    let mut l = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..=i {
            let s: f64 = (0..j).map(|k| l[(i, k)] * l[(j, k)]).sum();
            if i == j {
                let d = a[(i, i)] - s;
                if d <= 0.0 {
                    return None;
                }
                l[(i, j)] = d.sqrt();
            } else {
                if l[(j, j)].abs() < 1e-14 {
                    return None;
                }
                l[(i, j)] = (a[(i, j)] - s) / l[(j, j)];
            }
        }
    }
    Some(l)
}

/// Determinant via LU with partial pivoting.
pub fn determinant(a: &Matrix) -> f64 {
    assert_eq!(a.rows(), a.cols(), "determinant: must be square");
    let n = a.rows();
    if n == 0 {
        return 1.0;
    }
    let mut m = a.clone();
    let mut sign = 1.0_f64;
    for col in 0..n {
        let (mut mx_row, mut mx_val) = (col, m[(col, col)].abs());
        for row in (col + 1)..n {
            let v = m[(row, col)].abs();
            if v > mx_val {
                mx_val = v;
                mx_row = row;
            }
        }
        if mx_val < 1e-14 {
            return 0.0;
        }
        if mx_row != col {
            for j in 0..n {
                let tmp = m[(col, j)];
                m[(col, j)] = m[(mx_row, j)];
                m[(mx_row, j)] = tmp;
            }
            sign = -sign;
        }
        for row in (col + 1)..n {
            let f = m[(row, col)] / m[(col, col)];
            for j in (col + 1)..n {
                m[(row, j)] -= f * m[(col, j)];
            }
        }
    }
    (0..n).fold(sign, |acc, i| acc * m[(i, i)])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn approx(a: f64, b: f64, t: f64) -> bool {
        (a - b).abs() < t
    }
    fn mat_eq(a: &Matrix, b: &Matrix, t: f64) -> bool {
        a.rows() == b.rows()
            && a.cols() == b.cols()
            && (0..a.rows()).all(|i| (0..a.cols()).all(|j| approx(a[(i, j)], b[(i, j)], t)))
    }
    #[test]
    fn identity_multiply() {
        let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
        let i = Matrix::identity(2);
        assert!(mat_eq(&matmul(&a, &i), &a, 1e-10));
        assert!(mat_eq(&matmul(&i, &a), &a, 1e-10));
    }
    #[test]
    fn matmul_known() {
        let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
        let b = Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
        let c = matmul(&a, &b);
        assert!(approx(c[(0, 0)], 19.0, 1e-10) && approx(c[(0, 1)], 22.0, 1e-10));
        assert!(approx(c[(1, 0)], 43.0, 1e-10) && approx(c[(1, 1)], 50.0, 1e-10));
    }
    #[test]
    fn matmul_1x1() {
        let c = matmul(&Matrix::new(1, 1, vec![3.0]), &Matrix::new(1, 1, vec![7.0]));
        assert!(approx(c[(0, 0)], 21.0, 1e-10));
    }
    #[test]
    #[should_panic]
    fn matmul_dim_mismatch() {
        matmul(&Matrix::zeros(2, 3), &Matrix::zeros(4, 2));
    }
    #[test]
    fn transpose_involution() {
        let a = Matrix::from_rows(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        assert!(mat_eq(&transpose(&transpose(&a)), &a, 1e-10));
    }
    #[test]
    fn transpose_shape() {
        let t = transpose(&Matrix::zeros(2, 3));
        assert_eq!((t.rows(), t.cols()), (3, 2));
    }
    #[test]
    fn inverse_identity() {
        let i = Matrix::identity(3);
        assert!(mat_eq(&inverse(&i).unwrap(), &i, 1e-10));
    }
    #[test]
    fn inverse_roundtrip() {
        let a = Matrix::from_rows(&[&[4.0, 7.0], &[2.0, 6.0]]);
        assert!(mat_eq(
            &matmul(&a, &inverse(&a).unwrap()),
            &Matrix::identity(2),
            1e-8
        ));
    }
    #[test]
    fn inverse_3x3() {
        let a = Matrix::from_rows(&[&[1.0, 2.0, 3.0], &[0.0, 1.0, 4.0], &[5.0, 6.0, 0.0]]);
        assert!(mat_eq(
            &matmul(&a, &inverse(&a).unwrap()),
            &Matrix::identity(3),
            1e-8
        ));
    }
    #[test]
    fn inverse_singular() {
        assert!(inverse(&Matrix::zeros(3, 3)).is_none());
    }
    #[test]
    fn cholesky_roundtrip() {
        let a = Matrix::from_rows(&[&[4.0, 2.0], &[2.0, 3.0]]);
        let l = cholesky(&a).unwrap();
        assert!(mat_eq(&matmul(&l, &transpose(&l)), &a, 1e-10));
    }
    #[test]
    fn cholesky_3x3() {
        let a = Matrix::from_rows(&[&[25.0, 15.0, -5.0], &[15.0, 18.0, 0.0], &[-5.0, 0.0, 11.0]]);
        let l = cholesky(&a).unwrap();
        assert!(mat_eq(&matmul(&l, &transpose(&l)), &a, 1e-10));
        for i in 0..3 {
            for j in (i + 1)..3 {
                assert!(approx(l[(i, j)], 0.0, 1e-14));
            }
        }
    }
    #[test]
    fn cholesky_not_spd() {
        assert!(cholesky(&Matrix::from_rows(&[&[-1.0, 0.0], &[0.0, 1.0]])).is_none());
    }
    #[test]
    fn det_identity() {
        assert!(approx(determinant(&Matrix::identity(3)), 1.0, 1e-10));
    }
    #[test]
    fn det_2x2() {
        assert!(approx(
            determinant(&Matrix::from_rows(&[&[3.0, 8.0], &[4.0, 6.0]])),
            -14.0,
            1e-10
        ));
    }
    #[test]
    fn det_singular() {
        assert!(approx(
            determinant(&Matrix::from_rows(&[&[1.0, 2.0], &[2.0, 4.0]])),
            0.0,
            1e-10
        ));
    }
    #[test]
    fn det_3x3() {
        let a = Matrix::from_rows(&[&[6.0, 1.0, 1.0], &[4.0, -2.0, 5.0], &[2.0, 8.0, 7.0]]);
        assert!(approx(determinant(&a), -306.0, 1e-8));
    }
    #[test]
    fn det_multiplicativity() {
        let a = Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
        let b = Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
        assert!(approx(
            determinant(&matmul(&a, &b)),
            determinant(&a) * determinant(&b),
            1e-8
        ));
    }
}
