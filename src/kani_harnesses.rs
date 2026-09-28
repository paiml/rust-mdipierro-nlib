//! Kani bounded model checking harnesses.
//!
//! Proves contract invariants for ALL inputs within bounds.
//! Inputs are small and bounded to keep CBMC tractable; each doc comment names its domain.
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
        assert!(a[0] <= a[1] && a[1] <= a[2], "quicksort: not sorted");
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

    // ---- GH-6: one harness for every `harness:` a contract names. Each doc comment states the
    // domain it covers; a property Kani cannot state for all inputs is proved on the bounded
    // instance named there, never claimed in general.

    /// Small integers as f64: every one is exact, so equalities below are exact, not rounded.
    fn small(lo: i8, hi: i8) -> f64 {
        let v: i8 = kani::any();
        kani::assume(v >= lo && v <= hi);
        f64::from(v)
    }

    /// KANI-SORT-002: quicksort's output is a permutation of its input (all 3-element i8 arrays).
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_sort_permutation() {
        let orig: [i8; 3] = kani::any();
        let mut a = orig;
        crate::sort::quicksort(&mut a);
        for v in orig {
            let before = orig.iter().filter(|&&w| w == v).count();
            let after = a.iter().filter(|&&w| w == v).count();
            assert_eq!(before, after, "quicksort: multiset changed");
        }
    }

    /// Sorts by `key` only; `tag` records the input position.
    #[derive(Clone, Copy, Debug)]
    struct Keyed {
        key: u8,
        tag: u8,
    }
    impl PartialEq for Keyed {
        fn eq(&self, o: &Self) -> bool {
            self.key == o.key
        }
    }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Keyed {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            self.key.cmp(&o.key)
        }
    }

    /// KANI-SORT-003: mergesort is stable: equal keys keep their input order, for all 16
    /// 4-element inputs with keys in 0..=1, enumerated concretely: with symbolic keys, merge's
    /// symbolic slice lengths did not finish in CBMC.
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_mergesort_stable() {
        for hi in 0..4u8 {
            for lo in 0..4u8 {
                let bits = hi * 4 + lo;
                let input: Vec<Keyed> = (0..4u8)
                    .map(|i| Keyed {
                        key: bits >> i & 1,
                        tag: i,
                    })
                    .collect();
                let out = crate::sort::mergesort(&input);
                assert_eq!(out.len(), 4);
                for i in 0..3 {
                    assert!(out[i].key <= out[i + 1].key, "mergesort: not sorted");
                    if out[i].key == out[i + 1].key {
                        assert!(
                            out[i].tag < out[i + 1].tag,
                            "mergesort: equal keys reordered"
                        );
                    }
                }
            }
        }
    }

    /// KANI-STAT-001: variance(x) >= 0 for every 4-element x with integer entries in -4..=4.
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_variance_nonneg() {
        let x = [small(-4, 4), small(-4, 4), small(-4, 4), small(-4, 4)];
        let var = crate::stats::variance(&x);
        assert!(var >= 0.0, "variance negative");
    }

    /// KANI-STAT-003: chi_squared >= 0 for every 2-cell table with observed counts in 0..=8 and
    /// expected counts in 1..=8 (the contract's precondition E_i > 0).
    #[kani::proof]
    #[kani::unwind(4)]
    #[kani::stub(aprender::stats::hypothesis::chi_square_pvalue, stub_chi_square_pvalue)]
    #[kani::stub(f32::powi, stub_powi_f32)]
    #[kani::stub(f64::powi, stub_powi_f64)]
    fn verify_chi2_nonneg() {
        let obs = [small(0, 8), small(0, 8)];
        let exp = [small(1, 8), small(1, 8)];
        let chi2 = crate::stats::chi_squared(&obs, &exp);
        assert!(chi2 >= 0.0, "chi-squared negative");
    }

    /// KANI-INTG-001 (trapezoid): the composite trapezoid rule on f(x) = p x + q over [0, 1] with
    /// 4 panels returns exactly p/2 + q, for every integer p, q in -8..=8 (dyadic nodes, no rounding).
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_trapezoid_linear_exact() {
        let (p, q) = (small(-8, 8), small(-8, 8));
        let got = crate::integrate::trapezoid(|x| p * x + q, 0.0, 1.0, 4);
        assert_eq!(got, p / 2.0 + q, "trapezoid not exact on a line");
    }

    /// KANI-INTG-002: composite Simpson with 2 panels on c3 x^3 + c2 x^2 + c1 x + c0 over [0, 1]
    /// is the exact integral, within 1e-12 of rounding, for every integer coefficient in -4..=4.
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_simpson_cubic_exact() {
        let (c3, c2, c1, c0) = (small(-4, 4), small(-4, 4), small(-4, 4), small(-4, 4));
        let got = crate::integrate::simpson(|x| ((c3 * x + c2) * x + c1) * x + c0, 0.0, 1.0, 2);
        let exact = c3 / 4.0 + c2 / 3.0 + c1 / 2.0 + c0;
        assert!((got - exact).abs() < 1e-12, "simpson not exact on a cubic");
    }

    /// KANI-INTG-003: adaptive quadrature of c x^2 over [0, 1] with tol 1e-6 lands within tol
    /// of c/3, for every integer c in 1..=4.
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_adaptive_convergence() {
        let c = small(1, 4);
        let tol = 1e-6;
        let got = crate::integrate::adaptive_quadrature(|x| c * x * x, 0.0, 1.0, tol);
        assert!(
            (got - c / 3.0).abs() < tol,
            "adaptive quadrature missed its tolerance"
        );
    }

    /// KANI-MATX-001: matmul of an m x p by a p x n matrix is m x n, and C[0,0] is the dot product
    /// of row 0 and column 0, for every shape with m, p, n in 1..=2 and entries in -4..=4. The 8
    /// shapes are enumerated concretely and the entries are symbolic; a symbolic shape makes
    /// every Vec length symbolic, which CBMC could not finish in 30 minutes.
    #[kani::proof]
    #[kani::unwind(9)]
    fn verify_matmul_shape() {
        for shape in 0..8usize {
            let (m, p, n) = (1 + (shape & 1), 1 + (shape >> 1 & 1), 1 + (shape >> 2));
            let a = crate::matrix::Matrix::new(m, p, (0..m * p).map(|_| small(-4, 4)).collect());
            let b = crate::matrix::Matrix::new(p, n, (0..p * n).map(|_| small(-4, 4)).collect());
            let c = crate::matrix::matmul(&a, &b);
            assert_eq!((c.rows(), c.cols()), (m, n), "matmul shape");
            let dot: f64 = (0..p).map(|k| a.get(0, k) * b.get(k, 0)).sum();
            assert_eq!(c.get(0, 0), dot, "matmul entry");
        }
    }

    /// KANI-MATX-002: for every 2 x 2 integer matrix with entries in -1..=1 and det != 0,
    /// inverse returns Some, and |A * inv(A) - I| < 1e-8 entrywise. The 48 such matrices are
    /// enumerated concretely: with symbolic entries, bit-blasting the f64 divisions did not
    /// finish in 30 minutes.
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_inverse_roundtrip() {
        for p in -1..=1i8 {
            for q in -1..=1i8 {
                for r in -1..=1i8 {
                    for s in -1..=1i8 {
                        if p * s == q * r {
                            continue;
                        }
                        let v = [p, q, r, s].map(f64::from).to_vec();
                        let a = crate::matrix::Matrix::new(2, 2, v);
                        let inv = crate::matrix::inverse(&a)
                            .expect("a non-singular matrix has an inverse");
                        let prod = crate::matrix::matmul(&a, &inv);
                        for i in 0..2 {
                            for j in 0..2 {
                                let want = if i == j { 1.0 } else { 0.0 };
                                assert!((prod.get(i, j) - want).abs() < 1e-8, "A * inv(A) != I");
                            }
                        }
                    }
                }
            }
        }
    }

    /// KANI-MATX-003: for every A = L L^T with L 2 x 2 lower triangular, diagonal in 1..=2 and
    /// off-diagonal in -2..=2, cholesky returns Some(L') with |L' L'^T - A| < 1e-10 entrywise.
    /// The 20 factors are enumerated concretely, as for KANI-MATX-002.
    #[kani::proof]
    #[kani::unwind(8)]
    #[kani::stub(f64::sqrt, super::exact_sqrt::sqrt)]
    fn verify_cholesky_roundtrip() {
        for d0 in 1..=2i8 {
            for d1 in 1..=2i8 {
                for o in -2..=2i8 {
                    let (d0, d1, o) = (f64::from(d0), f64::from(d1), f64::from(o));
                    let a = crate::matrix::Matrix::new(
                        2,
                        2,
                        vec![d0 * d0, d0 * o, d0 * o, o * o + d1 * d1],
                    );
                    let l =
                        crate::matrix::cholesky(&a).expect("an SPD matrix has a Cholesky factor");
                    let llt = crate::matrix::matmul(&l, &crate::matrix::transpose(&l));
                    for i in 0..2 {
                        for j in 0..2 {
                            assert!((llt.get(i, j) - a.get(i, j)).abs() < 1e-10, "L L^T != A");
                        }
                    }
                }
            }
        }
    }

    /// Kani models f64 sin and cos as any value in [-1, 1], so no trig identity is provable
    /// through them. This stub is exact where length-2 transforms evaluate trig, at the angles
    /// 0, -0, pi and -pi, returning what libm returns there (sin(pi) is 1.2246e-16, not 0).
    /// Elsewhere it stays as loose as Kani's own model. The FFT proofs below therefore cover
    /// N = 2 only, and they assume libm's values at those four angles.
    fn stub_sin_cos(x: f64) -> (f64, f64) {
        const SIN_PI: f64 = 1.224_646_799_147_353_2e-16;
        if x == 0.0 {
            (x, 1.0)
        } else if x == std::f64::consts::PI {
            (SIN_PI, -1.0)
        } else if x == -std::f64::consts::PI {
            (-SIN_PI, -1.0)
        } else {
            let (s, c): (f64, f64) = (kani::any(), kani::any());
            kani::assume((-1.0..=1.0).contains(&s) && (-1.0..=1.0).contains(&c));
            (s, c)
        }
    }

    /// KANI-FFT-001: fft(x) equals dft(x) within 1e-9 for every length-2 real input with
    /// integer entries in -8..=8.
    #[kani::proof]
    #[kani::unwind(4)]
    #[kani::stub(f64::sin_cos, stub_sin_cos)]
    fn verify_fft_dft_equivalence() {
        let x = [(small(-8, 8), 0.0), (small(-8, 8), 0.0)];
        let (f, d) = (crate::fourier::fft(&x), crate::fourier::dft(&x));
        for k in 0..2 {
            assert!(
                (f[k].0 - d[k].0).abs() < 1e-9 && (f[k].1 - d[k].1).abs() < 1e-9,
                "fft != dft"
            );
        }
    }

    /// KANI-FFT-002: inverse_dft(dft(x)) returns x within 1e-9 for every length-2 real input
    /// with integer entries in -8..=8.
    #[kani::proof]
    #[kani::unwind(4)]
    #[kani::stub(f64::sin_cos, stub_sin_cos)]
    fn verify_idft_roundtrip() {
        let x = [(small(-8, 8), 0.0), (small(-8, 8), 0.0)];
        let back = crate::fourier::inverse_dft(&crate::fourier::dft(&x));
        for k in 0..2 {
            assert!(
                (back[k].0 - x[k].0).abs() < 1e-9 && back[k].1.abs() < 1e-9,
                "idft(dft(x)) != x"
            );
        }
    }

    /// KANI-FFT-003: Parseval, sum |x_n|^2 == sum |X_k|^2 / N within 1e-9, for every length-2
    /// real input with integer entries in -8..=8.
    #[kani::proof]
    #[kani::unwind(4)]
    #[kani::stub(f64::sin_cos, stub_sin_cos)]
    fn verify_parseval_theorem() {
        let x = [(small(-8, 8), 0.0), (small(-8, 8), 0.0)];
        let big = crate::fourier::dft(&x);
        let time: f64 = x.iter().map(|&(r, i)| r * r + i * i).sum();
        let freq: f64 = big.iter().map(|&(r, i)| r * r + i * i).sum::<f64>() / 2.0;
        assert!((time - freq).abs() < 1e-9, "Parseval fails");
    }

    /// Stands in for f32::powi and f64::powi, whose CBMC model leaves the result loose. For n = 2
    /// compiler-rt's __powisf2/__powidf2 return exactly x * x, which is what this returns.
    fn stub_powi_f32(x: f32, n: i32) -> f32 {
        assert_eq!(n, 2, "stub_powi_f32 covers n = 2 only");
        x * x
    }

    /// See stub_powi_f32.
    fn stub_powi_f64(x: f64, n: i32) -> f64 {
        assert_eq!(n, 2, "stub_powi_f64 covers n = 2 only");
        x * x
    }

    /// Stands in for the cpuid instruction (inline asm, which Kani rejects) that std's feature
    /// detection runs when ChaCha seeds. All-zero registers report a CPU without SIMD, so ChaCha
    /// takes its portable path, which by design yields the same stream.
    fn stub_cpuid(_leaf: u32, _sub_leaf: u32) -> std::arch::x86_64::CpuidResult {
        std::arch::x86_64::CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        }
    }

    /// Stands in for aprender's chi-square p-value, an incomplete-gamma series that Kani cannot
    /// unwind. nlib reads only the statistic, which is still computed for real.
    fn stub_chi_square_pvalue(_chi2: f32, _df: usize) -> f32 {
        0.0
    }

    /// Stands in for MonteCarloRng::uniform, whose ChaCha backend runs cpuid (inline asm, which
    /// Kani rejects). It returns any value in [0, 1), so a harness using it covers every stream.
    fn stub_uniform(_rng: &mut aprender::monte_carlo::prelude::MonteCarloRng) -> f64 {
        let u: f64 = kani::any();
        kani::assume((0.0..1.0).contains(&u));
        u
    }

    /// Mean, as a `fn` pointer for bootstrap_error.
    fn mean_stat(x: &[f64]) -> f64 {
        x.iter().sum::<f64>() / x.len() as f64
    }

    /// KANI-MC-001: the bootstrap standard error is >= 0 for every 2-point sample with integer
    /// entries in -2..=2 (4 resamples, seed 42). The 25 samples are enumerated concretely, as
    /// are the inputs of every harness below whose domain is a few integers: symbolic f64 inputs
    /// through a sqrt, a division or a 30-step loop do not finish in CBMC.
    #[kani::proof]
    #[kani::unwind(18)]
    #[kani::stub(f64::sqrt, super::exact_sqrt::sqrt)]
    #[kani::stub(f64::powi, stub_powi_f64)]
    fn verify_bootstrap_nonneg() {
        for p in -2..=2i8 {
            for q in -2..=2i8 {
                let data = [f64::from(p), f64::from(q)];
                let se = crate::monte_carlo::bootstrap_error(&data, mean_stat, 4, 42);
                assert!(se >= 0.0, "bootstrap standard error negative");
            }
        }
    }

    /// KANI-MC-002: on a constant integrand c over [0, 2], mc_integrate returns exactly 2c for
    /// every integer c in -8..=8 (4 samples, seed 7). Unbiasedness in general is a statistical
    /// property of the stream; this is the one instance where every sample is exact.
    #[kani::proof]
    #[kani::unwind(18)]
    #[kani::stub(aprender::monte_carlo::prelude::MonteCarloRng::uniform, stub_uniform)]
    #[kani::stub(std::arch::x86_64::__cpuid_count, stub_cpuid)]
    fn verify_mc_unbiased() {
        for c in (-8..=8i8).map(f64::from) {
            let got = crate::monte_carlo::mc_integrate(|_| c, 0.0, 2.0, 4, 7);
            assert_eq!(got, 2.0 * c, "mc_integrate biased on a constant");
        }
    }

    /// KANI-MC-003: detailed balance of the Metropolis acceptance rule with a symmetric proposal,
    /// p(x) min(1, p(y)/p(x)) == p(y) min(1, p(x)/p(y)), for every pair of weights in 1..=8.
    /// nlib has no Metropolis sampler, so this proves the rule the contract states, not code in src/.
    #[kani::proof]
    #[kani::unwind(9)]
    fn verify_metropolis_balance() {
        let accept = |from: f64, to: f64| (to / from).min(1.0);
        for px in (1..=8i8).map(f64::from) {
            for py in (1..=8i8).map(f64::from) {
                let lhs = px * accept(px, py);
                let rhs = py * accept(py, px);
                assert!((lhs - rhs).abs() < 1e-12, "detailed balance fails");
            }
        }
    }

    /// KANI-SOLV-001: bisection on f(x) = x - r over [0, 4] with tol 1e-6 returns a point with
    /// |f| < tol, for every integer root r in 1..=3.
    #[kani::proof]
    #[kani::unwind(30)]
    fn verify_bisection_convergence() {
        let tol = 1e-6;
        for r in (1..=3i8).map(f64::from) {
            let root = crate::solve::bisection(|x| x - r, 0.0, 4.0, tol);
            assert!((root - r).abs() < tol, "bisection missed the root");
        }
    }

    /// KANI-SOLV-002: newton on f(x) = x^2 - c from x0 = c with tol 1e-10 returns |f| < tol,
    /// for every integer c in 2..=4.
    #[kani::proof]
    #[kani::unwind(12)]
    fn verify_newton_convergence() {
        let tol = 1e-10;
        for c in (2..=4i8).map(f64::from) {
            let root = crate::solve::newton(|x| x * x - c, |x| 2.0 * x, c, tol);
            assert!((root * root - c).abs() < tol, "newton did not converge");
        }
    }

    /// KANI-SOLV-003: bisection on [0, 1] halves the bracket every step: the k-th midpoint it
    /// evaluates moves by exactly 2^-(k+1) from the one before (f(x) = x - 1/3, tol 1e-4).
    #[kani::proof]
    #[kani::unwind(20)]
    fn verify_bisection_interval() {
        let seen = std::cell::RefCell::new(Vec::new());
        crate::solve::bisection(
            |x| {
                seen.borrow_mut().push(x);
                x - 1.0 / 3.0
            },
            0.0,
            1.0,
            1e-4,
        );
        let mut pts = seen.into_inner();
        // pts[0] = a, pts[1] = b, then one midpoint per step; with debug assertions on, the
        // postcondition evaluates f at the returned midpoint once more.
        if pts.len() > 3 && pts[pts.len() - 1] == pts[pts.len() - 2] {
            pts.pop();
        }
        assert!(pts.len() > 4);
        let mut step = 0.25;
        for k in 3..pts.len() {
            assert_eq!((pts[k] - pts[k - 1]).abs(), step, "bracket did not halve");
            step /= 2.0;
        }
    }

    /// KANI-OPT-001: golden_section on (x - c)^2 over [0, 4] with tol 1e-3 returns a point within
    /// tol of the minimizer c, for every integer c in 1..=3.
    #[kani::proof]
    #[kani::unwind(20)]
    #[kani::stub(f64::sqrt, super::exact_sqrt::sqrt)]
    fn verify_golden_section_convergence() {
        let tol = 1e-3;
        for c in (1..=3i8).map(f64::from) {
            let x = crate::optimize::golden_section(|x| (x - c) * (x - c), 0.0, 4.0, tol);
            assert!((x - c).abs() < tol, "golden section missed the minimizer");
        }
    }

    /// KANI-OPT-002: gradient_descent on f(x) = x^2 from x0 = 1 (lr 0.1, tol 1e-2) never
    /// increases f from one iterate to the next.
    #[kani::proof]
    #[kani::unwind(30)]
    #[kani::stub(f64::sqrt, super::exact_sqrt::sqrt)]
    fn verify_gradient_descent_monotone() {
        let seen = std::cell::RefCell::new(Vec::new());
        crate::optimize::gradient_descent(
            |x| x[0] * x[0],
            |x| {
                seen.borrow_mut().push(x[0]);
                vec![2.0 * x[0]]
            },
            &[1.0],
            0.1,
            1e-2,
        );
        let xs = seen.into_inner();
        assert!(xs.len() > 1);
        for k in 1..xs.len() {
            assert!(xs[k] * xs[k] <= xs[k - 1] * xs[k - 1], "f increased");
        }
    }

    /// KANI-OPT-003: newton_optimize on f(x) = x^3/3 - c x from x0 = c with tol 1e-10 returns
    /// |f'(x*)| < tol, for every integer c in 2..=4.
    #[kani::proof]
    #[kani::unwind(12)]
    fn verify_newton_opt_convergence() {
        let tol = 1e-10;
        for c in (2..=4i8).map(f64::from) {
            let x = crate::optimize::newton_optimize(
                |x| x * x * x / 3.0 - c * x,
                |x| x * x - c,
                |x| 2.0 * x,
                c,
                tol,
            );
            assert!((x * x - c).abs() < tol, "newton_optimize did not converge");
        }
    }

    /// KANI-RNG-001: every LCG output is < m, for every m in 2..=1000, a in (0, m), c < m and
    /// seed < m (two steps).
    #[kani::proof]
    fn verify_lcg_range() {
        let (m, a, c, seed): (u16, u16, u16, u16) =
            (kani::any(), kani::any(), kani::any(), kani::any());
        kani::assume((2..=1000).contains(&m) && a > 0 && a < m && c < m && seed < m);
        let mut g =
            crate::random::Lcg::new(u64::from(seed), u64::from(a), u64::from(c), u64::from(m));
        assert!(g.next_val() < u64::from(m), "lcg output >= m");
        assert!(g.next_val() < u64::from(m), "lcg output >= m");
    }

    /// KANI-RNG-002: two MINSTD generators with the same seed give the same first two values,
    /// for every seed in 1..2^31 - 1.
    #[kani::proof]
    fn verify_lcg_deterministic() {
        let m = 2_147_483_647u64;
        let seed: u32 = kani::any();
        kani::assume(seed > 0 && u64::from(seed) < m);
        let mut g1 = crate::random::Lcg::new(u64::from(seed), 16807, 0, m);
        let mut g2 = crate::random::Lcg::new(u64::from(seed), 16807, 0, m);
        assert_eq!(g1.next_val(), g2.next_val());
        assert_eq!(g1.next_val(), g2.next_val());
    }

    /// KANI-RNG-003: MT19937 seeded with 5489 (the reference seed) yields next_f64 in [0, 1) for
    /// its first two draws; a u32 is < 2^32 by type, so the check that matters is the f64 map.
    #[kani::proof]
    #[kani::unwind(626)]
    fn verify_mt_range() {
        let mut mt = crate::random::Mt19937::new(5489);
        for _ in 0..2 {
            let u = mt.next_f64();
            assert!((0.0..1.0).contains(&u), "mt next_f64 outside [0, 1)");
        }
    }
}

/// IEEE 754 square root, correctly rounded to nearest, in integer arithmetic only. CBMC models
/// f64::sqrt with a symbolic value it must solve for, which makes even a concrete sqrt cost
/// millions of SAT variables; integer steps on concrete values fold to constants instead. The
/// test below checks this against f64::sqrt, so a harness that stubs sqrt with it still proves
/// the real code.
#[cfg(any(kani, test))]
mod exact_sqrt {
    /// sqrt(x) for x = +0 or a positive normal finite x; asserts on any other input.
    pub(crate) fn sqrt(x: f64) -> f64 {
        if x == 0.0 {
            return x;
        }
        assert!(
            x.is_normal() && x > 0.0,
            "exact_sqrt: input outside the modelled range"
        );
        let bits = x.to_bits();
        // x = m * 2^e with m a 53-bit integer.
        let mut e = ((bits >> 52) & 0x7ff) as i64 - 1075;
        let mut m = u128::from((bits & ((1 << 52) - 1)) | (1 << 52));
        if e % 2 != 0 {
            m <<= 1;
            e -= 1;
        }
        // sqrt(x) = sqrt(n) * 2^((e - 54) / 2) with n = m * 2^54 in [2^106, 2^108).
        let n = m << 54;
        let mut r: u128 = 1 << 55;
        loop {
            let next = (r + n / r) / 2;
            if next >= r {
                break;
            }
            r = next;
        }
        // r = floor(sqrt(n)) has 54 bits: keep 53 and round to nearest, ties to even.
        let (mut q, guard, sticky) = (r >> 1, r & 1 == 1, r * r != n);
        let mut k = (e - 54) / 2 + 1;
        if guard && (sticky || q & 1 == 1) {
            q += 1;
        }
        if q == 1 << 53 {
            q >>= 1;
            k += 1;
        }
        // sqrt(x) = q * 2^k with q in [2^52, 2^53).
        let exp = (k + 52 + 1023) as u64;
        f64::from_bits((exp << 52) | (q as u64 & ((1 << 52) - 1)))
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn matches_std_sqrt() {
            let mut s: u64 = 0x9e37_79b9_7f4a_7c15;
            let mut xs = vec![
                1.0,
                2.0,
                4.0,
                5.0,
                0.5,
                1e-300,
                1e300,
                f64::MAX,
                f64::MIN_POSITIVE,
            ];
            for _ in 0..200_000 {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let x = f64::from_bits(s >> 1);
                if x.is_normal() {
                    xs.push(x);
                }
            }
            for i in 1..=10_000 {
                xs.push(f64::from(i));
                xs.push(f64::from(i) * f64::from(i));
            }
            assert!(xs.len() > 200_000, "only {} inputs", xs.len());
            for x in xs {
                assert_eq!(super::sqrt(x).to_bits(), x.sqrt().to_bits(), "sqrt({x:e})");
            }
        }
    }
}
