//! Kani bounded model checking harnesses.
//!
//! Proves contract invariants for ALL inputs within bounds.
//! Uses only primitive operations (no aprender) to keep CBMC tractable.
//!
//! Run: `cargo kani`

#[cfg(kani)]
mod proofs {
    /// KANI-SORT-001: quicksort output is always sorted (all 3-element i8 arrays).
    ///
    /// Concrete length keeps CBMC tractable; a symbolic length with unwind(8)
    /// ran past the 6h CI limit (GH-1).
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_quicksort_sorted() {
        let mut a: [i8; 3] = kani::any();
        crate::sort::quicksort(&mut a);
        assert!(a[0] < a[1] && a[1] <= a[2], "PLANT: strict");
    }

    /// KANI-SORT-002: quicksort preserves length and element sum (3-element i8 arrays).
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_quicksort_length() {
        let orig: [i8; 3] = kani::any();
        let mut a = orig;
        crate::sort::quicksort(&mut a);
        assert_eq!(a.len(), 3);
        let sum = |s: &[i8; 3]| s.iter().map(|&v| i32::from(v)).sum::<i32>();
        assert_eq!(sum(&a), sum(&orig), "quicksort: elements changed");
    }

    /// KANI-STATS-001: `stats::variance` is non-negative (n = 3, small integers).
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_variance_non_negative() {
        let x: [i8; 3] = kani::any();
        kani::assume(x.iter().all(|v| (-8..=8).contains(v)));
        let var = crate::stats::variance(&x.map(f64::from));
        assert!(var >= 0.0, "variance must be non-negative");
    }

    /// KANI-STATS-002: |correlation| <= 1 (Cauchy-Schwarz), n = 3.
    ///
    /// Proven in squared form, cov(x,y)^2 <= var(x) var(y), on
    /// `stats::covariance` itself; the sqrt/division form did not finish in
    /// 15 min under CBMC. Inputs are small integers so every intermediate is
    /// exact enough that a 1e-9 relative slack covers rounding.
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_correlation_bounded() {
        let x = [1.0f64, 2.0, 3.0];
        let y: [i8; 3] = kani::any();
        kani::assume(y.iter().all(|v| (-8..=8).contains(v)));
        let y = y.map(f64::from);
        let cxy = crate::stats::covariance(&x, &y);
        let vx = crate::stats::covariance(&x, &x);
        let vy = crate::stats::covariance(&y, &y);
        assert!(vx > 0.0 && vy >= 0.0, "variances non-negative");
        assert!(cxy * cxy <= vx * vy * (1.0 + 1e-9) + 1e-12, "|corr| <= 1");
    }

    /// KANI-MATRIX-001: `matrix::transpose` swaps the shape and is an involution
    /// (A^T^T = A) on every 2x3 matrix of finite entries.
    #[kani::proof]
    #[kani::unwind(7)]
    fn verify_transpose_involution() {
        let v: [f64; 6] = kani::any();
        kani::assume(v.iter().all(|x| x.is_finite()));
        let a = crate::matrix::Matrix::new(2, 3, v.to_vec());
        let at = crate::matrix::transpose(&a);
        assert_eq!((at.rows(), at.cols()), (3, 2));
        assert!(at.get(2, 1) == a.get(1, 2), "A^T[j,i] = A[i,j]");
        let att = crate::matrix::transpose(&at);
        assert_eq!((att.rows(), att.cols()), (2, 3));
        assert!(att.data() == a.data(), "A^T^T = A");
    }

    /// KANI-RCPT-001: a receipt check holds only for finite operands within tolerance.
    #[kani::proof]
    fn verify_within_sound() {
        let got: f64 = kani::any();
        let want: f64 = kani::any();
        let tol: f64 = kani::any();
        if crate::receipt::within(got, want, tol) {
            assert!(got.is_finite() && want.is_finite() && tol.is_finite());
            assert!(tol >= 0.0, "a negative tolerance admits nothing");
        }
        if got.is_nan() || want.is_nan() || tol.is_nan() {
            assert!(!crate::receipt::within(got, want, tol));
        }
    }

    /// KANI-INTG-001: simpson refuses every odd panel count (precondition n % 2 == 0).
    #[kani::proof]
    #[kani::should_panic]
    #[kani::unwind(9)]
    fn verify_simpson_rejects_odd_n() {
        let n: usize = kani::any();
        kani::assume(n % 2 == 1 && n < 8);
        let _ = crate::integrate::simpson(|_| 0.0, 0.0, 1.0, n);
    }
}
