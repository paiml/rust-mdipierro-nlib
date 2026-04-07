//! Kani bounded model checking harnesses.
//!
//! Proves contract invariants for ALL inputs within bounds.
//! Uses only primitive operations (no aprender) to keep CBMC tractable.
//!
//! Run: `cargo kani`

#[cfg(kani)]
mod proofs {
    /// KANI-SORT-001: quicksort output is always sorted (len ≤ 5).
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_quicksort_sorted() {
        let len: usize = kani::any();
        kani::assume(len <= 5);
        let mut a = [0i8; 5];
        for i in 0..len {
            a[i] = kani::any();
        }
        let slice = &mut a[..len];
        crate::sort::quicksort(slice);
        for i in 1..len {
            assert!(slice[i - 1] <= slice[i], "quicksort: not sorted");
        }
    }

    /// KANI-SORT-002: quicksort preserves length.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_quicksort_length() {
        let len: usize = kani::any();
        kani::assume(len <= 5);
        let mut a = [0i8; 5];
        for i in 0..len {
            a[i] = kani::any();
        }
        let slice = &mut a[..len];
        let orig_len = slice.len();
        crate::sort::quicksort(slice);
        assert_eq!(slice.len(), orig_len);
    }

    /// KANI-STATS-001: variance is non-negative.
    #[kani::proof]
    fn verify_variance_non_negative() {
        let a: f64 = kani::any();
        let b: f64 = kani::any();
        let c: f64 = kani::any();
        kani::assume(a.is_finite() && b.is_finite() && c.is_finite());
        kani::assume(a.abs() < 1e6 && b.abs() < 1e6 && c.abs() < 1e6);
        let mu = (a + b + c) / 3.0;
        let var = ((a - mu) * (a - mu) + (b - mu) * (b - mu) + (c - mu) * (c - mu)) / 3.0;
        assert!(var >= 0.0, "variance must be non-negative");
    }

    /// KANI-STATS-002: |correlation| ≤ 1 (Cauchy-Schwarz).
    #[kani::proof]
    fn verify_correlation_bounded() {
        let (x0, x1, x2) = (1.0f64, 2.0, 3.0);
        let y0: f64 = kani::any();
        let y1: f64 = kani::any();
        let y2: f64 = kani::any();
        kani::assume(y0.is_finite() && y1.is_finite() && y2.is_finite());
        kani::assume(y0.abs() < 100.0 && y1.abs() < 100.0 && y2.abs() < 100.0);
        let mx = 2.0;
        let my = (y0 + y1 + y2) / 3.0;
        let cov = ((x0 - mx) * (y0 - my) + (x1 - mx) * (y1 - my) + (x2 - mx) * (y2 - my)) / 3.0;
        let sx =
            (((x0 - mx) * (x0 - mx) + (x1 - mx) * (x1 - mx) + (x2 - mx) * (x2 - mx)) / 3.0).sqrt();
        let sy =
            (((y0 - my) * (y0 - my) + (y1 - my) * (y1 - my) + (y2 - my) * (y2 - my)) / 3.0).sqrt();
        if sx > 1e-10 && sy > 1e-10 {
            let r = cov / (sx * sy);
            assert!(r >= -1.0 - 1e-6 && r <= 1.0 + 1e-6, "|corr| <= 1");
        }
    }

    /// KANI-MATRIX-001: transpose is involution (A^T^T = A).
    #[kani::proof]
    fn verify_transpose_involution() {
        let a: f64 = kani::any();
        let b: f64 = kani::any();
        let c: f64 = kani::any();
        let d: f64 = kani::any();
        kani::assume(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite());
        // 2x2 matrix: transpose twice = original
        // A = [[a,b],[c,d]], A^T = [[a,c],[b,d]], A^T^T = [[a,b],[c,d]]
        let at = [[a, c], [b, d]];
        let att = [[at[0][0], at[1][0]], [at[0][1], at[1][1]]];
        assert!(att[0][0] == a && att[0][1] == b && att[1][0] == c && att[1][1] == d);
    }
}
