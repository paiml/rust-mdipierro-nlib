-- Theorem: trapezoid_exact_linear
-- Property: Trapezoid exact for linear functions
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Numerical

namespace ProvableContracts.Numerical

open Finset

-- Formal: trapezoid(a*x+b, lo, hi, n) == exact for all n >= 1
/-- For every `n ≥ 1` panels, the trapezoid rule on `c x + d` equals the exact integral over `[a, b]`. -/
theorem trapezoid_exact_linear (c d a b : ℝ) {n : ℕ} (hn : 1 ≤ n) :
    trapezoid (fun x => c * x + d) a b n = ∫ x in a..b, c * x + d := by
  set F : ℝ → ℝ := fun x => c * x ^ 2 / 2 + d * x with hF
  have hint : ∫ x in a..b, c * x + d = F b - F a := by
    refine intervalIntegral.integral_eq_sub_of_hasDerivAt (fun x _ => ?_)
      ((by fun_prop : Continuous fun x : ℝ => c * x + d).intervalIntegrable a b)
    have := (HasDerivAt.const_mul (c / 2) (hasDerivAt_pow 2 x)).add (HasDerivAt.const_mul d (hasDerivAt_id x))
    convert this using 1
    · funext y; simp [hF]; ring
    · simp; ring
  set h := (b - a) / n with hh
  have key : ∀ N : ℕ, 1 ≤ N → ((c * a + d + (c * (a + N * h) + d)) / 2 +
      ∑ i ∈ Ico 1 N, (c * (a + i * h) + d)) * h = F (a + N * h) - F a := by
    intro N hN
    induction N, hN using Nat.le_induction with
    | base => simp [hF]; ring
    | succ N hN ih =>
      rw [Finset.sum_Ico_succ_top (by omega : 1 ≤ N)]
      push_cast
      simp only [hF] at ih ⊢
      linear_combination ih
  have hb : a + (n : ℝ) * h = b := by
    have : (n : ℝ) ≠ 0 := by exact_mod_cast (show n ≠ 0 by omega)
    rw [hh]; field_simp; ring
  rw [hint, trapezoid, ← hh]
  have := key n hn
  rw [hb] at this
  exact this

end ProvableContracts.Numerical
