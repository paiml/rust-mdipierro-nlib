#!/usr/bin/env bash
# proof_ratchet.sh — proof levels only rise (GH-1 scope c).
#
# contracts/proof-status.json is the committed proof receipt: `pv proof-status contracts --binding
# contracts/binding.yaml --verify-bindings . --format json` with its wall-clock `timestamp` removed,
# so the same contracts always give the same bytes. contracts/proof-baseline.json records each
# contract's level and the verified binding count. This script fails when
#   - a contract in the baseline is gone, or its level is below the baseline,
#   - a contract in the receipt is missing from the baseline (every level is ratcheted from day one),
#   - fewer bindings are verified than the baseline records, or any binding is unverified,
#   - fewer Lean theorems are grounded in a contract equation than the baseline's lean_grounded (GH-5).
# A level above the baseline passes and is printed, so the baseline can be raised in the same PR.
#
# It also fails when the number of Kani harness names the contracts declare, but no
# `#[kani::proof]` function under src/ defines, rises above the baseline's phantom_kani_harnesses.
#
# usage: proof_ratchet.sh [--self-test | --kani-table | receipt.json [baseline.json]]
set -euo pipefail

# kani_table [dir] — one row per contract that declares Kani harnesses: stem, declared, present.
# pv proof-status counts a declared `harness:` toward L3 without looking for it, so this checks
# each name against the `#[kani::proof]` functions under src/: a harness that is not there is a
# phantom, and a phantom proves nothing.
kani_table() {
    local dir=${1:-.} have f names declared present
    have=$(find "$dir/src" -name '*.rs' -exec awk '/#\[kani::proof\]/ { p = 1; next }
        p && match($0, /fn [A-Za-z0-9_]+/) { print substr($0, RSTART + 3, RLENGTH - 3); p = 0 }' {} + | sort -u)
    for f in "$dir"/contracts/*.yaml; do
        names=$(grep -oE '^[[:space:]]*harness:[[:space:]]*[A-Za-z0-9_]+' "$f" 2>/dev/null | awk '{ print $NF }' | sort || true)
        [ -n "$names" ] || continue
        declared=$(grep -c . <<<"$names")
        present=$(comm -12 <(printf '%s\n' "$names") <(printf '%s\n' "$have") | grep -c . || true)
        printf '%s\t%s\t%s\n' "$(basename "$f" .yaml)" "$declared" "$present"
    done
}

# kani_ratchet <baseline> [dir] — the phantom harness count may fall, never rise.
kani_ratchet() {
    local max rows phantom backed n
    max=$(jq -r '.phantom_kani_harnesses // empty' "$1" 2>/dev/null) || max=
    [[ "$max" =~ ^[0-9]+$ ]] || {
        echo "proof-ratchet: FAIL $1 records no phantom_kani_harnesses"
        return 1
    }
    rows=$(kani_table "${2:-.}")
    [ -n "$rows" ] || {
        echo "proof-ratchet: FAIL no contract declares a Kani harness (nothing measured is a decline)"
        return 1
    }
    phantom=$(awk -F'\t' '{ s += $2 - $3 } END { print s + 0 }' <<<"$rows")
    backed=$(awk -F'\t' '$2 == $3' <<<"$rows" | grep -c . || true)
    n=$(grep -c . <<<"$rows")
    echo "proof-ratchet: kani: $backed of $n contract(s) with harnesses have every one in src/; $phantom phantom harness reference(s), baseline $max"
    if [ "$phantom" -gt "$max" ]; then
        awk -F'\t' '$2 != $3 { printf "proof-ratchet:   %s declares %s, %s present\n", $1, $2, $3 }' <<<"$rows"
        echo "proof-ratchet: FAIL phantom Kani harnesses rose from $max to $phantom"
        return 1
    fi
    [ "$phantom" -lt "$max" ] && echo "proof-ratchet: RISE phantom harnesses fell $max -> $phantom (lower the baseline)"
    return 0
}

ratchet() { # ratchet <receipt> <baseline>
    local out
    jq -e '.contracts | length > 0' "$1" >/dev/null 2>&1 || {
        echo "proof-ratchet: FAIL $1 holds no contracts (nothing measured is a decline, not a pass)"
        return 1
    }
    jq -e '.levels | length > 0' "$2" >/dev/null 2>&1 || {
        echo "proof-ratchet: FAIL $2 records no levels"
        return 1
    }
    out=$(jq -r --slurpfile base "$2" -f /dev/stdin "$1" <<'JQ'
        def rank: ltrimstr("L") | tonumber;
        ($base[0]) as $b
        | (.contracts | map({key: .stem, value: .proof_level}) | from_entries) as $now
        | [ ($b.levels | to_entries[]
              | if ($now[.key] == null) then "FAIL \(.key): in the baseline at \(.value), gone from the receipt"
                elif ($now[.key] | rank) < (.value | rank) then "FAIL \(.key): \(.value) -> \($now[.key]) (a level dropped)"
                elif ($now[.key] | rank) > (.value | rank) then "RISE \(.key): \(.value) -> \($now[.key]) (raise the baseline)"
                else empty end),
            ($now | to_entries[] | select($b.levels[.key] == null)
              | "FAIL \(.key): at \(.value) but not in the baseline (record it)"),
            (if .totals.bindings_implemented < $b.bindings_implemented
               then "FAIL bindings verified \(.totals.bindings_implemented) < baseline \($b.bindings_implemented)" else empty end),
            (if (.totals.lean_grounded // 0) < ($b.lean_grounded // 0)
               then "FAIL Lean theorems grounded in an equation \(.totals.lean_grounded // 0) < baseline \($b.lean_grounded)" else empty end),
            (if (.totals.lean_grounded // 0) > ($b.lean_grounded // 0)
               then "RISE Lean theorems grounded \($b.lean_grounded // 0) -> \(.totals.lean_grounded) (raise the baseline)" else empty end),
            (if .totals.bindings_implemented != .totals.bindings_total
               then "FAIL \(.totals.bindings_total - .totals.bindings_implemented) binding(s) not verified in source" else empty end),
            "levels: \([.contracts[].proof_level] | group_by(.) | map("\(.[0])=\(length)") | join(" ")); bindings \(.totals.bindings_implemented)/\(.totals.bindings_total) verified; Lean theorems grounded \(.totals.lean_grounded // 0)"
          ] | .[]
JQ
) || {
        echo "proof-ratchet: FAIL could not read $1 against $2"
        return 1
    }
    printf 'proof-ratchet: %s\n' "${out//$'\n'/$'\n'proof-ratchet: }"
    ! grep -q '^FAIL' <<<"$out"
}

self_test() {
    local d pass=0 fail=0
    d=$(mktemp -d)
    receipt() { # receipt <level of a> <level of b> <implemented> <total>
        printf '{"contracts":[{"stem":"a","proof_level":"%s"},{"stem":"b","proof_level":"%s"}],"totals":{"bindings_implemented":%s,"bindings_total":%s}}\n' "$@" >"$d/r.json"
    }
    printf '{"levels":{"a":"L3","b":"L3"},"bindings_implemented":4}\n' >"$d/b.json"
    expect() { # expect <0|1> <message>
        local got=0
        ratchet "$d/r.json" "$d/b.json" >"$d/out" 2>&1 || got=1
        if [ "$got" = "$1" ]; then
            pass=$((pass + 1))
            printf 'ok   %s\n' "$2"
        else
            fail=$((fail + 1))
            printf 'FAIL %s\n' "$2"
            sed 's/^/     /' "$d/out"
        fi
    }
    receipt L3 L3 4 4
    expect 0 "levels equal to the baseline pass"
    receipt L4 L3 4 4
    expect 0 "a level above the baseline passes (and is reported as a rise)"
    grep -q 'RISE a: L3 -> L4' "$d/out" && pass=$((pass + 1)) || {
        fail=$((fail + 1))
        echo "FAIL a rise is named"
    }
    receipt L2 L3 4 4
    expect 1 "a level below the baseline fails"
    printf '{"contracts":[{"stem":"a","proof_level":"L3"}],"totals":{"bindings_implemented":4,"bindings_total":4}}\n' >"$d/r.json"
    expect 1 "a baseline contract gone from the receipt fails"
    printf '{"contracts":[{"stem":"a","proof_level":"L3"},{"stem":"b","proof_level":"L3"},{"stem":"c","proof_level":"L5"}],"totals":{"bindings_implemented":4,"bindings_total":4}}\n' >"$d/r.json"
    expect 1 "a contract missing from the baseline fails"
    receipt L3 L3 3 3
    expect 1 "fewer verified bindings than the baseline fails"
    receipt L3 L3 4 5
    expect 1 "an unverified binding fails"
    printf '{"contracts":[],"totals":{}}\n' >"$d/r.json"
    expect 1 "an empty receipt fails"
    printf '{"levels":{"a":"L3","b":"L3"},"bindings_implemented":4,"lean_grounded":2}\n' >"$d/b.json"
    printf '{"contracts":[{"stem":"a","proof_level":"L3"},{"stem":"b","proof_level":"L3"}],"totals":{"bindings_implemented":4,"bindings_total":4,"lean_grounded":2}}\n' >"$d/r.json"
    expect 0 "as many grounded Lean theorems as the baseline pass"
    printf '{"contracts":[{"stem":"a","proof_level":"L3"},{"stem":"b","proof_level":"L3"}],"totals":{"bindings_implemented":4,"bindings_total":4,"lean_grounded":1}}\n' >"$d/r.json"
    expect 1 "fewer grounded Lean theorems than the baseline fails"
    printf '{"levels":{"a":"L3","b":"L3"},"bindings_implemented":4}\n' >"$d/b.json"
    printf 'not json\n' >"$d/r.json"
    expect 1 "a receipt that is not JSON fails"
    # The Kani phantom ratchet, on a fixture tree.
    mkdir -p "$d/k/src" "$d/k/contracts"
    printf '#[kani::proof]\n#[kani::unwind(3)]\nfn verify_a() {}\n' >"$d/k/src/h.rs"
    printf 'kani_harnesses:\n- id: K1\n  harness: verify_a\n- id: K2\n  harness: verify_gone\n' >"$d/k/contracts/c.yaml"
    kexpect() { # kexpect <0|1> <max> <message>
        local got=0
        printf '{"levels":{"a":"L3"},"bindings_implemented":1,"phantom_kani_harnesses":%s}\n' "$2" >"$d/kb.json"
        kani_ratchet "$d/kb.json" "$d/k" >"$d/out" 2>&1 || got=1
        if [ "$got" = "$1" ]; then
            pass=$((pass + 1))
            printf 'ok   %s\n' "$3"
        else
            fail=$((fail + 1))
            printf 'FAIL %s\n' "$3"
            sed 's/^/     /' "$d/out"
        fi
    }
    kexpect 0 1 "one phantom harness against a baseline of one passes"
    kexpect 1 0 "a phantom harness above the baseline fails"
    printf '{"levels":{"a":"L3"}}\n' >"$d/kb.json"
    if kani_ratchet "$d/kb.json" "$d/k" >"$d/out" 2>&1; then
        fail=$((fail + 1))
        echo "FAIL a baseline without phantom_kani_harnesses fails"
    else
        pass=$((pass + 1))
        echo "ok   a baseline without phantom_kani_harnesses fails"
    fi
    printf 'kani_harnesses:\n- id: K1\n  harness: verify_a\n' >"$d/k/contracts/c.yaml"
    kexpect 0 1 "a harness found after #[kani::proof] and another attribute counts as present"
    grep -q '1 of 1 contract(s) with harnesses have every one in src/; 0 phantom' "$d/out" && pass=$((pass + 1)) || {
        fail=$((fail + 1))
        echo "FAIL the kani summary counts a backed contract"
    }
    rm -f "$d/k/contracts/c.yaml"
    kexpect 1 5 "no contract declaring any harness is a decline, not a pass"
    rm -rf "${d:?}"
    printf 'proof-ratchet: self-test: %d passed, %d failed\n' "$pass" "$fail"
    [ "$fail" -eq 0 ]
}

case "${1:-}" in
    --self-test)
        self_test
        exit $?
        ;;
    --kani-table)
        kani_table .
        exit 0
        ;;
esac
rc=0
ratchet "${1:-contracts/proof-status.json}" "${2:-contracts/proof-baseline.json}" || rc=1
kani_ratchet "${2:-contracts/proof-baseline.json}" . || rc=1
exit "$rc"
