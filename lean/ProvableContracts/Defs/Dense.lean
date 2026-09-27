-- Dense matrix algebra — contract matrix-algebra-v1, code src/matrix.rs.
-- Written by hand in the `pv lean` layout (the transpose equation carries no obligation of its own).
--
-- `Mat` is the Rust `Matrix`: a row count, a column count and a row-major buffer, entry `(i, j)`
-- stored at index `i * cols + j`. `transpose` is the Rust loop `data[j * rows + i] = a[(i, j)]`,
-- written as the value it leaves at each index `k = j * rows + i`. Transposing only moves values,
-- so the model is generic in the entry type and nothing is rounded.

import Mathlib.Tactic.Ring

namespace ProvableContracts.Dense

/-- A `rows × cols` matrix over `α` stored row-major, as the Rust `Matrix` stores its `Vec<f64>`. -/
structure Mat (α : Type*) where
  rows : ℕ
  cols : ℕ
  data : ℕ → α

variable {α : Type*}

/-- `a[(i, j)]`: the row-major index `i * cols + j`. -/
def Mat.get (a : Mat α) (i j : ℕ) : α := a.data (i * a.cols + j)

/-- `transpose`: a `cols × rows` matrix whose buffer holds `a[(i, j)]` at `j * rows + i`. -/
def transpose (a : Mat α) : Mat α where
  rows := a.cols
  cols := a.rows
  data := fun k => a.get (k % a.rows) (k / a.rows)

end ProvableContracts.Dense
