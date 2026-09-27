-- Theorem: transpose_transpose
-- The theorem the `transpose` equation of matrix-algebra-v1 names: `(Aᵀ)ᵀ == A`, with the shape swap.
-- Written by hand (the equation carries no obligation of its own, so `pv lean` makes no stub).

import ProvableContracts.Defs.Dense

namespace ProvableContracts.Dense

variable {α : Type*}

/-- `Aᵀ[j, i] = A[i, j]` for every in-range `(i, j)`: the index arithmetic of the Rust loop is right. -/
theorem transpose_get (a : Mat α) {i j : ℕ} (hi : i < a.rows) :
    (transpose a).get j i = a.get i j := by
  simp only [Mat.get, transpose]
  have hr : 0 < a.rows := lt_of_le_of_lt (Nat.zero_le i) hi
  rw [Nat.add_comm (j * a.rows) i, Nat.add_mul_mod_self_right, Nat.mod_eq_of_lt hi,
    Nat.add_mul_div_right _ _ hr, Nat.div_eq_of_lt hi, Nat.zero_add]

/-- The transpose of an `m × n` matrix is `n × m`. -/
theorem transpose_shape (a : Mat α) : (transpose a).rows = a.cols ∧ (transpose a).cols = a.rows :=
  ⟨rfl, rfl⟩

/-- `(Aᵀ)ᵀ == A`: the same shape, and the same entry at every in-range `(i, j)`. -/
theorem transpose_transpose (a : Mat α) :
    (transpose (transpose a)).rows = a.rows ∧ (transpose (transpose a)).cols = a.cols ∧
      ∀ i j, i < a.rows → j < a.cols → (transpose (transpose a)).get i j = a.get i j := by
  refine ⟨rfl, rfl, fun i j hi hj => ?_⟩
  rw [transpose_get (transpose a) (show j < (transpose a).rows from hj), transpose_get a hi]

end ProvableContracts.Dense
