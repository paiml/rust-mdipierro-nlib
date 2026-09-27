#!/usr/bin/env bash
# contracts_gate.sh — the provable-contract gate CI runs on every PR (ONT-001 v4.17 row ONT-G; GH-1).
#
# Ported from paiml/aprender scripts/contracts_gate.sh (aprender#4475). Its lesson: a gate whose first step
# ends the recipe never runs the rest, so census, `extract --check`, README sync and provenance were never
# checked there. Here every step has its own status, EVERY step runs (one red may not hide another), the exit
# is non-zero if any step failed, and the summary says how many steps RAN.
#
# Steps:
#   lint        pv lint with --binding/--crate-dir, so every armed gate (reverse-coverage too) is computed
#               then scripts/proof_ratchet.sh: no proof level below contracts/proof-baseline.json, no fewer
#               verified bindings, no more phantom Kani harnesses
#   shapes      pv lint --gate shapes, FAIL-CLOSED: only Pass with shapes_n > 0, focus_nodes_n > 0, pv's
#               planted control fired and every extractor's control fired is a pass. rc 2 is a DECLINE,
#               reported with pv's reason, never a pass.
#   regen       regenerate contracts/census.json, contracts.nt, shapes.ttl and the proof receipt
#               contracts/proof-status.json (pv proof-status --verify-bindings, timestamp removed)
#   readme      scripts/readme_sync.sh --check (every README table is generated), then
#               scripts/example_links.sh: each example has one row, its five links in order, each target real
#   provenance  scripts/lint-provenance.sh --self-test, then over the files that carry numeric claims
#   diff        git says whether a generated file moved: a contract changed without regenerating is RED
#
# Usage:  bash scripts/contracts_gate.sh              # every step
#         bash scripts/contracts_gate.sh shapes       # one step
#         bash scripts/contracts_gate.sh --self-test  # case table against a stub pv, plus a mutant
# CONTRACTS_GATE_PV=<path> replaces pv (the self-test uses it); otherwise `pv` on PATH.

set -u

STEPS=(lint shapes regen readme provenance diff)
GENERATED=(contracts/census.json contracts/contracts.nt contracts/shapes.ttl contracts/proof-status.json)
PROVENANCE_FILES=(contracts/external-corpora.yaml docs/ontology-conformance.md)

step_lint() {
    local log rc
    log=$(mktemp) || return 1
    "$PV" lint contracts/ --binding contracts/binding.yaml --crate-dir . >"$log" 2>&1
    rc=$?
    tail -5 "$log"
    rm -f "$log"
    bash scripts/proof_ratchet.sh --self-test >/dev/null || {
        echo "FAIL: proof_ratchet.sh --self-test"
        rc=1
    }
    bash scripts/proof_ratchet.sh || rc=1
    return "$rc"
}

# shapes_verdict <rc> <json-file> <stderr-file> — prints ONE verdict line, returns 0 only for a real PASS.
shapes_verdict() {
    local rc=$1 json=$2 err=$3 why
    case "$rc" in
        0) ;;
        1)
            echo "SHAPES FAIL rc=1 (reject: measured and failed)"
            return 1
            ;;
        2)
            why=$(grep -h -m1 -oE 'decline[^"]*' "$err" "$json" 2>/dev/null | head -1)
            echo "SHAPES DECLINE rc=2 (${why:-pv declined without naming a reason}): nothing was measured; a decline is not a pass"
            return 2
            ;;
        *)
            echo "SHAPES ERROR rc=$rc (pv neither passed, rejected nor declined)"
            return 3
            ;;
    esac
    jq -e . "$json" >/dev/null 2>&1 || {
        echo "SHAPES ERROR rc=0 but no JSON verdict"
        return 3
    }
    why=$(jq -r -f /dev/stdin "$json" <<'JQ'
        [ (if .verdict != "Pass" then "verdict=\(.verdict|tostring)" else empty end),
          (if ((.extra.shapes_n // .shapes_n // 0) <= 0) then "shapes_n=0" else empty end),         # probe:shapes_n
          (if ((.extra.focus_nodes_n // .focus_nodes_n // 0) <= 0) then "focus_nodes_n=0" else empty end),
          (if ((.extra.pc_shape // .pc_shape) != "fired") then "pc_shape=\((.extra.pc_shape // .pc_shape)|tostring)" else empty end),
          ((.extra.pc_extract // .pc_extract // {}) | to_entries[] | select(.value != "fired") | "pc_extract.\(.key)=\(.value)")
        ] | join(" ")
JQ
)
    if [ -n "$why" ]; then
        echo "SHAPES FAIL rc=0 but not a pass: $why"
        return 1
    fi
    echo "SHAPES PASS $(jq -r '"shapes_n=\(.extra.shapes_n // .shapes_n) focus_nodes_n=\(.extra.focus_nodes_n // .focus_nodes_n)"' "$json")"
}

step_shapes() {
    local json err rc
    json=$(mktemp) && err=$(mktemp) || return 3
    "$PV" lint contracts/ --gate shapes >"$json" 2>"$err"
    rc=$?
    shapes_verdict "$rc" "$json" "$err"
    rc=$?
    rm -f "$json" "$err"
    return "$rc"
}

# Regenerate every generated file from the committed contracts; `diff` then asks GIT whether they moved.
step_regen() {
    local raw
    "$PV" census contracts --format json >contracts/census.json || {
        echo "FAIL: pv census exited non-zero"
        return 1
    }
    "$PV" extract contracts >/dev/null || {
        echo "FAIL: pv extract contracts exited non-zero"
        return 1
    }
    # The proof receipt, minus its wall-clock timestamp so the same contracts give the same bytes.
    raw=$(mktemp) || return 1
    if ! "$PV" proof-status contracts --binding contracts/binding.yaml --verify-bindings . --format json >"$raw"; then
        echo "FAIL: pv proof-status exited non-zero"
        rm -f "$raw"
        return 1
    fi
    jq 'del(.timestamp)' "$raw" >contracts/proof-status.json || {
        echo "FAIL: pv proof-status printed no JSON"
        rm -f "$raw"
        return 1
    }
    rm -f "$raw"
}

step_readme() {
    local rc=0
    PV="$PV" bash scripts/readme_sync.sh --check || rc=1
    bash scripts/example_links.sh --self-test >/dev/null || {
        echo "FAIL: example_links.sh --self-test"
        rc=1
    }
    bash scripts/example_links.sh README.md || rc=1
    return "$rc"
}

step_provenance() {
    bash scripts/lint-provenance.sh --self-test \
        && bash scripts/lint-provenance.sh "${PROVENANCE_FILES[@]}"
}

step_diff() {
    local f
    for f in "${GENERATED[@]}"; do
        git ls-files --error-unmatch "$f" >/dev/null 2>&1 || {
            echo "FAIL: $f is not tracked, so diffing it proves nothing"
            return 1
        }
    done
    git diff --stat --exit-code -- "${GENERATED[@]}" || {
        echo "FAIL: a generated file differs from what the committed contracts produce: a contract changed without regenerating. Run \`bash scripts/contracts_gate.sh regen\` and commit ${GENERATED[*]}"
        return 1
    }
}

run_gate() {
    local s ran=0 failed=()
    for s in "$@"; do
        echo "== contracts gate: $s =="
        ran=$((ran + 1))
        "step_$s" || failed+=("$s")
    done
    echo "contracts gate: $ran of $# step(s) RAN, ${#failed[@]} FAILED${failed[*]:+: ${failed[*]}}"
    [ "${#failed[@]}" -eq 0 ]
}

# ---------------------------------------------------------------------------------------------------------
self_test() {
    local d stub rc out pass=0 fail=0 mutant
    d=$(mktemp -d) || return 2
    stub="$d/pv"
    # The stub pv: census counts contracts/*.yaml, extract writes that count into both graph files, so a
    # contract added without regenerating really does move all three. STUB_FAIL=<subcommand> makes one fail;
    # STUB_SHAPES picks the --gate shapes answer.
    cat >"$stub" <<'STUB'
#!/usr/bin/env bash
n=$(find contracts -maxdepth 1 -name '*.yaml' | wc -l | tr -d ' ')
key=$1
[ "${3:-}" = --gate ] && key=shapes
if [ "${STUB_FAIL:-}" = "$key" ]; then
    echo "stub: $1 fails"
    exit 1
fi
case "$1" in
  census)  echo "{\"n_files\": $n}" ;;
  extract) echo "nt $n" >contracts/contracts.nt; echo "ttl $n" >contracts/shapes.ttl ;;
  proof-status) echo "{\"timestamp\": \"pid-$$\", \"n\": $n}" ;;
  lint)
    [ "${3:-}" = --gate ] || exit 0
    ok='"verdict":"Pass","shapes_n":11,"focus_nodes_n":11,"pc_shape":"fired","pc_extract":{"json":"fired","code":"fired"}'
    case "${STUB_SHAPES:-pass}" in
      pass)      echo "{$ok}" ;;
      fail)      echo '{"verdict":"Fail"}'; exit 1 ;;
      noshapes)  echo 'decline: Unknown{NoShapes}' >&2; exit 2 ;;
      nofocus)   echo 'decline: Unknown{NoFocus}' >&2; exit 2 ;;
      zero)      echo "{${ok/\"shapes_n\":11/\"shapes_n\":0}}" ;;
      nofocus0)  echo "{${ok/\"focus_nodes_n\":11/\"focus_nodes_n\":0}}" ;;
      unknown)   echo "{${ok/\"Pass\"/\"Unknown\"}}" ;;
      silent)    echo "{${ok/\"pc_shape\":\"fired\"/\"pc_shape\":\"silent\"}}" ;;
      extsilent) echo "{${ok/\"code\":\"fired\"/\"code\":\"silent\"}}" ;;
      panic)     echo 'thread main panicked' >&2; exit 101 ;;
      nojson)    echo 'not json' ;;
    esac ;;
esac
exit 0
STUB
    chmod +x "$stub"
    (
        cd "$d" && git init -q . && mkdir -p contracts scripts \
            && echo 'id: a' >contracts/a.yaml \
            && echo '{"n_files": 1}' >contracts/census.json \
            && echo 'nt 1' >contracts/contracts.nt && echo 'ttl 1' >contracts/shapes.ttl \
            && printf '{\n  "n": 1\n}\n' >contracts/proof-status.json \
            && printf '#!/usr/bin/env bash\n[ "${STUB_FAIL:-}" = "$(basename "$0")" ] && exit 1\nexit 0\n' >scripts/readme_sync.sh \
            && cp scripts/readme_sync.sh scripts/lint-provenance.sh && cp scripts/readme_sync.sh scripts/example_links.sh \
            && cp scripts/readme_sync.sh scripts/proof_ratchet.sh \
            && git add contracts scripts \
            && git -c core.hooksPath=/dev/null -c user.email=t@t -c user.name=t commit -qm fixture
    ) || {
        echo "self-test: fixture setup failed"
        return 2
    }
    row() {
        if [ "$2" -eq 0 ]; then
            pass=$((pass + 1))
            echo "ok   $1"
        else
            fail=$((fail + 1))
            echo "FAIL $1"
        fi
    }
    # run <script> <STUB_FAIL> <STUB_SHAPES> [step...]
    run() {
        local script=$1 sf=$2 ss=$3
        shift 3
        (cd "$d" && git checkout -q -- . && CONTRACTS_GATE_PV="$stub" STUB_FAIL="$sf" STUB_SHAPES="$ss" bash "$script" "$@" 2>&1)
    }

    out=$(run "$SELF" "" pass)
    rc=$?
    [ "$rc" = 0 ] && grep -q '6 of 6 step(s) RAN, 0 FAILED' <<<"$out" && grep -q '^SHAPES PASS shapes_n=11' <<<"$out"
    row "all green: exit 0, all 6 steps RAN, shapes PASS" $?

    out=$(run "$SELF" lint pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q '6 of 6 step(s) RAN, 1 FAILED: lint' <<<"$out"
    row "a red pv lint is non-zero AND every later step still runs (rc=$rc)" $?

    out=$(run "$SELF" extract pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q 'FAILED: regen' <<<"$out"
    row "a failing pv extract is non-zero and named (rc=$rc)" $?

    (cd "$d" && echo 'id: b' >contracts/b.yaml && git add contracts/b.yaml \
        && git -c core.hooksPath=/dev/null -c user.email=t@t -c user.name=t commit -qm 'add a contract, do not regenerate')
    out=$(run "$SELF" "" pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q 'FAILED: diff' <<<"$out" && grep -q 'census.json' <<<"$out" \
        && grep -q 'contracts.nt' <<<"$out" && grep -q 'shapes.ttl' <<<"$out" && grep -q proof-status.json <<<"$out"
    row "a contract added WITHOUT regenerating is RED on diff, naming census.json, contracts.nt, shapes.ttl and proof-status.json (rc=$rc)" $?
    (cd "$d" && git checkout -q -- . && CONTRACTS_GATE_PV="$stub" bash "$SELF" regen >/dev/null 2>&1 \
        && git add contracts && git -c core.hooksPath=/dev/null -c user.email=t@t -c user.name=t commit -qm regenerate)
    out=$(run "$SELF" "" pass)
    rc=$?
    [ "$rc" = 0 ]
    row "the same tree after \`contracts_gate.sh regen\` + commit is green again (rc=$rc)" $?


    # GH-1 scope c and b: the proof ratchet, the proof receipt and the example links are each load-bearing.
    out=$(run "$SELF" proof_ratchet.sh pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q '6 of 6 step(s) RAN, 1 FAILED: lint' <<<"$out"
    row "a proof level below the baseline (red proof_ratchet.sh) is RED on lint, every step still RAN (rc=$rc)" $?

    out=$(run "$SELF" proof-status pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q 'FAIL: pv proof-status exited non-zero' <<<"$out" && grep -q 'FAILED: regen' <<<"$out"
    row "a failing pv proof-status is RED on regen (rc=$rc)" $?

    out=$(run "$SELF" example_links.sh pass)
    rc=$?
    [ "$rc" != 0 ] && grep -q '6 of 6 step(s) RAN, 1 FAILED: readme' <<<"$out"
    row "an example missing a link or a target (red example_links.sh) is RED on readme (rc=$rc)" $?
    local c want
    for c in fail:1:FAIL noshapes:2:'DECLINE.*NoShapes' nofocus:2:'DECLINE.*NoFocus' zero:1:'shapes_n=0' \
        nofocus0:1:'focus_nodes_n=0' unknown:1:'verdict=Unknown' silent:1:'pc_shape=silent' \
        extsilent:1:'pc_extract.code=silent' panic:3:'ERROR rc=101' nojson:3:'no JSON verdict'; do
        want=${c#*:}
        out=$(run "$SELF" "" "${c%%:*}" shapes)
        rc=$?
        [ "$rc" != 0 ] && grep -qE "^SHAPES .*${want#*:}" <<<"$out" && ! grep -q 'SHAPES PASS' <<<"$out"
        row "shapes fails closed on ${c%%:*}: non-zero, named ${want#*:}, never PASS (rc=$rc)" $?
    done

    # MUTANT: the same script with the shapes_n probe deleted must go GREEN on shapes_n=0, proving the row
    # above is held up by that probe and nothing else. A guard never seen failing is not evidence.
    mutant="$d/contracts_gate.mutant.sh"
    grep -v "probe:""shapes_n" "$SELF" >"$mutant"
    if [ "$(grep -c "probe:""shapes_n" "$SELF")" -ne 1 ]; then
        row "mutant: exactly one shapes_n probe line to delete" 1
    else
        out=$(run "$mutant" "" zero shapes)
        rc=$?
        [ "$rc" = 0 ] && grep -q '^SHAPES PASS shapes_n=0' <<<"$out"
        row "mutant without the shapes_n probe PASSES shapes_n=0: the probe is load-bearing (rc=$rc)" $?
    fi

    rm -rf "${d:?}"
    echo "contracts_gate self-test: $pass passed, $fail failed"
    [ "$fail" -eq 0 ] && [ "$pass" -eq 19 ]
}

SELF=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")
if [ "${1:-}" = --self-test ]; then
    self_test
    exit $?
fi
command -v jq >/dev/null || {
    echo "contracts gate: jq is required to read pv's verdict"
    exit 3
}
PV=${CONTRACTS_GATE_PV:-pv}
command -v "$PV" >/dev/null || {
    echo "contracts gate: pv not found ($PV); a gate that cannot run is NOT RUN, not a pass"
    exit 3
}
if [ $# -gt 0 ]; then
    for s in "$@"; do
        declare -F "step_$s" >/dev/null || {
            echo "contracts gate: no step '$s' (steps: ${STEPS[*]})"
            exit 3
        }
    done
    run_gate "$@"
else
    run_gate "${STEPS[@]}"
fi
