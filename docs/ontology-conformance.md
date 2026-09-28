# Ontology conformance (ONT-001)

Status of this repository against the paiml ontology spec (ONT-001 v4.17).
Each row is **row → status → evidence**. A gate that did not run is reported
as NOT RUN, never as a pass.

Every row that states a number carries a provenance mark (R-10): `[V date]`
measured on that date, `[C source]` cited. `scripts/lint-provenance.sh`
fails the contracts gate on an unmarked claim.

Tracking ticket: GH-1 ([#1](https://github.com/paiml/rust-mdipierro-nlib/issues/1)).

## Toolchain pin

| Item | Value |
|------|-------|
| aprender (sole runtime dependency) | 0.70.0-dev (car/0.70.0 @ 1274d040d); `rev = "1274d040de369d37bb2a9549af5816658dfb4856"` in `Cargo.toml` [V 2026-09-27] |
| pv (aprender-contracts-cli) | built from the same rev; reports `pv 0.70.0 (1274d040d)`; CI installs it from the rev in `Cargo.toml` [V 2026-09-27] |
| Upgrade path | next CI-green car/0.70.0 rev (between PRs only) → `v0.70.0` tag → crates.io [C GH-1] |

### Upstream CI state at the pin

Check runs on paiml/aprender@1274d040d, observed 2026-09-27. They are recorded
here and not fixed (this repo never patches aprender).

| Upstream check | Conclusion |
|----------------|------------|
| ci / gate, determinism, workspace-test shards 1–3, Contract Enforcement | success [V 2026-09-27] |
| `x86-main` | **failure** [V 2026-09-27] |
| `gate` | **failure** |
| `present` | **failure** |
| `yoga` | cancelled |
| `coverage` | in progress |

## Conformance rows

Measured with `pv lint contracts/ --binding contracts/binding.yaml --crate-dir .`
(exit 0) and `bash scripts/contracts_gate.sh`.

| Row | Status | Evidence |
|-----|--------|----------|
| aprender 0.70 line, sole runtime dependency | PASS | `Cargo.toml`; `cargo tree -e normal --depth 1` shows only `aprender` [V 2026-09-27] |
| `pv lint`, every armed gate | PASS (14 of 14 armed gates Pass) | validate, audit, score, verify, enforce, enforcement-level, duplicate-stems, composition, reverse-coverage, sigma, relations, shapes, valid-under, depends-on-present. The only gate not armed is theorem-pairing (see N/A). The 102 findings are warnings: 96 PV-ENF-002 (no `lean_theorem`) and 6 PV-RCV-001 (unbound public fns) [V 2026-09-27] |
| binding.yaml symbols resolve (extract:code) | PASS (96/96) | `--gate shapes` → `symbols_resolved: 96, symbols_unresolved: 0`. 16 bindings were spelled `Type::method`, which pv never resolves ([aprender#4521](https://github.com/paiml/aprender/issues/4521)); they now use the bare method name [V 2026-09-27] |
| reverse-coverage | PASS (90.2%, threshold 50%) | pv reports `total_pub_fns: 61, bound_fns: 40, unbound_fns: 6, coverage_pct: 90.16`; it needs `--binding` and `--crate-dir`, and without them it declines (exit 2), so every lint here passes both [V 2026-09-27] |
| `entity:` declarations / Σ entity types | PASS | `contracts/ontology.yaml` (`schema: ont-sigma-v1`): 3 entity types (pv-contract, json, code), 5 roles (binds, falsified_by, proved_by, refines, depends_on; the last two acyclic), 20 symbols. The sigma gate passes with 77 formal expressions and 41 in prose. The 11 `example-*-v1` contracts carry `entity: {type: json}` [V 2026-09-27] |
| relations / depends-on-present | PASS | 10 example contracts declare `relations.depends_on` on their algorithm contract(s) and on `example-receipt-v1`; every target exists [V 2026-09-27] |
| valid-under | PASS (22/22) | every contract has `metadata.valid_under.world: committed` [V 2026-09-27] |
| SHACL shapes, `pv lint --gate shapes` fails closed | PASS | 11 shapes over 11 focus nodes; pv's planted control fired and every extractor's control fired; 66 plant violations. Planting a Simpson weight of 3.9 in `src/integrate.rs` made the example exit 1 and the gate Fail (exit 1, 5 violations); restoring it gave Pass with byte-identical evidence [V 2026-09-27] |
| contracts gate, every step RUNS (ONT-G) | PASS | `scripts/contracts_gate.sh` runs lint, shapes, regen, readme, provenance and diff with separate status, and prints `contracts gate: 6 of 6 step(s) RAN, 0 FAILED`. `--self-test` runs 19 cases against a stub pv (a red proof ratchet, a failing `pv proof-status` and a red link check each turn their step red), including a mutant with the `shapes_n` probe deleted that must go green on `shapes_n=0`. CI runs both [V 2026-09-27] |
| census | PASS | `contracts/census.json` from `pv census contracts --format json`: 22 files, all kind kernel, 11 of entity type json; `external-corpora.yaml` declares the nlib corpus. The gate regenerates it and diffs it [V 2026-09-27] |
| `extract --check` | PASS | `contracts/contracts.nt` (1254 triples) and `contracts/shapes.ttl` (11 shapes) are tracked; `pv extract contracts --check` exits 0, and the gate's diff step fails when a contract changes without regenerating [V 2026-09-27] |
| generated README tables | PASS | `scripts/readme_sync.sh` writes five blocks between markers: the contract table, the SHACL summary, the proof status, the example table and the metrics. They come from `pv score`, `--gate shapes`, `contracts/proof-status.json` and `contracts/example-origins.tsv`. `--check` is the gate's readme step (22 contracts, 96 equations). Planted README drift (a hand-edited W3C count) turned it red [V 2026-09-27] |
| provenance | PASS | `scripts/lint-provenance.sh` (self-test 10/10) over `contracts/external-corpora.yaml` and this document. A file in which no claim was examined fails [V 2026-09-27] |
| ratchet | PASS | `contracts/lint-baseline.json`: `armed_gates` (14) and `armed_shapes` (11) are monotone against merge-base(HEAD, origin/main), which pv reports as "OK … (8 committed, 14 declared)". The counters `contracts_without_valid_under` 0, `contracts_without_depends_on` 11 and `ont.formal_prose` 41 only fall [V 2026-09-27] |
| example links (GH-1 b) | PASS (11 examples, 55 links) | Every example has one README row with five links, in this order: `examples/<name>.rs`; the Python original at nlib `5db3a42` with a `#L` anchor on its `def`/`class` line; `contracts/example-<name>-v1.yaml`; its `shapes.ttl` NodeShape line; its `proof-status.json` row, showing the level. `nlib.py` has no sort or Fourier code, so those two rows link `docs/book_numerical.tex` at the same commit. `scripts/example_links.sh` checks every target (self-test 17/17). Each plant turned the readme step red: a row missing its Python link, an anchor moved one line (L1687), an unfetchable upstream file, and a deleted `examples/parity.rs` [V 2026-09-27] |
| proof receipt (GH-1 c) | PASS | `contracts/proof-status.json` is `pv proof-status contracts --binding contracts/binding.yaml --verify-bindings . --format json` minus its timestamp. The regen step writes it and the diff step fails if the regenerated copy differs from the committed one. It records 22 contracts at L3, 77 obligations, 104 falsification tests, 48 Kani harness references, 0 Lean theorems and 96/96 bindings verified [V 2026-09-27] |
| Lean theorem per equation (PV-ENF-002, GH-5) | PASS (9 of 96) | 9 equations name a theorem in `lean/` that the Lean Proofs job builds (`scripts/lean_gate.sh`: `lake build`, no sorry/admit/axiom/native_decide, every cited theorem exists and is imported). `pv` counts 9 grounded theorems, and `proof_ratchet.sh` holds that count. The other 87 PV-ENF-002 findings are warnings. Each theorem is about an exact model, not about f64; `docs/lean-proofs.md` states the gap for each [V 2026-09-27] |
| proof-level ratchet (GH-1 c) | PASS | `scripts/proof_ratchet.sh` (self-test 22/22) runs in the lint step against `contracts/proof-baseline.json`. It fails if any level drops or any contract is missing, if verified bindings fall, or if phantom Kani harnesses rise above the baseline (3 after #6; 28 before), or if a `#[kani::proof]` fn is not in exactly one CI shard. Planted: removing `example-sort-v1`'s harnesses gave L3 → L2 and a red lint; renaming one `example-stats-v1` harness gave 29 phantoms and a red lint [V 2026-09-27] |
| every example an enforced contract (entity, shape, bound equations, falsification, Kani) | PASS (11/11) | see the table below [V 2026-09-27] |

## Examples as contracts

Each `examples/<name>.rs` has a `receipt()` whose checks are named after its
contract's equations. `--json` prints the receipt, and the exit code is 1 when
any check fails. `evidence/examples/<name>.json` is the tracked copy.
`contracts/example-<name>-v1.yaml` extracts that copy as a JSON entity and
closes it with a SHACL shape that requires every check to hold. CI's
"Examples" job reruns each example and diffs its receipt against the evidence.
The receipt type itself is pinned by `example-receipt-v1.yaml`
(FALSIFY-RCPT-001..007, KANI-RCPT-001).

| Example | Contract | Checks | Equations | Falsification | Kani | Mark |
|---------|----------|-------:|----------:|---------------|------|------|
| integrate | example-integrate-v1 | 7 | 6 | FALSIFY-EXINTG-001..004 | KANI-RCPT-001, KANI-INTG-001 | [V 2026-09-27] |
| solve | example-solve-v1 | 5 | 5 | FALSIFY-EXSOLVE-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| sort | example-sort-v1 | 7 | 7 | FALSIFY-EXSORT-001..004 | KANI-RCPT-001, KANI-SORT-001/002 | [V 2026-09-27] |
| stats | example-stats-v1 | 6 | 6 | FALSIFY-EXSTATS-001..004 | KANI-RCPT-001, KANI-STATS-001/002 | [V 2026-09-27] |
| matrix | example-matrix-v1 | 6 | 6 | FALSIFY-EXMATRIX-001..004 | KANI-RCPT-001, KANI-MATRIX-001 | [V 2026-09-27] |
| fourier | example-fourier-v1 | 6 | 5 | FALSIFY-EXFOURIER-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| graph | example-graph-v1 | 6 | 6 | FALSIFY-EXGRAPH-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| monte_carlo | example-monte-carlo-v1 | 6 | 3 | FALSIFY-EXMC-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| optimize | example-optimize-v1 | 5 | 4 | FALSIFY-EXOPT-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| random | example-random-v1 | 4 | 3 | FALSIFY-EXRANDOM-001..004 | KANI-RCPT-001 | [V 2026-09-27] |
| parity | example-parity-v1 | 23 | 6 | FALSIFY-EXPARITY-001..004 | KANI-RCPT-001 | [V 2026-09-27] |

Kani applies where the claim is a bounded property of a function the example
exercises. The floating-point claims in fourier, graph, monte_carlo, optimize,
random and parity are covered by the receipt harness and falsification tests,
not by a per-example BMC proof.

## N/A rows

| Row | Status | Evidence |
|-----|--------|----------|
| Lean theorem for fourier, random, monte_carlo (GH-5) | N/A | Their claims are floating-point tolerances (the fourier roundtrip within 1e-10, Parseval), integer recurrences already pinned by golden vectors and Kani (random), or probabilistic bounds on a pseudo-random stream (monte_carlo). Over ℝ the first are textbook facts, not facts about this code, and the others are not theorems. See `docs/lean-proofs.md` [V 2026-09-27] |
| `valid_under` qualifiers beyond `world` (time, agent) | N/A | The contracts describe deterministic numerics over committed code; one world, `committed`, is declared in `ontology.yaml`, and no claim depends on time or agent [C ONT-001 v4.17] |
| `llm` reader in `ontology.yaml` | N/A | No contract here is read by an LLM extractor; the readers map omits that key [C ONT-001 v4.17] |

## Findings

| Finding | Status | Evidence |
|---------|--------|----------|
| `examples/optimize.rs`: Newton from x0 = 0.5 converged to the local **maximum** x = 0 of the quartic, which the old example printed as the minimum | FIXED | Newton now starts at x0 = 1.0. FALSIFY-EXOPT-004 keeps the old start and asserts that it lands on the maximum [V 2026-09-27] |
| `examples/graph.rs` claimed a "Fig. 3.12 style" graph | FIXED | Claim removed; the receipt checks computed distances and the MST weight instead [V 2026-09-27] |
| `src/matrix.rs` `IndexMut` writes through a pointer cast from `as_slice().as_ptr()` (a shared borrow), which is unsound | OPEN | Tracked on GH-1 [V 2026-09-27] |
| `pv proof-status` credits L3 for declared Kani harnesses without checking that they exist. Ten module contracts named 28 harnesses (of 48 references) that no `#[kani::proof]` fn defined, so L3 was Kani-backed for only 12 of 22 contracts | OPEN (3 left) | [aprender#4531](https://github.com/paiml/aprender/issues/4531) for pv. [#6](https://github.com/paiml/rust-mdipierro-nlib/issues/6) wrote 25 of the 28, each verified locally and run by a CI shard; 21 of 22 contracts are now Kani-backed. graph-algorithms-v1's three stay phantom and unproven: KANI-GRPH-001 `verify_dijkstra_optimality`, KANI-GRPH-002 `verify_mst_edge_count` and KANI-GRPH-003 `verify_bfs_levels` did not finish in 30 minutes within an 18 GB memory cap, even bounded to 3, 2 and 2 concrete 3-vertex graphs. The f64 weights and vertex lists pass through `Vec<Vec<_>>` heap objects, where CBMC loses their concrete values and bit-blasts the whole run. The declarations stay in place, and the ratchet holds the phantom count at 3 or below [V 2026-09-27] |
| The org ruleset "Green Main" requires one check, `gate`, and until GH-1 that was the first job (fmt, clippy, `cargo test --lib`). Kani, Examples, Contract Validation, parity, golden vectors and PR-diff mutants were not required: a PR could merge with any of them red | FIXED | `gate` is now the last job: `if: always()`, it needs every blocking job, and `scripts/ci_gate.sh --results` fails unless each succeeded. `--check-workflow` fails if a job is neither needed nor advisory (self-test 15/15). Planted: a failing step in the Kani BMC job gave lint-test success, kani failure, `gate` failure, and the PR's merge state BLOCKED ([run](https://github.com/paiml/rust-mdipierro-nlib/actions/runs/36314753804)) [V 2026-09-27] |
| The ruleset lets OrganizationAdmin bypass it ("always"), and the account that merges here is an org admin | OPEN (by policy) | No merge used the bypass: PRs #2, #3, #4 and #7 had `gate` and every blocking check green before their merge time (check runs on each head commit; PMAT Comply, advisory, failed on all four). `gh pr merge --admin` is never used. The 25 commits of 2026-04-07 went to `main` without a PR [V 2026-09-27] |
| README.md mixed generated blocks with prose written in place, so a hand edit outside the markers passed the gate, and the "What is enforced" rows were strings in the generator | FIXED (one gap) | README.md is now generated whole: `scripts/readme_sync.sh` renders it from `docs/README.md.in`, fills every block, and `--check` fails on any byte that differs. The enforcement rows are claim entities in `evidence/enforcement/claims.json`, closed by the SHACL shape in `contracts/enforcement-claims-v1.yaml` (armed); an enforced claim without a plant run, or an advisory check presented as enforced, violates it. Plants: one README byte hand-edited turned only Contract Validation and `gate` red (run 36316337423); the advisory PMAT claim moved into `enforced` did the same, the shape naming `job: "comply"` (run 36316963827). Gap: the prose in `docs/README.md.in` is still written by hand [V 2026-09-27] |
| Binding `uniform_chi2 → nlib::random::next_f64` resolves to `Lcg::next_f64`, the first impl, not `Mt19937::next_f64` | OPEN (upstream) | [aprender#4521](https://github.com/paiml/aprender/issues/4521) [C aprender#4521] |
| The codebase grade includes pv's drift dimension: a contract whose last commit is newer than `contracts/binding.yaml`'s counts as stale. Committing the 22 contracts without the binding dropped the grade from A (0.92) to C (0.72), drift 0 | BY DESIGN | `binding.yaml` now records when its bindings were last re-verified and is committed with any contract change; the README check turns the gate red otherwise [V 2026-09-27] |
| `pv validate contracts/ontology.yaml` fails ("missing field `metadata`"): the Σ file is not a contract | BY DESIGN | CI validates contracts through `pv lint` (validate gate), which reads `ontology.yaml` as Σ; the per-file loop is gone [V 2026-09-27] |

## CI gates and their scope

| Gate | Status | Why |
|------|--------|-----|
| `gate` (the one check the ruleset requires) | blocking | The last job: `if: always()`, `needs:` every blocking job, fails unless each result is `success`. Only this check blocks a merge; the others block through it [V 2026-09-27] |
| Kani BMC | blocking, 60 min timeout per shard | 32 harnesses in 4 matrix shards (core, numerical, sort-cholesky, inverse), all verified locally with Kani 0.68; the slowest are inverse 785 s, quicksort length 309 s, sort permutation 210 s and quicksort sorted 211 s. `scripts/proof_ratchet.sh` fails the contracts gate unless every `#[kani::proof]` fn is in exactly one shard. Stubs, each stated at its definition: libm's `sin_cos` at 0 and ±π, an exact integer `f64::sqrt` (tested against std's on 220 000 inputs), `powi(x, 2) = x * x`, all-zero `cpuid`, and a constant chi-square p-value. The old symbolic-length quicksort harness (unwind 8) ran into the 6 h job limit on main [V 2026-09-27] |
| Mutation testing | blocking on the PR diff (`--in-diff`); full sweep advisory on `main` | Full sweep on main had 114 missed mutants before GH-1 (fourier, graph, integrate, matrix, monte_carlo, optimize, random). The backlog is being killed module by module; the sweep becomes blocking at zero [V 2026-09-27] |
| PMAT comply | advisory | Failing checks are repository governance (branch protection needs an admin, `deny.toml`, `build.rs`, roadmap/spec schema, TDG), not contract enforcement |
| Contracts gate (ONT-G) | blocking | "Contract Validation" job: `bash scripts/contracts_gate.sh --self-test`, then `bash scripts/contracts_gate.sh`; any failed step fails the job, and every step runs [V 2026-09-27] |
| Example receipts | blocking | "Examples" job: `cargo test --test examples`, then each example with and without `--json`; the receipt must equal `evidence/examples/<name>.json` |
