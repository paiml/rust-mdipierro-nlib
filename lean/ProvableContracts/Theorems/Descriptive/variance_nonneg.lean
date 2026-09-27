-- Theorem: variance_nonneg
-- Property: Variance non-negative
-- Obligation type: bound
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Descriptive

namespace ProvableContracts.Descriptive

-- Formal: variance(x) >= 0.0 for all x with n > 1
/-- The population variance is never negative (for every `n`, not only `n > 1`). -/
theorem variance_nonneg {n : ℕ} (x : Fin n → ℝ) : 0 ≤ variance x :=
  div_nonneg (Finset.sum_nonneg fun _ _ => sq_nonneg _) (Nat.cast_nonneg n)

/-- The Bessel-corrected variance of the contract's formula text is never negative for `n ≥ 1`. -/
theorem sampleVariance_nonneg {n : ℕ} (hn : 1 ≤ n) (x : Fin n → ℝ) : 0 ≤ sampleVariance x := by
  have : (1 : ℝ) ≤ n := by exact_mod_cast hn
  exact div_nonneg (Finset.sum_nonneg fun _ _ => sq_nonneg _) (by linarith)

end ProvableContracts.Descriptive
