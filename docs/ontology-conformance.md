# Ontology conformance (ONT-001)

Status of this repository against the paiml ontology spec (ONT-001 v4.17).
Each row is **row → status → evidence**. A gate that did not run is reported
as NOT RUN, never as a pass.

Tracking ticket: GH-1 ([#1](https://github.com/paiml/rust-mdipierro-nlib/issues/1)).

## Toolchain pin

| Item | Value |
|------|-------|
| aprender (sole runtime dependency) | 0.70.0-dev (car/0.70.0 @ 1274d040d) — `rev = "1274d040de369d37bb2a9549af5816658dfb4856"` in `Cargo.toml` |
| pv (aprender-contracts-cli) | built from the same rev; reports `pv 0.70.0 (1274d040d)`; CI installs it from the rev in `Cargo.toml` |
| Upgrade path | next CI-green car/0.70.0 rev (between PRs only) → `v0.70.0` tag → crates.io |

### Upstream CI state at the pin

Check runs on paiml/aprender@1274d040d, observed 2026-09-27. Recorded, not
fixed here (aprender is never patched from this repo).

| Upstream check | Conclusion |
|----------------|------------|
| ci / gate, determinism, workspace-test shards 1–3, Contract Enforcement | success |
| `x86-main` | **failure** |
| `gate` | **failure** |
| `present` | **failure** |
| `yoga` | cancelled |
| `coverage` | in progress |

## Conformance rows

| Row | Status | Evidence |
|-----|--------|----------|
| aprender 0.70 line, sole runtime dependency | PASS | `Cargo.toml`; `cargo tree -e normal --depth 1` shows only `aprender` |
| `pv validate` on every contract | PASS (22/22) | CI job "Contract Validation" |
| `pv lint` | PASS (96 warnings, all PV-ENF-002 "no `lean_theorem`"; see N/A rows) | CI job "Contract Validation", step "Lint contracts" |
| binding.yaml symbols resolve (extract:code) | PASS (96/96) | `pv lint contracts/ --gate shapes` → `symbols_resolved: 96, symbols_unresolved: 0`. 16 bindings were spelled `Type::method`, which pv never resolves ([aprender#4521](https://github.com/paiml/aprender/issues/4521)); they now use the bare method name |
| `entity:` declarations / Σ types | PARTIAL | 11 `example-*-v1` contracts carry `entity: {type: json}`; the Σ ontology (`contracts/ontology.yaml`) is NOT YET, so gates sigma, relations and valid-under are NotArmed |
| SHACL shapes, `pv lint --gate shapes` fails closed | PASS | `--gate shapes` → Pass: 11 shapes, 11 focus nodes, all armed, 0 violations, 66 plant violations. Planting a Simpson weight of 3.9 in `src/integrate.rs` made the example exit 1 and the gate Fail (exit 1, 5 violations); restoring it gave Pass with byte-identical evidence |
| contracts gate, every step RUNS (ONT-G) | NOT YET | — |
| census, `extract --check`, generated README table, provenance, ratchet | NOT YET | — |
| every example an enforced contract (entity, shape, bound equations, falsification, Kani) | PASS (11/11) | see the table below |

## Examples as contracts

Each `examples/<name>.rs` has a `receipt()` whose checks are named after its
contract's equations. `--json` prints the receipt, and the exit code is 1 when
any check fails. `evidence/examples/<name>.json` is the tracked copy.
`contracts/example-<name>-v1.yaml` extracts that copy as a JSON entity and
closes it with a SHACL shape that requires every check to hold. CI's
"Examples" job reruns each example and diffs its receipt against the evidence.
The receipt type itself is pinned by `example-receipt-v1.yaml`
(FALSIFY-RCPT-001..007, KANI-RCPT-001).

| Example | Contract | Checks | Equations | Falsification | Kani |
|---------|----------|-------:|----------:|---------------|------|
| integrate | example-integrate-v1 | 7 | 6 | FALSIFY-EXINTG-001..004 | KANI-RCPT-001, KANI-INTG-001 |
| solve | example-solve-v1 | 5 | 5 | FALSIFY-EXSOLVE-001..004 | KANI-RCPT-001 |
| sort | example-sort-v1 | 7 | 7 | FALSIFY-EXSORT-001..004 | KANI-RCPT-001, KANI-SORT-001/002 |
| stats | example-stats-v1 | 6 | 6 | FALSIFY-EXSTATS-001..004 | KANI-RCPT-001, KANI-STATS-001/002 |
| matrix | example-matrix-v1 | 6 | 6 | FALSIFY-EXMATRIX-001..004 | KANI-RCPT-001, KANI-MATRIX-001 |
| fourier | example-fourier-v1 | 6 | 5 | FALSIFY-EXFOURIER-001..004 | KANI-RCPT-001 |
| graph | example-graph-v1 | 6 | 6 | FALSIFY-EXGRAPH-001..004 | KANI-RCPT-001 |
| monte_carlo | example-monte-carlo-v1 | 6 | 3 | FALSIFY-EXMC-001..004 | KANI-RCPT-001 |
| optimize | example-optimize-v1 | 5 | 4 | FALSIFY-EXOPT-001..004 | KANI-RCPT-001 |
| random | example-random-v1 | 4 | 3 | FALSIFY-EXRANDOM-001..004 | KANI-RCPT-001 |
| parity | example-parity-v1 | 23 | 6 | FALSIFY-EXPARITY-001..004 | KANI-RCPT-001 |

Kani applies where the claim is a bounded property of a function the example
exercises. The floating-point claims in fourier, graph, monte_carlo, optimize,
random and parity are covered by the receipt harness and falsification tests,
not by a per-example BMC proof.

## N/A rows

| Row | Status | Evidence |
|-----|--------|----------|
| Lean theorem per equation (PV-ENF-002, theorem-pairing gate) | N/A | This crate proves its bounded properties with Kani (8 harnesses in `src/kani_harnesses.rs`); there is no Lean theorem base. The 96 PV-ENF-002 warnings are recommendations, not failures |

## Findings

| Finding | Status | Evidence |
|---------|--------|----------|
| `examples/optimize.rs`: Newton from x0 = 0.5 converged to the local **maximum** x = 0 of the quartic, which the old example printed as the minimum | FIXED | Newton now starts at x0 = 1.0. FALSIFY-EXOPT-004 keeps the old start and asserts that it lands on the maximum |
| `examples/graph.rs` claimed a "Fig. 3.12 style" graph | FIXED | Claim removed; the receipt checks computed distances and the MST weight instead |
| `src/matrix.rs` `IndexMut` writes through a pointer cast from `as_slice().as_ptr()` (a shared borrow), which is unsound | OPEN | Tracked on GH-1 |
| Binding `uniform_chi2 → nlib::random::next_f64` resolves to `Lcg::next_f64`, the first impl, not `Mt19937::next_f64` | OPEN (upstream) | [aprender#4521](https://github.com/paiml/aprender/issues/4521) |

## CI gates and their scope

| Gate | Status | Why |
|------|--------|-----|
| Kani BMC | blocking, 60 min timeout | 7 harnesses, all verified locally (quicksort sorted 211 s, quicksort length 309 s, correlation 142 s, variance 10 s, transpose 8 s, within 1 s, simpson odd n 2 s). The old symbolic-length quicksort harness (unwind 8) ran into the 6 h job limit on main. |
| Mutation testing | blocking on the PR diff (`--in-diff`); full sweep advisory on `main` | Full sweep on main had 114 missed mutants before GH-1 (fourier, graph, integrate, matrix, monte_carlo, optimize, random). The backlog is being killed module by module; the sweep becomes blocking at zero. |
| PMAT comply | advisory | Failing checks are repository governance (branch protection needs an admin, `deny.toml`, `build.rs`, roadmap/spec schema, TDG), not contract enforcement. |
| SHACL shapes | blocking | `pv lint contracts/ --gate shapes` in "Contract Validation": exit 1 on a violation, 2 when no shape is armed |
| Example receipts | blocking | "Examples" job: `cargo test --test examples`, then each example with and without `--json`; the receipt must equal `evidence/examples/<name>.json` |
