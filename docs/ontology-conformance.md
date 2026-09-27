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
| proof-level ratchet (GH-1 c) | PASS | `scripts/proof_ratchet.sh` (self-test 16/16) runs in the lint step against `contracts/proof-baseline.json`. It fails if any level drops or any contract is missing, if verified bindings fall, or if phantom Kani harnesses rise above 28. Planted: removing `example-sort-v1`'s harnesses gave L3 → L2 and a red lint; renaming one `example-stats-v1` harness gave 29 phantoms and a red lint [V 2026-09-27] |
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
| Lean theorem per equation (PV-ENF-002, theorem-pairing gate) | N/A | This crate proves its bounded properties with Kani (7 harnesses in `src/kani_harnesses.rs`); there is no Lean theorem base, so theorem-pairing is not armed. The 96 PV-ENF-002 findings are warnings, not failures [V 2026-09-27] |
| `valid_under` qualifiers beyond `world` (time, agent) | N/A | The contracts describe deterministic numerics over committed code; one world, `committed`, is declared in `ontology.yaml`, and no claim depends on time or agent [C ONT-001 v4.17] |
| `llm` reader in `ontology.yaml` | N/A | No contract here is read by an LLM extractor; the readers map omits that key [C ONT-001 v4.17] |

## Findings

| Finding | Status | Evidence |
|---------|--------|----------|
| `examples/optimize.rs`: Newton from x0 = 0.5 converged to the local **maximum** x = 0 of the quartic, which the old example printed as the minimum | FIXED | Newton now starts at x0 = 1.0. FALSIFY-EXOPT-004 keeps the old start and asserts that it lands on the maximum [V 2026-09-27] |
| `examples/graph.rs` claimed a "Fig. 3.12 style" graph | FIXED | Claim removed; the receipt checks computed distances and the MST weight instead [V 2026-09-27] |
| `src/matrix.rs` `IndexMut` writes through a pointer cast from `as_slice().as_ptr()` (a shared borrow), which is unsound | OPEN | Tracked on GH-1 [V 2026-09-27] |
| `pv proof-status` credits L3 for declared Kani harnesses without checking that they exist. Ten module contracts name 28 harnesses (of 48 references) that no `#[kani::proof]` fn defines, so L3 is Kani-backed for only 12 of 22 contracts | OPEN | [aprender#4531](https://github.com/paiml/aprender/issues/4531) for pv. [#6](https://github.com/paiml/rust-mdipierro-nlib/issues/6) tracks writing the harnesses, and the declarations stay in place. The README states the gap, and the ratchet holds the phantom count at 28 or below [V 2026-09-27] |
| Binding `uniform_chi2 → nlib::random::next_f64` resolves to `Lcg::next_f64`, the first impl, not `Mt19937::next_f64` | OPEN (upstream) | [aprender#4521](https://github.com/paiml/aprender/issues/4521) [C aprender#4521] |
| The codebase grade includes pv's drift dimension: a contract whose last commit is newer than `contracts/binding.yaml`'s counts as stale. Committing the 22 contracts without the binding dropped the grade from A (0.92) to C (0.72), drift 0 | BY DESIGN | `binding.yaml` now records when its bindings were last re-verified and is committed with any contract change; the README check turns the gate red otherwise [V 2026-09-27] |
| `pv validate contracts/ontology.yaml` fails ("missing field `metadata`"): the Σ file is not a contract | BY DESIGN | CI validates contracts through `pv lint` (validate gate), which reads `ontology.yaml` as Σ; the per-file loop is gone [V 2026-09-27] |

## CI gates and their scope

| Gate | Status | Why |
|------|--------|-----|
| Kani BMC | blocking, 60 min timeout | 7 harnesses, all verified locally (quicksort sorted 211 s, quicksort length 309 s, correlation 142 s, variance 10 s, transpose 8 s, within 1 s, simpson odd n 2 s). The old symbolic-length quicksort harness (unwind 8) ran into the 6 h job limit on main [V 2026-09-27] |
| Mutation testing | blocking on the PR diff (`--in-diff`); full sweep advisory on `main` | Full sweep on main had 114 missed mutants before GH-1 (fourier, graph, integrate, matrix, monte_carlo, optimize, random). The backlog is being killed module by module; the sweep becomes blocking at zero [V 2026-09-27] |
| PMAT comply | advisory | Failing checks are repository governance (branch protection needs an admin, `deny.toml`, `build.rs`, roadmap/spec schema, TDG), not contract enforcement |
| Contracts gate (ONT-G) | blocking | "Contract Validation" job: `bash scripts/contracts_gate.sh --self-test`, then `bash scripts/contracts_gate.sh`; any failed step fails the job, and every step runs [V 2026-09-27] |
| Example receipts | blocking | "Examples" job: `cargo test --test examples`, then each example with and without `--json`; the receipt must equal `evidence/examples/<name>.json` |
