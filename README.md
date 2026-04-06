# rust-mdipierro-nlib

Provable-contracts-first Rust port of Massimo Di Pierro's
[nlib](https://github.com/mdipierro/nlib) — the companion library to
*Annotated Algorithms in Python* (3rd Ed., 2023).

## Design Methodology

This project uses **contract-first design**: every algorithm is specified
as a YAML contract *before* any Rust code is written. Each contract
defines:

- **Equations** with exact mathematical formulas from the book
- **Preconditions** (what must hold before calling)
- **Postconditions** (what must hold after return)
- **Proof obligations** (what we must prove)
- **Falsification tests** (Popperian tests that try to break invariants)
- **Kani harnesses** (bounded model checking proofs)

The contracts are validated by
[provable-contracts](https://github.com/paiml/provable-contracts) (`pv`).

## Contracts

| Contract | Equations | Chapter | Domain |
|----------|-----------|---------|--------|
| `matrix-algebra-v1` | matmul, transpose, inverse, cholesky, determinant | 4.4 | Linear algebra |
| `nonlinear-solvers-v1` | bisection, newton, secant, fixed_point | 4.6 | Root finding |
| `optimization-v1` | golden_section, newton_optimize, gradient_descent | 4.7-4.8 | Minimization |
| `integration-v1` | trapezoid, simpson, adaptive_quadrature | 4.10 | Quadrature |
| `fourier-transform-v1` | dft, fft, inverse_dft, parseval | 4.11 | Spectral analysis |
| `random-generators-v1` | lcg, mersenne_twister | 6 | PRNGs |
| `monte-carlo-v1` | mc_integrate, bootstrap_error, metropolis | 7 | Simulation |
| `graph-algorithms-v1` | dijkstra, kruskal_mst, bfs, dfs | 3.7 | Graph theory |
| `sorting-v1` | quicksort, mergesort, heapsort | 3.5 | Sorting |
| `statistics-v1` | mean, variance, covariance, correlation, chi_squared | 5 | Statistics |

**Totals:** 36 equations, 41 proof obligations, 53 falsification tests,
30 Kani harnesses.

## Workflow

```bash
# Validate all contracts
pv validate contracts/

# Score contract quality
pv score contracts/

# Generate Rust trait scaffolds
pv scaffold contracts/matrix-algebra-v1.yaml

# Generate Kani proof harnesses
pv kani contracts/matrix-algebra-v1.yaml

# Generate falsification tests
pv probar contracts/matrix-algebra-v1.yaml

# Full pipeline
pv generate contracts/matrix-algebra-v1.yaml -o generated/
```

## Module Map

| Rust Module | Contract | Book Section |
|-------------|----------|--------------|
| `nlib::matrix` | `matrix-algebra-v1` | Ch. 4.4 |
| `nlib::solve` | `nonlinear-solvers-v1` | Ch. 4.6 |
| `nlib::optimize` | `optimization-v1` | Ch. 4.7-4.8 |
| `nlib::integrate` | `integration-v1` | Ch. 4.10 |
| `nlib::fourier` | `fourier-transform-v1` | Ch. 4.11 |
| `nlib::random` | `random-generators-v1` | Ch. 6 |
| `nlib::monte_carlo` | `monte-carlo-v1` | Ch. 7 |
| `nlib::graph` | `graph-algorithms-v1` | Ch. 3.7 |
| `nlib::sort` | `sorting-v1` | Ch. 3.5 |
| `nlib::stats` | `statistics-v1` | Ch. 5 |

## Reference

Di Pierro, M. (2023). *Annotated Algorithms in Python: With Applications
in Physics, Biology, Finance* (3rd Ed.). ISBN 9798254871569.

Source: [github.com/mdipierro/nlib](https://github.com/mdipierro/nlib)

## License

MIT
