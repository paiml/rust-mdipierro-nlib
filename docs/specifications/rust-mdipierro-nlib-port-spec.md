# rust-mdipierro-nlib — Port Specification v1.0.0

**Contract-first Rust port of Di Pierro's nlib numerical library.**

A zero-external-dependency numerical algorithms library built on the
aprender monorepo (tensor, FFT, graph, monte-carlo, solve, rand) with
provable-contracts enforcement at every layer. Every function exists
because a YAML contract requires it. No function ships without A-grade
coverage, full penetration, and pmat compliance.

**Canonical spec.** This is the ONE spec for the rust-mdipierro-nlib port.

---

## Table of Contents

| # | Section |
|---|---------|
| 1 | [Design Principles](#1-design-principles) |
| 2 | [Dependency Policy](#2-dependency-policy) |
| 3 | [Contract-First Workflow](#3-contract-first-workflow) |
| 4 | [Contract Suite](#4-contract-suite) |
| 5 | [Module Architecture](#5-module-architecture) |
| 6 | [Depyler Integration](#6-depyler-integration) |
| 7 | [Quality Gates](#7-quality-gates) |
| 8 | [Falsification Protocol](#8-falsification-protocol) |
| 9 | [Implementation Phases](#9-implementation-phases) |
| 10 | [References](#10-references) |

---

## 1. Design Principles

**P1: Contract before code.** No Rust function may be written until its
YAML contract exists, passes `pv validate`, and scores >= 0.60 (Grade C).
The contract defines the equation, domain, codomain, preconditions,
postconditions, invariants, falsification tests, and Kani harnesses.
Code is a proof that the contract is satisfiable.

**P2: Zero external dependencies.** The ONLY allowed dependency is the
aprender monorepo (`aprender-tensor`, `aprender-fft`, `aprender-graph`,
`aprender-monte-carlo`, `aprender-rand`, `aprender-solve`, `aprender-sparse`).
No `nalgebra`, no `ndarray`, no `num`, no `rand`. If aprender doesn't
have it, we implement it in nlib with a contract, then upstream it.

**P3: Full penetration.** Every contract equation must have at least one
call site in source (`contract_pre_*!` + `contract_post_*!`). Penetration
= call_sites / bindings >= 100%. No dead contracts. No unbound equations.

**P4: A-grade files.** Every `.rs` file must score A- or higher on pmat
TDG analysis. No B+ or below. This means: structural complexity <= 15,
duplication ratio <= 15%, doc coverage >= 50%, consistency >= 90%.

**P5: 95% coverage floor.** `cargo llvm-cov` must report >= 95% line
coverage across the workspace. Every public function must have at least
one test exercising the happy path and one test exercising a boundary.

**P6: Popperian falsification.** Every contract must have falsification
tests that *try to break* the invariants. If a falsification test passes
(the invariant holds), confidence increases. If it fails, we found a bug.
Falsification tests are the primary quality signal, not unit tests.

**P7: Mathematical fidelity.** Every equation uses the exact formula from
Di Pierro (2023). No "simplified" versions. The contract `formula` field
must be copy-pasteable into a Lean 4 theorem statement.

---

## 2. Dependency Policy

### Allowed dependencies

| Crate | Purpose | Source |
|-------|---------|--------|
| `aprender-tensor` | Dense/sparse tensor ops | aprender monorepo |
| `aprender-fft` | FFT/DFT | aprender monorepo |
| `aprender-graph` | Graph data structures | aprender monorepo |
| `aprender-monte-carlo` | MC simulation engine | aprender monorepo |
| `aprender-rand` | PRNG and distributions | aprender monorepo |
| `aprender-solve` | Nonlinear solvers | aprender monorepo |
| `aprender-sparse` | Sparse matrix ops | aprender monorepo |
| `provable-contracts` | Contract enforcement (dev) | crates.io |

### Forbidden dependencies

Everything else. Specifically:
- `nalgebra`, `ndarray`, `faer` — use `aprender-tensor`
- `rand`, `rand_distr` — use `aprender-rand`
- `num`, `num-traits`, `num-complex` — implement in nlib
- `petgraph` — use `aprender-graph`
- `rustfft` — use `aprender-fft`
- `statrs` — implement in nlib

### Rationale

The aprender monorepo is the PAIML sovereign stack's compute layer. By
depending only on aprender, nlib validates that aprender's APIs are
sufficient for a complete numerical algorithms textbook. Gaps found
during nlib development become aprender feature requests — dogfooding
the stack.

---

## 3. Contract-First Workflow

```
Paper equation ─→ YAML contract ─→ pv validate ─→ pv score ─→ GATE
                                                                 │
                                  ┌──────────────────────────────┘
                                  ▼
                    pv scaffold ─→ Trait stubs ─→ Implement ─→ GATE
                                                                │
                    pv probar ──→ Falsify tests ─→ Run ────────┘
                    pv kani ───→ BMC harnesses ──→ Verify ─────┘
                    depyler ───→ Reference impl ─→ Compare ────┘
```

### Step-by-step

1. **Extract**: Read Di Pierro Ch. N, extract equation with domain/codomain
2. **Specify**: Write `contracts/<name>-v1.yaml` with full YAML schema
3. **Validate**: `pv validate contracts/<name>-v1.yaml` must pass
4. **Score**: `pv score contracts/<name>-v1.yaml` must be >= C (0.25)
5. **Scaffold**: `pv scaffold contracts/<name>-v1.yaml` generates trait
6. **Transpile**: `depyler transpile nlib.py --function <name>` for reference
7. **Implement**: Write Rust against the trait, using aprender primitives
8. **Falsify**: `pv probar` generates property tests; run with `cargo test`
9. **Verify**: `cargo kani` runs bounded model checking harnesses
10. **Bind**: Add equation→function mapping to `contracts/binding.yaml`
11. **Inject**: Add `contract_pre_*!()` and `contract_post_*!()` call sites
12. **Gate**: `pmat comply check` must pass; all files must be A-grade

### Depyler role

Depyler transpiles the Python nlib source to Rust as a *reference
implementation*. This reference is NOT shipped — it's used to:
- Generate known-answer test vectors (Python output = expected Rust output)
- Validate semantic equivalence between Python and Rust implementations
- Bootstrap initial implementations before optimization
- Cross-check numerical precision (Python f64 vs Rust f64)

The depyler-generated code is compared against the contract-driven
implementation via `depyler verify --semantic-equiv`.

---

## 4. Contract Suite

Ten contracts covering the full nlib scope. 36 equations, 41 proof
obligations, 53 falsification tests, 30 Kani harnesses.

| Contract | Ch. | Equations | Key Invariants |
|----------|-----|-----------|----------------|
| `matrix-algebra-v1` | 4.4 | matmul, transpose, inverse, cholesky, determinant | A*A⁻¹ ≈ I, L*Lᵀ ≈ A, det(AB) = det(A)*det(B) |
| `nonlinear-solvers-v1` | 4.6 | bisection, newton, secant, fixed_point | \|f(root)\| < ε, sign change preserved |
| `optimization-v1` | 4.7 | golden_section, newton_opt, gradient_descent | \|∇f(x*)\| < ε, f(x*) ≤ f(x₀) |
| `integration-v1` | 4.10 | trapezoid, simpson, adaptive_quadrature | Richardson convergence, known-answer ∫sin = 2 |
| `fourier-transform-v1` | 4.11 | dft, fft, inverse_dft, parseval | \|ifft(fft(x))-x\| < ε, Parseval energy |
| `random-generators-v1` | 6 | lcg, mersenne_twister | Period > 0, output ∈ [0,m), chi² uniformity |
| `monte-carlo-v1` | 7 | mc_integrate, bootstrap, metropolis | Error ∝ 1/√N, detailed balance |
| `graph-algorithms-v1` | 3.7 | dijkstra, kruskal, bfs, dfs | Shortest path optimal, MST minimal, all reachable visited |
| `sorting-v1` | 3.5 | quicksort, mergesort, heapsort | Sorted output, permutation of input, stability (merge) |
| `statistics-v1` | 5 | mean, variance, covariance, correlation, chi² | variance ≥ 0, \|ρ\| ≤ 1, Σ(x-μ) = 0 |

### Binding registry

`contracts/binding.yaml` maps every equation to a Rust function:

```yaml
version: 1.0.0
target_crate: nlib
bindings:
- contract: matrix-algebra-v1.yaml
  equation: matmul
  module_path: nlib::matrix
  function: matmul
  status: implemented  # or: not_implemented
```

**Penetration target: 100%.** All 36 equations must be bound and have
call sites. No equation may exist in a contract without a corresponding
Rust function.

---

## 5. Module Architecture

```
nlib/
├── contracts/          # 10 YAML contracts (source of truth)
│   ├── binding.yaml    # Equation → function mapping
│   └── *.yaml          # One per domain
├── src/
│   ├── lib.rs          # Module declarations
│   ├── matrix.rs       # Dense matrix algebra
│   ├── solve.rs        # Nonlinear solvers
│   ├── optimize.rs     # Optimization methods
│   ├── integrate.rs    # Numerical integration
│   ├── fourier.rs      # DFT/FFT
│   ├── random.rs       # PRNGs
│   ├── monte_carlo.rs  # MC simulation
│   ├── graph.rs        # Graph algorithms
│   ├── sort.rs         # Sorting
│   └── stats.rs        # Statistics
├── tests/
│   └── contracts/      # pv-generated falsification tests
└── generated/          # pv scaffold output (not committed)
```

### Module → aprender mapping

| nlib module | aprender crate | What nlib adds |
|-------------|----------------|----------------|
| `matrix` | `aprender-tensor` | Cholesky, inverse, determinant, condition number |
| `solve` | `aprender-solve` | Bisection, secant, fixed-point (Newton exists) |
| `optimize` | `aprender-solve` | Golden section, multi-dim Newton, gradient descent |
| `integrate` | — | Trapezoid, Simpson, adaptive quadrature |
| `fourier` | `aprender-fft` | DFT (small-N), Parseval check, inverse |
| `random` | `aprender-rand` | LCG, MT19937 (if not in aprender-rand) |
| `monte_carlo` | `aprender-monte-carlo` | Bootstrap error, Metropolis-Hastings |
| `graph` | `aprender-graph` | Dijkstra, Kruskal, BFS, DFS |
| `sort` | — | Quicksort, mergesort, heapsort (contract-verified) |
| `stats` | — | Mean, variance, covariance, correlation, chi² |

---

## 6. Depyler Integration

Depyler transpiles Di Pierro's Python nlib to Rust for cross-validation.

### Workflow

```bash
# 1. Extract Python function
depyler extract nlib.py --function solve_bisection

# 2. Transpile to Rust
depyler transpile nlib.py --function solve_bisection --output /tmp/ref.rs

# 3. Compare against contract-driven implementation
depyler verify --semantic-equiv src/solve.rs /tmp/ref.rs

# 4. Generate test vectors from Python execution
depyler test-vectors nlib.py --function solve_bisection --count 100
```

### Semantic equivalence verification

Depyler's `depyler-verify` crate checks that:
- Same inputs produce same outputs (within f64 epsilon)
- Error conditions map correctly (Python exceptions → Rust Results)
- Edge cases match (empty input, NaN, Inf, zero-length)

This is NOT a correctness proof — it's a cross-validation signal.
The contract is the proof; depyler confirms the Python baseline agrees.

---

## 7. Quality Gates

### Gate 1: Contract validation
```bash
pv validate contracts/    # All 10 must pass
pv score contracts/       # All must be >= C (0.25)
```

### Gate 2: Binding completeness
```bash
pv lint contracts/ --binding contracts/binding.yaml
# 0 unbound equations, 0 unimplemented bindings
```

### Gate 3: Test coverage
```bash
cargo llvm-cov --lib      # >= 95% line coverage
```

### Gate 4: TDG quality
```bash
pmat comply check         # All files A-grade
```

### Gate 5: Falsification
```bash
cargo test                # All falsification tests pass
# Every contract has >= 5 falsification tests
```

### Gate 6: Contract enforcement
```bash
# Penetration check: call_sites / bindings >= 100%
pv kaizen --src-root .    # Grade A required
```

### CI pipeline

All 6 gates run on every push. Any gate failure blocks merge.

---

## 8. Falsification Protocol

Every contract includes falsification tests designed to *break*
invariants. These follow the Popperian methodology: we don't try to
prove correctness — we try to find counterexamples.

### Template

```yaml
falsification_tests:
- id: FALSIFY-XX-001
  rule: <invariant name>
  prediction: <what should hold>
  test: |
    <Rust test code that tries to break the invariant>
  if_fails: <what the bug would be>
```

### Categories

1. **Precondition violation**: Call with invalid input, assert error
2. **Known-answer**: Compare against textbook/Wolfram Alpha values
3. **Boundary**: Test at domain boundaries (empty, single-element, max)
4. **Cross-reference**: Compare two implementations (e.g., DFT vs FFT)
5. **Roundtrip**: Apply then inverse, assert identity (fft/ifft, A*A⁻¹)
6. **Statistical**: Run 10,000 times, check distribution properties
7. **Adversarial**: Craft inputs that maximize floating-point error

### Minimum counts

| Domain | Minimum falsification tests |
|--------|---------------------------|
| Linear algebra | 5 per equation |
| Solvers/optimization | 5 per equation |
| Integration/FFT | 5 per equation |
| Random/Monte Carlo | 6 per equation |
| Graph/sorting | 6 per equation |
| Statistics | 6 per equation |

---

## 9. Implementation Phases

### Phase 0: Contracts (DONE)
- [x] 10 YAML contracts written and validated
- [x] 36 equations with exact Di Pierro formulas
- [x] 53 falsification tests specified
- [x] 30 Kani harnesses specified
- [x] Repo created, module stubs in place

### Phase 1: Foundation (sort + stats + matrix)
- [ ] Implement `sort.rs`: quicksort, mergesort, heapsort
- [ ] Implement `stats.rs`: mean, variance, covariance, correlation
- [ ] Implement `matrix.rs`: matmul, transpose, inverse, cholesky
- [ ] Binding registry for Phase 1 equations
- [ ] 95% coverage, all A-grade files
- [ ] Gate: `pv lint` + `pmat comply` pass

### Phase 2: Solvers (solve + optimize + integrate)
- [ ] Implement `solve.rs`: bisection, newton, secant, fixed_point
- [ ] Implement `optimize.rs`: golden_section, newton_opt, gradient_descent
- [ ] Implement `integrate.rs`: trapezoid, simpson, adaptive_quadrature
- [ ] Depyler cross-validation against Python nlib
- [ ] Gate: all Phase 2 falsification tests pass

### Phase 3: Spectral + Stochastic (fourier + random + monte_carlo)
- [ ] Implement `fourier.rs`: dft, fft, inverse_dft
- [ ] Implement `random.rs`: lcg, mersenne_twister
- [ ] Implement `monte_carlo.rs`: mc_integrate, bootstrap, metropolis
- [ ] Gate: Parseval theorem holds, chi² uniformity test passes

### Phase 4: Graph algorithms
- [ ] Implement `graph.rs`: dijkstra, kruskal, bfs, dfs
- [ ] Gate: known-answer tests on textbook graphs

### Phase 5: Full integration
- [ ] 100% binding penetration (36/36 equations bound)
- [ ] `pv kaizen` Grade A
- [ ] `pmat comply check` all pass
- [ ] 95%+ coverage
- [ ] Publish to crates.io as `nlib`

---

## 10. References

### Primary source

Di Pierro, M. (2023). *Annotated Algorithms in Python: With Applications
in Physics, Biology, Finance* (3rd Ed.). ISBN 9798254871569.
Source: https://github.com/mdipierro/nlib

### Numerical analysis foundations

- Higham, N.J. (2002). *Accuracy and Stability of Numerical Algorithms*
  (2nd Ed.). SIAM. — Canonical reference for floating-point error analysis.
- Goldberg, D. (1991). "What Every Computer Scientist Should Know About
  Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1):5-48.
- Kahan, W. (1996). "IEEE 754R Meeting Notes." — Compensated summation.

### Formal verification of numerical code

- Boldo, S. & Melquiond, G. (2011). "Flocq: A Unified Library for
  Proving Floating-Point Algorithms in Coq." *IEEE ARITH*, pp. 243-252.
  — Formal verification of IEEE 754 floating-point in Coq.
- Ramananandro, T. et al. (2016). "A Unified Coq Framework for Verifying
  C Programs with Floating-Point Computations." *CPP*, pp. 15-26.
  — End-to-end verified C numerical code via Coq extraction.
- Daumas, M. & Melquiond, G. (2010). "Certification of Bounds on
  Expressions Involving Rounded Operators." *ACM TOMS*, 37(1):1-20.
  — Automated error bound certificates for numerical expressions.

### Algorithm correctness proofs

- Cormen, T.H. et al. (2022). *Introduction to Algorithms* (4th Ed.).
  MIT Press. — Loop invariant proofs for sorting, graph algorithms.
- Sedgewick, R. & Wayne, K. (2011). *Algorithms* (4th Ed.). Addison-Wesley.
  — Correctness proofs for quicksort, MST, shortest paths.
- Cooley, J.W. & Tukey, J.W. (1965). "An Algorithm for the Machine
  Calculation of Complex Fourier Series." *Math. Comp.*, 19:297-301.
  — Original FFT paper, correctness via DFT equivalence.

### Monte Carlo and random number generation

- L'Ecuyer, P. & Simard, R. (2007). "TestU01: A C Library for Empirical
  Testing of Random Number Generators." *ACM TOMS*, 33(4):22.
  — Comprehensive PRNG quality testing framework.
- Metropolis, N. et al. (1953). "Equation of State Calculations by Fast
  Computing Machines." *J. Chem. Phys.*, 21(6):1087-1092.
  — Original Metropolis algorithm, detailed balance proof.
- Matsumoto, M. & Nishimura, T. (1998). "Mersenne Twister: A
  623-Dimensionally Equidistributed Uniform Pseudo-Random Number
  Generator." *ACM TOMACS*, 8(1):3-30.

### Contract-based design

- Meyer, B. (1997). *Object-Oriented Software Construction* (2nd Ed.).
  Prentice Hall. — Design by Contract: preconditions, postconditions,
  invariants.
- Hatcliff, J. et al. (2012). "Behavioral Interface Specification
  Languages." *ACM Computing Surveys*, 44(3):16. — Survey of DbC
  specification languages (JML, Spec#, Eiffel).

### Python-to-Rust transpilation

- PAIML Engineering (2025). "Depyler: A Python-to-Rust Transpiler with
  Semantic Verification." https://github.com/paiml/depyler — Formal
  semantic equivalence checking between source and target languages.

### Provable contracts methodology

- PAIML Engineering (2025). "Provable Contracts: Papers to Math to
  Contracts in Code." https://github.com/paiml/provable-contracts —
  YAML contract schema, Kani BMC, Lean 4 theorem proving, seven-phase
  pipeline.
