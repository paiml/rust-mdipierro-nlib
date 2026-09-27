-- Theorem: quicksort_perm
-- Property: Output is a permutation of input
-- Obligation type: postcondition
-- Stub generated with `pv lean`; the statement and proof are written by hand.

import ProvableContracts.Defs.Sorting

namespace ProvableContracts.Sorting

variable {α : Type*} [LinearOrder α]

-- Formal: multiset(output) == multiset(input)
theorem quicksort_perm (l : List α) : (quicksort l).Perm l := by
  induction l using quicksort.induct with
  | case1 l h => rw [quicksort_nil_of l h, List.getLast?_eq_none_iff.mp h]
  | case2 l p h _ ih1 ih2 =>
    rw [quicksort_of_getLast l p h]
    have hF : (l.dropLast.filter (fun x => decide (x ≤ p)) ++
        l.dropLast.filter (fun x => decide ¬ x ≤ p)).Perm l.dropLast := by
      simpa only [decide_not] using List.filter_append_perm (fun x => decide (x ≤ p)) l.dropLast
    have hl : l.dropLast ++ [p] = l := List.dropLast_append_getLast? p (by simp [h])
    have h2 : (p :: l.dropLast).Perm l := by
      have := List.perm_append_singleton p l.dropLast
      rw [hl] at this
      exact this.symm
    exact ((ih1.append (ih2.cons p)).trans List.perm_middle).trans ((hF.cons p).trans h2)


end ProvableContracts.Sorting
