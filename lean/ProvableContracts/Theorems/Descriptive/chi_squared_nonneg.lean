-- Theorem: chi_squared_nonneg
-- Property: Chi-squared non-negative
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Descriptive

namespace ProvableContracts.Descriptive

-- Formal: chi_squared(O, E) >= 0.0
/-- Under the Rust's precondition `E_i > 0` (it asserts it), the statistic is never negative. -/
theorem chi_squared_nonneg {n : ℕ} (o e : Fin n → ℝ) (he : ∀ i, 0 < e i) : 0 ≤ chiSquared o e :=
  Finset.sum_nonneg fun i _ => div_nonneg (sq_nonneg _) (he i).le

end ProvableContracts.Descriptive
