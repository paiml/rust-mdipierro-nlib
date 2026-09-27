-- Descriptive statistics — contract statistics-v1, code src/stats.rs.
-- Layout from `pv lean contracts/statistics-v1.yaml`; the definitions are written by hand.
--
-- The model is exact: data are `Fin n → ℝ` and every operation is real arithmetic. The Rust computes
-- the same formulas in f64, with rounding; docs/lean-proofs.md states what that gap leaves unproved.

import Mathlib.Analysis.Real.Sqrt
import Mathlib.Algebra.Order.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.BigOperators
import Mathlib.Tactic.Linarith

namespace ProvableContracts.Descriptive

open Finset

variable {n : ℕ}

/-- `mean`: `(Σ x_i) / n`. -/
noncomputable def mean (x : Fin n → ℝ) : ℝ := (∑ i, x i) / n

/-- `variance`: the population variance `(Σ (x_i - μ)²) / n`, as src/stats.rs computes it. -/
noncomputable def variance (x : Fin n → ℝ) : ℝ := (∑ i, (x i - mean x) ^ 2) / n

/-- The Bessel-corrected `(Σ (x_i - μ)²) / (n - 1)` the contract's formula text names. -/
noncomputable def sampleVariance (x : Fin n → ℝ) : ℝ := (∑ i, (x i - mean x) ^ 2) / (n - 1 : ℝ)

/-- `std_dev`: `√variance`. -/
noncomputable def stdDev (x : Fin n → ℝ) : ℝ := Real.sqrt (variance x)

/-- `covariance`: `(Σ (x_i - μ_x)(y_i - μ_y)) / n`. -/
noncomputable def covariance (x y : Fin n → ℝ) : ℝ := (∑ i, (x i - mean x) * (y i - mean y)) / n

/-- `correlation`: `cov / (σ_x σ_y)`, and `0` when either deviation is `0`, as the Rust returns. -/
noncomputable def correlation (x y : Fin n → ℝ) : ℝ :=
  if stdDev x = 0 ∨ stdDev y = 0 then 0 else covariance x y / (stdDev x * stdDev y)

/-- `chi_squared`: `Σ (O_i - E_i)² / E_i`. -/
noncomputable def chiSquared (o e : Fin n → ℝ) : ℝ := ∑ i, (o i - e i) ^ 2 / e i

end ProvableContracts.Descriptive
