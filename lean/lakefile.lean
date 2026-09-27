import Lake
open Lake DSL

-- Lean 4 proofs for the contracts' equations (GH-5). The toolchain is pinned in lean-toolchain and
-- Mathlib to the release tag of the same version, so `lake exe cache get` fetches prebuilt oleans:
-- Mathlib is never built from source here or in CI.
package nlibProofs where
  leanOptions := #[
    ⟨`autoImplicit, false⟩,
    -- A `sorry` is a warning in Lean; here every warning is an error, so an admitted proof fails the build.
    ⟨`warningAsError, true⟩
  ]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4" @ "v4.34.1"

@[default_target]
lean_lib ProvableContracts where
  srcDir := "."
