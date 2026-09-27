#!/usr/bin/env bash
# readme_sync.sh — the README's contract table and metrics are GENERATED, never
# hand-written (ONT-001 v4.17, ONT-G step "readme"; GH-1).
#
# It rewrites exactly the lines BETWEEN each marker pair, and nothing else:
#
#     <!-- CONTRACT_TABLE_START -->   ...   <!-- CONTRACT_TABLE_END -->
#     <!-- CONTRACT_METRICS_START --> ...   <!-- CONTRACT_METRICS_END -->
#
# Every number comes from one instrument:
#   - per-contract grades and the codebase grade: `pv score --format json`
#   - equations per contract: the keys of each contract's `equations:` block
#   - shapes, focus nodes, bindings: `pv lint --gate shapes --format json`
#   - Kani harnesses: `#[kani::proof]` in src/kani_harnesses.rs
#
# Modes:
#   --write  rewrite both blocks in README.md in place (idempotent)
#   --print  print the rendered blocks; write nothing
#   --check  exit 0 iff README.md already equals what --write produces, else 1
#
# A README without both marker pairs is exit 3, never a silent no-op. A measure
# that reads empty or zero is a failure (ONT R-2: zero is a decline).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
README="${README_PATH:-"$REPO_ROOT/README.md"}"
PV="${PV:-pv}"

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

rendered_readme() { # rendered_readme <out>
    local t m mid
    t="$(mktemp)"
    m="$(mktemp)"
    mid="$(mktemp)"
    { printf '\n'; render_table; printf '\n'; } >"$t"
    { printf '\n'; render_metrics; printf '\n'; } >"$m"
    replace_block "$README" CONTRACT_TABLE "$t" >"$mid"
    replace_block "$mid" CONTRACT_METRICS "$m" >"$1"
    rm -f "$t" "$m" "$mid"
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
