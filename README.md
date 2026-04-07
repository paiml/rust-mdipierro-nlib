# rust-mdipierro-nlib

[![CI](https://github.com/paiml/rust-mdipierro-nlib/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/rust-mdipierro-nlib/actions/workflows/ci.yml)
[![Contract Grade](https://img.shields.io/badge/contract_grade-A_(0.93)-brightgreen)](contracts/)
[![Tests](https://img.shields.io/badge/tests-129_passed-brightgreen)](src/)
[![Coverage](https://img.shields.io/badge/coverage-98.7%25-brightgreen)](src/)
[![Binding](https://img.shields.io/badge/binding-36%2F36_(100%25)-brightgreen)](contracts/binding.yaml)
[![Falsification](https://img.shields.io/badge/falsification-53_tests-blue)](contracts/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Provable-contracts-first Rust port of Massimo Di Pierro's
[nlib](https://github.com/mdipierro/nlib) — the companion library to
*Annotated Algorithms in Python* (3rd Ed., 2023).

**Sole dependency:** [`aprender`](https://crates.io/crates/aprender) —
no other external crates.

## Contract Coverage

<!-- Generated from contract state. Do not edit manually. -->
<!-- Regenerate: pv score contracts/ --binding contracts/binding.yaml -->

| Contract | Grade | Spec | Falsify | Kani | Bind | Equations |
|----------|-------|------|---------|------|------|-----------|
| fourier-transform-v1 | B (0.85) | 0.70 | 1.00 | 0.90 | 1.00 | dft, fft, inverse_dft, parseval |
| graph-algorithms-v1 | B (0.81) | 0.70 | 1.00 | 0.75 | 1.00 | dijkstra, kruskal_mst, bfs, dfs |
| integration-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | trapezoid, simpson, adaptive_quadrature |
| matrix-algebra-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | matmul, transpose, inverse, cholesky, det |
| monte-carlo-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | mc_integrate, bootstrap_error, metropolis |
| nonlinear-solvers-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | bisection, newton, secant, fixed_point |
| optimization-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | golden_section, newton_opt, gradient_desc |
| random-generators-v1 | B (0.79) | 0.70 | 1.00 | 0.68 | 1.00 | lcg, mersenne_twister |
| sorting-v1 | B (0.77) | 0.70 | 1.00 | 0.60 | 1.00 | quicksort, mergesort, heapsort |
| statistics-v1 | B (0.76) | 0.70 | 1.00 | 0.54 | 1.00 | mean, variance, covariance, corr, chi² |

**Codebase: Grade A (0.93)** — 100% binding, 100% falsification, 36/36 equations.

## Quick Start

```rust
use nlib::sort::quicksort;
use nlib::stats::{mean, correlation};
use nlib::solve::bisection;
use nlib::integrate::simpson;

// Sort
let mut data = vec![5, 3, 1, 4, 2];
quicksort(&mut data);
assert_eq!(data, vec![1, 2, 3, 4, 5]);

// Statistics
let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
assert!((mean(&x) - 3.0).abs() < 1e-10);

// Root finding: solve cos(x) = x
let root = bisection(|x| x.cos() - x, 0.0, 1.0, 1e-10);
assert!((root.cos() - root).abs() < 1e-9);

// Integration: ∫₀^π sin(x) dx = 2
let area = simpson(|x| x.sin(), 0.0, std::f64::consts::PI, 100);
assert!((area - 2.0).abs() < 1e-10);
```

## Design Methodology

Every algorithm is specified as a YAML contract **before** any Rust code
is written. The contract defines:

- **Equations** — exact mathematical formulas from Di Pierro (2023)
- **Preconditions** — what must hold before calling
- **Postconditions** — what must hold after return
- **Falsification tests** — Popperian tests that try to break invariants
- **Kani harnesses** — bounded model checking proofs

Contracts are validated by
[provable-contracts](https://github.com/paiml/provable-contracts) (`pv`).

## Module Map

| Module | Contract | Ch. | aprender API |
|--------|----------|-----|-------------|
| `nlib::matrix` | matrix-algebra-v1 | 4.4 | `Matrix<f64>` |
| `nlib::solve` | nonlinear-solvers-v1 | 4.6 | `Vector<f32>` |
| `nlib::optimize` | optimization-v1 | 4.7 | `SGD`, `Vector<f32>` |
| `nlib::integrate` | integration-v1 | 4.10 | `Vector<f32>` |
| `nlib::fourier` | fourier-transform-v1 | 4.11 | `Matrix<f64>` |
| `nlib::random` | random-generators-v1 | 6 | `MonteCarloRng` |
| `nlib::monte_carlo` | monte-carlo-v1 | 7 | `MonteCarloRng` |
| `nlib::graph` | graph-algorithms-v1 | 3.7 | `graph::Graph` |
| `nlib::sort` | sorting-v1 | 3.5 | `Vector<f32>` |
| `nlib::stats` | statistics-v1 | 5 | `Vector<f32>`, `chisquare` |

## Examples

Every module has a runnable example. Each demonstrates the core API with
real numerical output.

| Example | Command | What it does |
|---------|---------|-------------|
| [sort](examples/sort.rs) | `cargo run --example sort` | Quicksort, mergesort, heapsort on the same input |
| [stats](examples/stats.rs) | `cargo run --example stats` | Mean, variance, correlation, chi-squared |
| [matrix](examples/matrix.rs) | `cargo run --example matrix` | Matmul, transpose, inverse, Cholesky, determinant |
| [solve](examples/solve.rs) | `cargo run --example solve` | Bisection, Newton, secant, fixed-point root finding |
| [optimize](examples/optimize.rs) | `cargo run --example optimize` | Golden section, Newton optimizer, gradient descent |
| [integrate](examples/integrate.rs) | `cargo run --example integrate` | Trapezoid, Simpson, adaptive quadrature |
| [fourier](examples/fourier.rs) | `cargo run --example fourier` | DFT, FFT, inverse DFT, roundtrip check |
| [random](examples/random.rs) | `cargo run --example random` | LCG (MINSTD), Mersenne Twister, uniformity test |
| [monte_carlo](examples/monte_carlo.rs) | `cargo run --example monte_carlo` | MC integration of x^2, bootstrap error estimation |
| [graph](examples/graph.rs) | `cargo run --example graph` | Dijkstra, BFS, DFS, Kruskal MST on weighted graph |

## Metrics

| Metric | Value |
|--------|-------|
| Contracts | 10 |
| Equations | 36 (100% bound) |
| Proof obligations | 41 |
| Falsification tests | 53 |
| Kani harnesses | 30 |
| Rust tests | 129 |
| Line coverage | 98.7% |
| Lines of code | 2,213 |
| External deps | 1 (aprender only) |
| Contract grade | A (0.93) |

## Workflow

```bash
# Validate contracts
for f in contracts/*.yaml; do pv validate "$f"; done

# Score contract quality
pv score contracts/ --binding contracts/binding.yaml

# Run tests
cargo test --lib

# Check coverage
cargo llvm-cov --lib
```

## Reference

Di Pierro, M. (2023). *Annotated Algorithms in Python: With Applications
in Physics, Biology, Finance* (3rd Ed.). ISBN 9798254871569.

Source: [github.com/mdipierro/nlib](https://github.com/mdipierro/nlib)

## License

MIT
