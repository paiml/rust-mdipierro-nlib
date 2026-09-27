-- Sorting — contract sorting-v1, code src/sort.rs.
-- Layout from `pv lean contracts/sorting-v1.yaml`; the definitions are written by hand.
--
-- `quicksort` models the Rust `quicksort<T: Ord>` (Lomuto partition, the LAST element is the pivot,
-- an element goes left when `a[j] <= pivot`). The Rust permutes a slice in place; the model returns
-- the list `left ++ pivot :: right` that the partition leaves, then recurses. The order the swaps
-- leave inside `left` and `right` is not modelled: docs/lean-proofs.md states that gap.

import Mathlib.Data.List.Perm.Basic
import Mathlib.Data.List.Sort

namespace ProvableContracts.Sorting

variable {α : Type*} [LinearOrder α]

/-- Lomuto quicksort over a list: pivot on the last element, `≤ pivot` to the left. -/
def quicksort (l : List α) : List α :=
  match h : l.getLast? with
  | none => []
  | some p =>
    have hlen : l.dropLast.length < l.length := by
      have hne : l ≠ [] := by rintro rfl; simp at h
      rw [List.length_dropLast]
      exact Nat.sub_lt (List.length_pos_iff.mpr hne) Nat.one_pos
    quicksort (l.dropLast.filter (· ≤ p)) ++ p :: quicksort (l.dropLast.filter (fun x => ¬ x ≤ p))
termination_by l.length
decreasing_by
  · exact lt_of_le_of_lt (List.length_filter_le _ _) hlen
  · exact lt_of_le_of_lt (List.length_filter_le _ _) hlen

/-- The empty case: a list with no last element sorts to `[]`. -/
theorem quicksort_nil_of (l : List α) (h : l.getLast? = none) : quicksort l = [] := by
  unfold quicksort; split
  · rfl
  · simp_all

/-- One step of the recursion: partition around the last element and sort both sides. -/
theorem quicksort_of_getLast (l : List α) (p : α) (h : l.getLast? = some p) :
    quicksort l = quicksort (l.dropLast.filter (· ≤ p)) ++ p ::
      quicksort (l.dropLast.filter (fun x => ¬ x ≤ p)) := by
  rw [quicksort]; split
  · simp_all
  · rename_i q hq; rw [h] at hq; cases hq; rfl

end ProvableContracts.Sorting
