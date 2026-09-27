-- Theorem: mean_within_range
-- Property: Mean within data range
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Descriptive

namespace ProvableContracts.Descriptive

-- Formal: min(x) <= mean(x) <= max(x)
/-- For non-empty data (the Rust asserts `n > 0`), the mean lies between the minimum and maximum. -/
theorem mean_within_range {n : ℕ} (hn : 0 < n) (x : Fin n → ℝ) :
    Finset.univ.inf' (Finset.univ_nonempty_iff.mpr ⟨⟨0, hn⟩⟩) x ≤ mean x ∧
      mean x ≤ Finset.univ.sup' (Finset.univ_nonempty_iff.mpr ⟨⟨0, hn⟩⟩) x := by
  have hn' : (0 : ℝ) < n := by exact_mod_cast hn
  set lo := Finset.univ.inf' (Finset.univ_nonempty_iff.mpr ⟨⟨0, hn⟩⟩) x
  set hi := Finset.univ.sup' (Finset.univ_nonempty_iff.mpr ⟨⟨0, hn⟩⟩) x
  have hlo : ∀ i ∈ (Finset.univ : Finset (Fin n)), lo ≤ x i :=
    fun i hi' => Finset.inf'_le _ hi'
  have hhi : ∀ i ∈ (Finset.univ : Finset (Fin n)), x i ≤ hi :=
    fun i hi' => Finset.le_sup' _ hi'
  have s1 := Finset.card_nsmul_le_sum _ _ _ hlo
  have s2 := Finset.sum_le_card_nsmul _ _ _ hhi
  simp only [Finset.card_univ, Fintype.card_fin, nsmul_eq_mul] at s1 s2
  unfold mean
  constructor
  · rw [le_div_iff₀ hn']; linarith
  · rw [div_le_iff₀ hn']; linarith

end ProvableContracts.Descriptive
