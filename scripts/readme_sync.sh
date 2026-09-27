#!/usr/bin/env bash
# readme_sync.sh — the README's contract table, SHACL summary, proof status, example table and
# metrics are GENERATED, never hand-written (ONT-001 v4.17, ONT-G step "readme"; GH-1).
#
# It rewrites exactly the lines BETWEEN each marker pair, and nothing else:
#
#     <!-- CONTRACT_TABLE_START -->   ...   <!-- CONTRACT_TABLE_END -->
#     <!-- SHACL_SUMMARY_START -->    ...   <!-- SHACL_SUMMARY_END -->
#     <!-- PROOF_STATUS_START -->     ...   <!-- PROOF_STATUS_END -->
#     <!-- EXAMPLE_TABLE_START -->    ...   <!-- EXAMPLE_TABLE_END -->
#     <!-- CONTRACT_METRICS_START --> ...   <!-- CONTRACT_METRICS_END -->
#
# Every number comes from one instrument:
#   - per-contract grades and the codebase grade: `pv score --format json`
#   - equations per contract: the keys of each contract's `equations:` block
#   - shapes, focus nodes, controls, bindings: `pv lint --gate shapes --format json`
#   - proof levels and counts: contracts/proof-status.json, the committed
#     `pv proof-status --verify-bindings --format json` receipt (the gate's regen step writes it)
#   - each example's Python original: contracts/example-origins.tsv at the ref pinned in
#     contracts/external-corpora.yaml; shape and proof-status anchors are looked up line by line
#   - Kani harnesses: `#[kani::proof]` in src/kani_harnesses.rs
#
# Modes:
#   --write  rewrite every block in README.md in place (idempotent)
#   --print  print the rendered blocks; write nothing
#   --check  exit 0 iff README.md already equals what --write produces, else 1
#
# A README without every marker pair is exit 3, never a silent no-op. A measure
# that reads empty or zero is a failure (ONT R-2: zero is a decline).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
README="${README_PATH:-"$REPO_ROOT/README.md"}"
PV="${PV:-pv}"
PROOF=contracts/proof-status.json

die() {
    printf 'FAIL readme_sync: %s\n' "$1" >&2
    exit "${2:-1}"
}

usage() {
    printf 'usage: bash scripts/readme_sync.sh [--write|--print|--check]\n' >&2
    exit 2
}

# The names of a contract's equations: two-space keys under `equations:`.
equations_of() {
    awk '/^equations:/ { on = 1; next }
         /^[^ #]/      { on = 0 }
         on && /^  [A-Za-z0-9_]+:/ { sub(/^  /, ""); sub(/:.*/, ""); printf "%s%s", sep, $0; sep = ", " }' "$1"
}

nonzero() { # nonzero <name> <value>
    case "$2" in
        '' | *[!0-9]*) die "$1 read '$2', not a number" ;;
    esac
    [ "$2" -gt 0 ] || die "$1 is 0: a broken measurement, not a README to regenerate"
}

measure() {
    cd "$REPO_ROOT" || exit 3
    local tmp
    tmp="$(mktemp -d)"
    trap 'rm -rf "${tmp:?}"' EXIT
    "$PV" score contracts/ --binding contracts/binding.yaml --format json >"$tmp/score.raw" 2>/dev/null \
        || die "pv score failed"
    jq -s '.' "$tmp/score.raw" >"$tmp/score.json" || die "pv score printed no JSON"
    "$PV" lint contracts/ --gate shapes --format json >"$tmp/shapes.json" 2>/dev/null \
        || die "pv lint --gate shapes did not pass"

    N_CONTRACTS=$(jq -r '.[0].contracts' "$tmp/score.json")
    MEAN=$(jq -r '.[0].mean_score * 100 | round / 100' "$tmp/score.json")
    MEAN_GRADE=$(jq -r '.[0].mean_grade' "$tmp/score.json")
    CB_GRADE=$(jq -r '.[1].codebase.grade' "$tmp/score.json")
    CB_SCORE=$(jq -r '.[1].codebase.composite * 100 | round / 100' "$tmp/score.json")
    SHAPES=$(jq -r '.shapes_n' "$tmp/shapes.json")
    FOCUS=$(jq -r '.focus_nodes_n' "$tmp/shapes.json")
    RESOLVED=$(jq -r '.symbols_resolved' "$tmp/shapes.json")
    UNRESOLVED=$(jq -r '.symbols_unresolved' "$tmp/shapes.json")
    PLANT_V=$(jq -r '.plant_violations' "$tmp/shapes.json")
    PC_SHAPE=$(jq -r '.pc_shape' "$tmp/shapes.json")
    PC_FIRED=$(jq -r '[.pc_extract[] | select(. == "fired")] | length' "$tmp/shapes.json")
    PC_N=$(jq -r '.pc_extract | length' "$tmp/shapes.json")
    W3C=$(jq -r '.w3c_cases_passed' "$tmp/shapes.json")
    W3C_N=$(jq -r '.w3c_cases_n' "$tmp/shapes.json")
    TRIPLES=$(jq -r '.triples' "$tmp/shapes.json")
    [ "$PC_SHAPE" = fired ] || die "pv's planted shape control did not fire ($PC_SHAPE)"
    [ "$PC_FIRED" = "$PC_N" ] || die "$PC_FIRED of $PC_N extractor controls fired"
    nonzero "planted violations" "$PLANT_V"
    nonzero "extractor controls" "$PC_N"
    nonzero "W3C cases" "$W3C"
    nonzero triples "$TRIPLES"
    KANI=$(grep -cF '#[kani::proof]' src/kani_harnesses.rs || true)
    nonzero contracts "$N_CONTRACTS"
    nonzero shapes "$SHAPES"
    nonzero "focus nodes" "$FOCUS"
    nonzero "resolved symbols" "$RESOLVED"
    nonzero "Kani harnesses" "$KANI"

    TABLE="$tmp/table.md"
    N_EQ=0
    {
        printf '| Contract | Grade | Spec | Falsify | Kani | Bind | Equations |\n'
        printf '|----------|-------|------|---------|------|------|-----------|\n'
        local stem cf eqs n
        while IFS=$'\t' read -r stem grade score spec fals kani bind; do
            cf="contracts/${stem}.yaml"
            eqs=$(equations_of "$cf")
            [[ -n "$eqs" ]] || die "$cf has no equations"
            n=$(printf '%s' "$eqs" | awk -F', ' '{ print NF }')
            N_EQ=$((N_EQ + n))
            printf '| %s | %s (%s) | %s | %s | %s | %s | %s |\n' \
                "$stem" "$grade" "$score" "$spec" "$fals" "$kani" "$bind" "$eqs"
        done < <(jq -r '.[0].scores | sort_by(.stem)[] |
            [.stem, .grade] + ([.composite, .spec_depth, .falsification_coverage, .kani_coverage, .binding_coverage]
            | map(. * 100 | round / 100)) | @tsv' "$tmp/score.json")
    } >"$TABLE"
    nonzero equations "$N_EQ"
    TABLE_TEXT="$(cat "$TABLE")"
    rm -rf "${tmp:?}"
    trap - EXIT
    measure_examples
    measure_proofs
}

# The per-example table (GH-1 scope b, c): links in the order rust, python, contract, shape, proof
# status. Every link target is looked up here, and a lookup that finds nothing is a failure; the
# gate's readme step then verifies each target independently (scripts/example_links.sh).
measure_examples() {
    local origins=contracts/example-origins.tsv
    local ref ex file sym line what stem shape_l proof_l level rows=""
    [ -f "$PROOF" ] || die "$PROOF is missing; the gate's regen step writes it"
    [ -f "$origins" ] || die "$origins is missing"
    ref=$(sed -n 's/^[[:space:]]*ref:[[:space:]]*\([0-9a-f]\{40\}\).*/\1/p' contracts/external-corpora.yaml | head -1)
    [ -n "$ref" ] || die "contracts/external-corpora.yaml pins no 40-hex ref"
    N_EX=0
    while IFS=$'\t' read -r ex file sym line _ what; do
        stem="example-${ex//_/-}-v1"
        [ -f "examples/$ex.rs" ] || die "$origins names $ex, but examples/$ex.rs does not exist"
        [ -f "contracts/$stem.yaml" ] || die "contracts/$stem.yaml does not exist"
        shape_l=$(grep -nF "shape/$stem> a sh:NodeShape" contracts/shapes.ttl | cut -d: -f1)
        [ "$(printf '%s' "$shape_l" | grep -c .)" = 1 ] || die "contracts/shapes.ttl declares no single shape for $stem"
        proof_l=$(grep -nF "\"stem\": \"$stem\"" "$PROOF" | cut -d: -f1)
        [ "$(printf '%s' "$proof_l" | grep -c .)" = 1 ] || die "$PROOF has no single row for $stem"
        level=$(jq -r --arg s "$stem" '.contracts[] | select(.stem == $s) | .proof_level' "$PROOF")
        [ -n "$level" ] || die "$PROOF records no level for $stem"
        rows+=$(printf '| [%s](examples/%s.rs) | [`%s`](https://github.com/mdipierro/nlib/blob/%s/%s#L%s) | [%s](contracts/%s.yaml) · [shape](contracts/shapes.ttl#L%s) | [%s](%s#L%s) | `cargo run --example %s` | %s |' \
            "$ex" "$ex" "$sym" "$ref" "$file" "$line" "$stem" "$stem" "$shape_l" "$level" "$PROOF" "$proof_l" "$ex" "$what")
        rows+=$'\n'
        N_EX=$((N_EX + 1))
    done < <(grep -v '^#' "$origins")
    nonzero examples "$N_EX"
    local n_rs
    n_rs=$(find examples -maxdepth 1 -name '*.rs' | grep -c . || true)
    [ "$n_rs" = "$N_EX" ] || die "$origins names $N_EX example(s), examples/ holds $n_rs"
    EXAMPLE_TEXT=$(
        printf '| Example | Python original (nlib @ `%s`) | Contract · SHACL shape | Proof | Run | What it does |\n' "${ref:0:7}"
        printf '|---------|-----------------|------------------------|-------|-----|--------------|\n'
        printf '%s' "$rows"
    )
}

# The proof summary: every number is a field of the committed receipt contracts/proof-status.json.
measure_proofs() {
    PROOF_TEXT=$(jq -r '
        def rank: ltrimstr("L") | tonumber;
        .totals as $t
        | ([.contracts[].proof_level] | min_by(rank)) as $min
        | ([.contracts[] | select(.proof_level == $min)] | length) as $at_min
        | (["tested"] + (if $t.kani_harnesses > 0 then ["Kani model-checked"] else [] end)
                      + (if $t.lean_grounded > 0 then ["Lean-proved"] else [] end)
          | join(" + ")) as $how
        | "**`pv proof-status` level: \($min) (\($how)), \($t.bindings_implemented)/\($t.bindings_total) bindings verified.**",
          "",
          "| Measure (`pv proof-status --verify-bindings`) | Value |",
          "|--------|-------|",
          "| Contracts | \($t.contracts) (\($at_min) at \($min), the lowest level) |",
          "| Levels | \([.contracts[].proof_level] | group_by(.) | map("\(.[0]): \(length)") | join(", ")) |",
          "| Proof obligations | \($t.obligations) (\($t.not_applicable) N/A) |",
          "| Falsification tests | \($t.falsification_tests) |",
          "| Kani harnesses | \($t.kani_harnesses) |",
          "| Lean theorems proved (grounded in an equation) | \($t.lean_proved) (\($t.lean_grounded)) |",
          "| Bindings verified in source | \($t.bindings_implemented)/\($t.bindings_total) |"
        ' "$PROOF" 2>/dev/null) || die "$PROOF is not a pv proof-status receipt"
    [ -n "$PROOF_TEXT" ] || die "$PROOF holds no contracts"

    # pv counts a declared Kani harness without looking for it; say how many exist.
    local kt n_k backed phantom stale
    kt=$(bash scripts/proof_ratchet.sh --kani-table) || die "proof_ratchet.sh --kani-table failed"
    n_k=$(printf '%s\n' "$kt" | grep -c . || true)
    nonzero "contracts declaring Kani harnesses" "$n_k"
    backed=$(awk -F'\t' '$2 == $3' <<<"$kt" | grep -c . || true)
    phantom=$(awk -F'\t' '{ s += $2 - $3 } END { print s + 0 }' <<<"$kt")
    stale=$(awk -F'\t' '$2 != $3 { printf "%s%s (%s of %s)", sep, $1, $3, $2; sep = ", " }' <<<"$kt")
    PROOF_TEXT+=$'\n'"| Contracts whose every declared Kani harness exists in \`src/\` | $backed of $n_k |"
    PROOF_TEXT+=$'\n'"| Declared Kani harnesses with no \`#[kani::proof]\` function (phantom) | $phantom |"
    if [ "$phantom" -gt 0 ]; then
        PROOF_TEXT=${PROOF_TEXT/'.**'/"; the Kani harnesses exist for $backed of the $n_k contracts that declare them.**"}
        PROOF_TEXT+=$'\n\n'"pv counts a declared \`harness:\` toward L3 without checking that it exists, so L3 is"
        PROOF_TEXT+=$'\n'"backed by a real Kani proof only for the $backed contracts above. These contracts name"
        PROOF_TEXT+=$'\n'"harnesses that are missing (present of declared): $stale."
        PROOF_TEXT+=$'\n'"\`scripts/proof_ratchet.sh\` fails the gate if the phantom count rises."
    fi
}

render_table() {
    printf '%s\n\n' "$TABLE_TEXT"
    printf '**%s contracts, %s equations; mean contract score %s (%s); codebase grade %s (%s).**\n' \
        "$N_CONTRACTS" "$N_EQ" "$MEAN" "$MEAN_GRADE" "$CB_GRADE" "$CB_SCORE"
}

render_metrics() {
    printf '| Metric | Value |\n'
    printf '|--------|-------|\n'
    printf '| Contracts | %s |\n' "$N_CONTRACTS"
    printf '| Equations | %s |\n' "$N_EQ"
    printf '| Bound symbols (extract:code) | %s resolved, %s unresolved |\n' "$RESOLVED" "$UNRESOLVED"
    printf '| SHACL shapes | %s, over %s focus nodes |\n' "$SHAPES" "$FOCUS"
    printf '| Kani BMC harnesses | %s |\n' "$KANI"
    printf '| Contract grade | %s (%s mean) |\n' "$MEAN_GRADE" "$MEAN"
    printf '| Codebase grade | %s (%s) |\n' "$CB_GRADE" "$CB_SCORE"
    printf '| External deps | 1 (aprender only) |\n'
}

# replace_block <file> <name> <body-file>: the lines between the markers become the body.
replace_block() {
    local start="<!-- $2_START -->" end="<!-- $2_END -->"
    [ "$(grep -cxF "$start" "$1")" = 1 ] && [ "$(grep -cxF "$end" "$1")" = 1 ] \
        || die "README carries no single $2 marker pair" 3
    awk -v s="$start" -v e="$end" -v body="$3" '
        $0 == s { print; while ((getline line < body) > 0) print line; skip = 1; next }
        $0 == e { skip = 0 }
        !skip' "$1"
}

render_shapes() {
    printf '| What `pv lint contracts/ --gate shapes` measured | Value |\n'
    printf '|--------|-------|\n'
    printf '| SHACL shapes (one per example contract) | %s |\n' "$SHAPES"
    printf '| Focus nodes (example `--json` receipts in `evidence/examples/`) | %s |\n' "$FOCUS"
    printf '| RDF triples checked | %s |\n' "$TRIPLES"
    printf '| Violations | 0 (any violation is exit 1) |\n'
    printf '| Planted control: pv'"'"'s own broken receipt must violate | %s, %s violations |\n' "$PC_SHAPE" "$PLANT_V"
    printf '| Extractor controls fired | %s of %s |\n' "$PC_FIRED" "$PC_N"
    printf '| W3C SHACL conformance cases | %s of %s |\n' "$W3C" "$W3C_N"
    printf '| Bound symbols resolved in source | %s (%s unresolved) |\n' "$RESOLVED" "$UNRESOLVED"
}

# rendered_readme <out>: every generated block, rewritten in turn.
rendered_readme() {
    local cur nxt body name
    cur="$(mktemp)"
    nxt="$(mktemp)"
    body="$(mktemp)"
    cp "$README" "$cur"
    for name in CONTRACT_TABLE SHACL_SUMMARY PROOF_STATUS EXAMPLE_TABLE CONTRACT_METRICS; do
        {
            printf '\n'
            case "$name" in
                CONTRACT_TABLE) render_table ;;
                SHACL_SUMMARY) render_shapes ;;
                PROOF_STATUS) printf '%s\n' "$PROOF_TEXT" ;;
                EXAMPLE_TABLE) printf '%s\n' "$EXAMPLE_TEXT" ;;
                CONTRACT_METRICS) render_metrics ;;
            esac
            printf '\n'
        } >"$body"
        replace_block "$cur" "$name" "$body" >"$nxt"
        cp "$nxt" "$cur"
    done
    cp "$cur" "$1"
    rm -f "$cur" "$nxt" "$body"
}

mode="${1:---write}"
case "$mode" in
    --write | --print | --check) ;;
    *) usage ;;
esac
[ -f "$README" ] || die "no README at $README" 3
measure

case "$mode" in
    --print)
        render_table
        printf '\n'
        render_shapes
        printf '\n%s\n\n%s\n\n' "$PROOF_TEXT" "$EXAMPLE_TEXT"
        render_metrics
        ;;
    --write)
        out="$(mktemp "$README.XXXXXX")"
        rendered_readme "$out"
        mv "$out" "$README"
        printf 'readme_sync: wrote %s contracts, %s equations\n' "$N_CONTRACTS" "$N_EQ"
        ;;
    --check)
        out="$(mktemp)"
        rendered_readme "$out"
        if cmp -s "$out" "$README"; then
            rm -f "$out"
            printf 'readme_sync: README.md in sync (%s contracts, %s equations)\n' "$N_CONTRACTS" "$N_EQ"
        else
            diff -u "$README" "$out" >&2 || true
            rm -f "$out"
            die "README.md is stale; run: bash scripts/readme_sync.sh --write"
        fi
        ;;
esac
