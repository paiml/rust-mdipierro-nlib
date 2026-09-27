-- Theorem: quicksort_correct
-- The theorem the `quicksort` equation of sorting-v1 names: sorted, a permutation, and the same length.
-- Written by hand (the equation carries no obligation of its own, so `pv lean` makes no stub).

import ProvableContracts.Theorems.Sorting.quicksort_sorted

namespace ProvableContracts.Sorting

variable {α : Type*} [LinearOrder α]

theorem quicksort_correct (l : List α) :
    (quicksort l).Pairwise (· ≤ ·) ∧ (quicksort l).Perm l ∧ (quicksort l).length = l.length :=
  ⟨quicksort_sorted l, quicksort_perm l, (quicksort_perm l).length_eq⟩

end ProvableContracts.Sorting
