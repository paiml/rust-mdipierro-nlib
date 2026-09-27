-- Theorem: correlation_bounded
-- Property: Correlation bounded
-- Obligation type: bound
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Descriptive

namespace ProvableContracts.Descriptive

-- Formal: -1.0 <= correlation(x, y) <= 1.0
/-- `|cov(x, y)| ≤ σ_x σ_y`, the Cauchy–Schwarz inequality on the centred data. -/
theorem abs_covariance_le {n : ℕ} (x y : Fin n → ℝ) :
    |covariance x y| ≤ stdDev x * stdDev y := by
  have hv : ∀ z : Fin n → ℝ, 0 ≤ variance z := fun z =>
    div_nonneg (Finset.sum_nonneg fun _ _ => sq_nonneg _) (Nat.cast_nonneg n)
  have hs : 0 ≤ stdDev x * stdDev y := mul_nonneg (Real.sqrt_nonneg _) (Real.sqrt_nonneg _)
  apply abs_le_of_sq_le_sq _ hs
  have cs := Finset.sum_mul_sq_le_sq_mul_sq Finset.univ (fun i => x i - mean x) (fun i => y i - mean y)
  rw [mul_pow, stdDev, stdDev, Real.sq_sqrt (hv x), Real.sq_sqrt (hv y)]
  unfold covariance variance
  rw [div_pow, div_mul_div_comm, ← sq]
  exact div_le_div_of_nonneg_right cs (sq_nonneg _)

theorem correlation_bounded {n : ℕ} (x y : Fin n → ℝ) :
    -1 ≤ correlation x y ∧ correlation x y ≤ 1 := by
  unfold correlation
  split_ifs with h
  · norm_num
  · rw [not_or] at h
    have hp : 0 < stdDev x * stdDev y :=
      mul_pos (lt_of_le_of_ne (Real.sqrt_nonneg _) (Ne.symm h.1)) (lt_of_le_of_ne (Real.sqrt_nonneg _) (Ne.symm h.2))
    have hb := abs_le.mp (abs_covariance_le x y)
    constructor
    · rw [le_div_iff₀ hp]; linarith
    · rw [div_le_iff₀ hp]; linarith

end ProvableContracts.Descriptive
