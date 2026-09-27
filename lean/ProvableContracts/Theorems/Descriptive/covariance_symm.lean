-- Theorem: covariance_symm
-- Property: Covariance symmetry
-- Obligation type: invariant
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Descriptive

namespace ProvableContracts.Descriptive

-- Formal: cov(x, y) == cov(y, x)
theorem covariance_symm {n : ℕ} (x y : Fin n → ℝ) : covariance x y = covariance y x := by
  unfold covariance
  congr 1
  exact Finset.sum_congr rfl fun _ _ => mul_comm _ _

end ProvableContracts.Descriptive
