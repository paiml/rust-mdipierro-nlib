-- Numerical quadrature — contract integration-v1, code src/integrate.rs.
-- Layout from `pv lean contracts/integration-v1.yaml`; the definitions are written by hand.
--
-- The rules are the Rust formulas over ℝ: `h = (b - a) / n`, nodes `a + i * h`, and the same sums
-- in the same order of terms. The Rust evaluates them in f64; docs/lean-proofs.md states the gap.

import Mathlib.MeasureTheory.Integral.IntervalIntegral.FundThmCalculus
import Mathlib.Analysis.Calculus.Deriv.Pow
import Mathlib.Analysis.Calculus.Deriv.Mul
import Mathlib.Tactic.LinearCombination
import Mathlib.Tactic.FieldSimp

namespace ProvableContracts.Numerical

open Finset

/-- `trapezoid`: `(f(a)/2 + f(b)/2 + Σ_{i=1}^{n-1} f(a + i h)) · h`, `h = (b - a) / n`. -/
noncomputable def trapezoid (f : ℝ → ℝ) (a b : ℝ) (n : ℕ) : ℝ :=
  ((f a + f b) / 2 + ∑ i ∈ Ico 1 n, f (a + i * ((b - a) / n))) * ((b - a) / n)

/-- `simpson`: `(f(a) + f(b) + Σ_{i=1}^{n-1} w_i f(a + i h)) · h / 3`, `w_i = 2` for even `i`, else
`4`, `h = (b - a) / n`; the Rust asserts `n` even and positive. -/
noncomputable def simpson (f : ℝ → ℝ) (a b : ℝ) (n : ℕ) : ℝ :=
  (f a + f b + ∑ i ∈ Ico 1 n, (if i % 2 = 0 then (2 : ℝ) else 4) * f (a + i * ((b - a) / n))) *
    ((b - a) / n) / 3

end ProvableContracts.Numerical
