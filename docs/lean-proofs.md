# Lean proofs (GH-5)

The theorems under [`lean/ProvableContracts/Theorems`](../lean/ProvableContracts/Theorems) are checked by
Lean 4. [`lean/lean-toolchain`](../lean/lean-toolchain) pins the toolchain, and
[`lean/lake-manifest.json`](../lean/lake-manifest.json) pins Mathlib to the release tag of the same version.
CI's **Lean Proofs** job runs [`scripts/lean_gate.sh`](../scripts/lean_gate.sh), and `gate` needs that job.
The gate fails when:

- a proof does not check (`lake build`; the lakefile makes every warning an error, `sorry` included);
- a Lean source holds `sorry`, `admit`, a new `axiom` or `native_decide`;
- a contract's `lean_theorem:` or obligation `module:` names a theorem that does not exist;
- a theorem file is not imported by [`lean/ProvableContracts.lean`](../lean/ProvableContracts.lean), so `lake build` would never check it.

`pv proof-status` credits an equation with L4 when its `lean_theorem:` names one of these theorems.
[`scripts/proof_ratchet.sh`](../scripts/proof_ratchet.sh) fails the contracts gate if the number of
grounded theorems falls below `lean_grounded` in `contracts/proof-baseline.json`.

## What is proved, and what is not

**Every theorem here is about an exact model, not about the f64 Rust code.** Each model is written by hand
in [`lean/ProvableContracts/Defs`](../lean/ProvableContracts/Defs), and its header names the Rust function
it follows. No step checks mechanically that the Rust code matches its model: the golden vectors, parity,
Kani and the tests are the evidence for that. For the real-valued theorems, rounding is a second gap.

| Theorem | Model | Statement | Gap to the Rust code |
|---------|-------|-----------|----------------------|
| [`quicksort_correct`](../lean/ProvableContracts/Theorems/Sorting/quicksort_correct.lean), [`quicksort_sorted`](../lean/ProvableContracts/Theorems/Sorting/quicksort_sorted.lean), [`quicksort_perm`](../lean/ProvableContracts/Theorems/Sorting/quicksort_perm.lean) | Lomuto quicksort on a `List` over any linear order; the last element is the pivot, `≤ pivot` goes left | The output is sorted, is a permutation of the input, and has the same length | The Rust sorts a slice in place with swaps. The model returns the list the partition leaves, and does not model the order of elements within each side. No rounding is involved, because only comparisons are made. For `f64`, the Rust needs `T: Ord`, so NaN never reaches it. |
| [`transpose_transpose`](../lean/ProvableContracts/Theorems/Dense/transpose_transpose.lean) | Row-major `Mat α` with a buffer indexed by `i * cols + j`, generic in the entry type | Transposing twice gives back the shape and every in-range entry | Entries are only moved, never computed, so none is rounded. The model's buffer is a function `ℕ → α`, not a `Vec` with a length. |
| [`variance_nonneg`](../lean/ProvableContracts/Theorems/Descriptive/variance_nonneg.lean) (and `sampleVariance_nonneg`) | `Fin n → ℝ`, the population formula `Σ(x−μ)²/n` as `src/stats.rs` computes it | `0 ≤ variance x`; the Bessel form is `≥ 0` for `n ≥ 1` | f64 sums round; a sum of squares stays `≥ 0` in f64 too, but that is not what Lean proves. |
| [`mean_within_range`](../lean/ProvableContracts/Theorems/Descriptive/mean_within_range.lean) | `Fin n → ℝ`, `n > 0` | `min x ≤ mean x ≤ max x` | In f64, the rounded mean of nearly equal values can fall one ulp outside that range; the real-valued theorem does not rule this out. |
| [`covariance_symm`](../lean/ProvableContracts/Theorems/Descriptive/covariance_symm.lean) | `Fin n → ℝ` | `cov(x, y) = cov(y, x)` | Real multiplication commutes, and so does f64 multiplication. The theorem does not cover how the f64 sum is ordered. |
| [`correlation_bounded`](../lean/ProvableContracts/Theorems/Descriptive/correlation_bounded.lean) | `Fin n → ℝ`; 0 when either deviation is 0, as the Rust returns | `−1 ≤ corr ≤ 1` (via `abs_covariance_le`, Cauchy–Schwarz) | In f64, a rounded `cov / (σx σy)` can exceed 1 by an ulp; the tests carry a tolerance for this. |
| [`chi_squared_nonneg`](../lean/ProvableContracts/Theorems/Descriptive/chi_squared_nonneg.lean) | `Fin n → ℝ`, every expected count `> 0` | `0 ≤ χ²` | The precondition `E_i > 0` is the contract's. The Rust asserts it only under `debug_assert!`. |
| [`simpson_exact_cubic`](../lean/ProvableContracts/Theorems/Numerical/simpson_exact_cubic.lean) | The Rust's composite Simpson over ℝ: `h = (b−a)/n`, nodes `a + i h`, weights 4/2 | For every even `n = 2m ≥ 2`, Simpson on a cubic equals the exact integral | "Exact" holds over ℝ only. The f64 result carries rounding that grows with `n`, and the golden vectors check it within a tolerance. |
| [`trapezoid_exact_linear`](../lean/ProvableContracts/Theorems/Numerical/trapezoid_exact_linear.lean) | The Rust's composite trapezoid over ℝ | For every `n ≥ 1`, the trapezoid rule on a line equals the exact integral | The same gap: exact over ℝ, within a tolerance in f64. |

## Out of scope, with the reason

| Example | Status | Reason |
|---------|--------|--------|
| fourier | N/A | Its claims (a roundtrip within 1e-10, Parseval) are floating-point tolerances. Over ℝ they are textbook DFT facts, not facts about this code. |
| random | N/A | The integer recurrences are pinned by golden vectors and Kani. Uniformity is a statistical property, not a theorem. |
| monte_carlo | N/A | The error bound is a probabilistic statement about a pseudo-random stream. Tests falsify it; nothing proves it. |
| solve, optimize, graph, parity | not proved yet | Proving convergence of the root finders and optimizers, or the correctness of the graph algorithms, is future work. |

The README's Lean column is generated from [`contracts/example-lean.tsv`](../contracts/example-lean.tsv).
