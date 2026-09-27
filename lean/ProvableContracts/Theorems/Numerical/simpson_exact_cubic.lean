-- Theorem: simpson_exact_cubic
-- Property: Simpson exact for cubics
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Numerical

namespace ProvableContracts.Numerical

open Finset

-- Formal: simpson(a*x^3+b*x^2+c*x+d, lo, hi, n) == exact for all even n >= 2
/-- For every even `n = 2m ≥ 2` panels, Simpson's rule on a cubic equals the exact integral over `[a, b]`. -/
theorem simpson_exact_cubic (c3 c2 c1 c0 a b : ℝ) {m : ℕ} (hm : 1 ≤ m) :
    simpson (fun x => c3 * x ^ 3 + c2 * x ^ 2 + c1 * x + c0) a b (2 * m) =
      ∫ x in a..b, c3 * x ^ 3 + c2 * x ^ 2 + c1 * x + c0 := by
  set p : ℝ → ℝ := fun x => c3 * x ^ 3 + c2 * x ^ 2 + c1 * x + c0 with hp
  set F : ℝ → ℝ := fun x => c3 * x ^ 4 / 4 + c2 * x ^ 3 / 3 + c1 * x ^ 2 / 2 + c0 * x with hF
  have hint : ∫ x in a..b, p x = F b - F a := by
    refine intervalIntegral.integral_eq_sub_of_hasDerivAt (fun x _ => ?_)
      ((by fun_prop : Continuous p).intervalIntegrable a b)
    have := (((HasDerivAt.const_mul (c3 / 4) (hasDerivAt_pow 4 x)).add (HasDerivAt.const_mul (c2 / 3) (hasDerivAt_pow 3 x))).add
      (HasDerivAt.const_mul (c1 / 2) (hasDerivAt_pow 2 x))).add (HasDerivAt.const_mul c0 (hasDerivAt_id x))
    convert this using 1
    · funext y; simp [hF]; ring
    · simp [hp]; ring
  set h := (b - a) / (2 * m : ℕ) with hh
  have key : ∀ M, 1 ≤ M → (p a + p (a + (2 * M : ℕ) * h) +
      ∑ i ∈ Ico 1 (2 * M), (if i % 2 = 0 then (2 : ℝ) else 4) * p (a + i * h)) * h / 3 =
      F (a + (2 * M : ℕ) * h) - F a := by
    intro M hM
    induction M, hM using Nat.le_induction with
    | base => simp [hp, hF]; ring
    | succ M hM ih =>
      have e : 2 * (M + 1) = 2 * M + 1 + 1 := by ring
      rw [e, Finset.sum_Ico_succ_top (by omega : 1 ≤ 2 * M + 1), Finset.sum_Ico_succ_top (by omega : 1 ≤ 2 * M)]
      have h0 : (2 * M) % 2 = 0 := by omega
      have h1 : (2 * M + 1) % 2 = 1 := by omega
      simp only [h0, h1, ite_true, one_ne_zero, ite_false]
      push_cast at ih ⊢
      simp only [hp, hF] at ih ⊢
      linear_combination ih
  have hb : a + ((2 * m : ℕ) : ℝ) * h = b := by
    have : ((2 * m : ℕ) : ℝ) ≠ 0 := by exact_mod_cast (show 2 * m ≠ 0 by omega)
    rw [hh]; field_simp; ring
  show simpson p a b (2 * m) = ∫ x in a..b, p x
  rw [hint, simpson, ← hh]
  have := key m hm
  rw [hb] at this
  exact this

theorem planted_gap : (1 : ℕ) = 2 := by sorry

end ProvableContracts.Numerical
