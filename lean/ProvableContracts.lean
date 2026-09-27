-- Root of the Lean proofs for rust-mdipierro-nlib (GH-5). `lake build` checks every module below.
-- Layout: Defs/<Module> hold the exact models of the Rust functions; Theorems/<Module>/<name> hold one
-- theorem each, the name a contract's `lean_theorem:` cites (`pv proof-status` scans this tree).
import ProvableContracts.Theorems.Descriptive.chi_squared_nonneg
import ProvableContracts.Theorems.Descriptive.correlation_bounded
import ProvableContracts.Theorems.Descriptive.covariance_symm
import ProvableContracts.Theorems.Descriptive.mean_within_range
import ProvableContracts.Theorems.Descriptive.variance_nonneg
import ProvableContracts.Theorems.Dense.transpose_transpose
import ProvableContracts.Theorems.Numerical.simpson_exact_cubic
import ProvableContracts.Theorems.Numerical.trapezoid_exact_linear
import ProvableContracts.Theorems.Sorting.quicksort_correct
import ProvableContracts.Theorems.Sorting.quicksort_perm
import ProvableContracts.Theorems.Sorting.quicksort_sorted
