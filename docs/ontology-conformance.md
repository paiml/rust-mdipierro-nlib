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
| `pv validate` on every contract | PASS (10/10) | CI job "Contract Validation" |
| `pv lint` | PASS (36 warnings) | CI job "Contract Validation" |
| `entity:` declarations / Σ types | NOT YET | census: 10 contracts, 10 unanchored |
| SHACL shapes, `pv lint --gate shapes` fails closed | NOT YET | `--gate shapes` → exit 2, `decline: NoShapes` |
| contracts gate, every step RUNS (ONT-G) | NOT YET | — |
| census, `extract --check`, generated README table, provenance, ratchet | NOT YET | — |
| every example an enforced contract (entity, shape, bound equations, falsification, Kani) | NOT YET (0/11) | — |

## CI gates and their scope

| Gate | Status | Why |
|------|--------|-----|
| Kani BMC | blocking, 60 min timeout | 5 harnesses, all verified locally (quicksort sorted 211 s, quicksort length 309 s, correlation 142 s, variance 17 s, transpose 2 s). The old symbolic-length quicksort harness (unwind 8) ran into the 6 h job limit on main. |
| Mutation testing | blocking on the PR diff (`--in-diff`); full sweep advisory on `main` | Full sweep on main had 114 missed mutants before GH-1 (fourier, graph, integrate, matrix, monte_carlo, optimize, random). The backlog is being killed module by module; the sweep becomes blocking at zero. |
| PMAT comply | advisory | Failing checks are repository governance (branch protection needs an admin, `deny.toml`, `build.rs`, roadmap/spec schema, TDG), not contract enforcement. |
