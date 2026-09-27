-- Theorem: quicksort_sorted
-- Property: Output is sorted
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Theorems.Sorting.quicksort_perm

namespace ProvableContracts.Sorting

variable {α : Type*} [LinearOrder α]

-- Formal: forall i in 0..n-1: result[i] <= result[i+1]
/-- Every element is `≤` every later one (`Pairwise`), which gives `result[i] ≤ result[i+1]`. -/
theorem quicksort_sorted (l : List α) : (quicksort l).Pairwise (· ≤ ·) := by
  induction l using quicksort.induct with
  | case1 l h => rw [quicksort_nil_of l h]; exact List.Pairwise.nil
  | case2 l p h _ ih1 ih2 =>
    rw [quicksort_of_getLast l p h]
    have hl : ∀ a ∈ quicksort (l.dropLast.filter (· ≤ p)), a ≤ p := fun a ha => by
      have := (quicksort_perm _).mem_iff.mp ha
      simpa using (List.mem_filter.mp this).2
    have hr : ∀ b ∈ quicksort (l.dropLast.filter (fun x => ¬ x ≤ p)), p ≤ b := fun b hb => by
      have := (quicksort_perm _).mem_iff.mp hb
      have := (List.mem_filter.mp this).2
      simp at this
      exact this.le
    refine List.pairwise_append.mpr ⟨ih1, List.pairwise_cons.mpr ⟨hr, ih2⟩, ?_⟩
    intro a ha b hb
    rcases List.mem_cons.mp hb with rfl | hb
    · exact hl a ha
    · exact le_trans (hl a ha) (hr b hb)


end ProvableContracts.Sorting
