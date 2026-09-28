#!/usr/bin/env bash
# ci_gate.sh — the `gate` job, the ONE check the org ruleset requires (GH-1).
#
# GitHub enforces only the checks a ruleset names, and "Green Main" names one: `gate`. Any other job
# could be red and a PR would still merge. So `gate` runs last in .github/workflows/ci.yml, with
# `if: always()` and `needs:` on every blocking job, and this script turns it red unless all of them
# succeeded.
#
#   --results            read NEEDS_JSON (`${{ toJSON(needs) }}`); fail unless every needed job's
#                        result is `success` (failure, cancelled and skipped all fail), or if none is
#   --check-workflow [f] fail unless, in the workflow f (default .github/workflows/ci.yml), the `gate`
#                        job has `if: always()` and needs every job that is not advisory, and needs no
#                        advisory job. A job is advisory iff it sets `continue-on-error: true` at job
#                        level. A new job is therefore blocking unless it says it is not.
#   --required [f]       print one line per job `gate` needs: id, then display name (for the README)
#   --advisory [f]       print the same for every advisory job
#   --self-test          run every rule above against fixtures, including planted faults
set -euo pipefail

WORKFLOW=.github/workflows/ci.yml

# jobs_of <workflow> — one TSV row per job: id, name, needs (comma separated), job-level
# continue-on-error, job-level if. Reads the two-space job keys under `jobs:` and their four-space
# fields; `needs:` may be a scalar or a one-line [a, b] list. A matrix suffix such as
# " (${{ matrix.shard }})" is dropped from the name: it is one job, whatever its shards.
jobs_of() {
    awk '
        function flush() { if (id != "") printf "%s\t%s\t%s\t%s\t%s\n", id, name, needs, coe, cond }
        /^jobs:[[:space:]]*$/ { on = 1; next }
        /^[^[:space:]#]/      { if (on) flush(); on = 0; id = ""; next }
        !on { next }
        /^  [A-Za-z0-9_-]+:[[:space:]]*$/ {
            flush(); id = $1; sub(/:$/, "", id); name = id; needs = ""; coe = "false"; cond = ""; next
        }
        /^    name:/ { v = $0; sub(/^    name:[[:space:]]*/, "", v); sub(/[[:space:]]+#.*$/, "", v); sub(/[[:space:]]*\(\$\{\{[^}]*\}\}\)$/, "", v); name = v; next }
        /^    needs:/ {
            v = $0; sub(/^    needs:[[:space:]]*/, "", v); gsub(/[][[:space:]]/, "", v); needs = v; next
        }
        /^    continue-on-error:[[:space:]]*true/ { coe = "true"; next }
        /^    if:/ { v = $0; sub(/^    if:[[:space:]]*/, "", v); cond = v; next }
        END { flush() }' "$1"
}

check_workflow() { # check_workflow <workflow>
    local f=$1 rows gate needs id name coe bad=0
    [ -f "$f" ] || {
        echo "ci-gate: FAIL no workflow at $f"
        return 1
    }
    rows=$(jobs_of "$f")
    gate=$(awk -F'\t' '$1 == "gate"' <<<"$rows")
    [ -n "$gate" ] || {
        echo "ci-gate: FAIL $f has no job with id gate"
        return 1
    }
    IFS=$'\t' read -r _ name needs _ cond <<<"$gate"
    [ "$name" = gate ] || {
        echo "ci-gate: FAIL the gate job is named '$name'; the ruleset requires the check name 'gate'"
        bad=1
    }
    [[ "$cond" == *'always()'* ]] || {
        echo "ci-gate: FAIL the gate job has no 'if: always()', so a failed need skips it, and a skipped check can read as passing"
        bad=1
    }
    [ -n "$needs" ] || {
        echo "ci-gate: FAIL the gate job needs nothing, so it checks nothing"
        bad=1
    }
    while IFS=$'\t' read -r id name _ coe _; do
        [ "$id" = gate ] && continue
        if [ "$coe" = true ]; then
            [[ ",$needs," == *",$id,"* ]] && {
                echo "ci-gate: FAIL advisory job $id ($name) is in the gate's needs"
                bad=1
            }
        elif [[ ",$needs," != *",$id,"* ]]; then
            echo "ci-gate: FAIL job $id ($name) is not advisory and the gate does not need it: it could fail and the PR would still merge"
            bad=1
        fi
    done <<<"$rows"
    for id in ${needs//,/ }; do
        awk -F'\t' -v j="$id" '$1 == j { found = 1 } END { exit !found }' <<<"$rows" || {
            echo "ci-gate: FAIL the gate needs $id, which is not a job"
            bad=1
        }
    done
    [ "$bad" = 0 ] && echo "ci-gate: workflow OK: gate needs $(tr ',' ' ' <<<"$needs")"
    return "$bad"
}

check_results() { # check_results <json>
    local rows bad
    rows=$(jq -r 'to_entries[] | "\(.key)\t\(.value.result)"' <<<"$1" 2>/dev/null) || {
        echo "ci-gate: FAIL NEEDS_JSON is not the JSON of \`needs\`"
        return 1
    }
    [ -n "$rows" ] || {
        echo "ci-gate: FAIL no needed job reported a result: nothing measured is a decline, not a pass"
        return 1
    }
    awk -F'\t' '{ printf "ci-gate: %-8s %s\n", $2, $1 }' <<<"$rows"
    bad=$(awk -F'\t' '$2 != "success"' <<<"$rows" | grep -c . || true)
    if [ "$bad" -gt 0 ]; then
        echo "ci-gate: FAIL $bad needed job(s) did not succeed"
        return 1
    fi
    echo "ci-gate: every one of $(grep -c . <<<"$rows") needed job(s) succeeded"
}

self_test() {
    local d pass=0 fail=0
    d=$(mktemp -d)
    expect() { # expect <0|1> <message> <command...>
        local want=$1 msg=$2 got=0
        shift 2
        "$@" >"$d/out" 2>&1 || got=1
        if [ "$got" = "$want" ]; then
            pass=$((pass + 1))
            printf 'ok   %s\n' "$msg"
        else
            fail=$((fail + 1))
            printf 'FAIL %s\n' "$msg"
            sed 's/^/     /' "$d/out"
        fi
    }
    wf() { # wf <gate needs> <gate if> — a workflow with jobs a, b and advisory c
        printf 'name: CI\non: [push]\njobs:\n  a:\n    name: A\n    runs-on: x\n  b:\n    name: B job\n    needs: a\n    runs-on: x\n  c:\n    name: C (advisory)\n    needs: a\n    continue-on-error: true\n    runs-on: x\n  gate:\n    name: gate\n' >"$d/ci.yml"
        [ -n "$2" ] && printf '    if: %s\n' "$2" >>"$d/ci.yml"
        printf '    needs: [%s]\n    runs-on: x\n    steps:\n      - run: true\n' "$1" >>"$d/ci.yml"
    }
    wf "a, b" "always()"
    expect 0 "a gate that needs every blocking job, with if: always(), passes" check_workflow "$d/ci.yml"
    wf "a" "always()"
    expect 1 "a blocking job the gate does not need fails" check_workflow "$d/ci.yml"
    wf "a, b" ""
    expect 1 "a gate without if: always() fails" check_workflow "$d/ci.yml"
    wf "a, b, c" "always()"
    expect 1 "an advisory job in the gate's needs fails" check_workflow "$d/ci.yml"
    wf "a, b, z" "always()"
    expect 1 "a need that is not a job fails" check_workflow "$d/ci.yml"
    wf "a, b" "always()"
    sed -i 's/^    name: gate$/    name: all-green/' "$d/ci.yml"
    expect 1 "a gate job whose check name is not 'gate' fails" check_workflow "$d/ci.yml"
    expect 1 "a workflow with no gate job fails" check_workflow /dev/null
    expect 0 "every needed job succeeded passes" \
        check_results '{"a":{"result":"success","outputs":{}},"b":{"result":"success","outputs":{}}}'
    expect 1 "one failed job fails the gate" \
        check_results '{"a":{"result":"success"},"kani":{"result":"failure"}}'
    expect 1 "a skipped job (its own need failed) fails the gate" \
        check_results '{"a":{"result":"failure"},"b":{"result":"skipped"}}'
    expect 1 "a cancelled job fails the gate" check_results '{"a":{"result":"cancelled"}}'
    expect 1 "no results is a decline, not a pass" check_results '{}'
    expect 1 "results that are not JSON fail" check_results 'not json'
    wf "a, b" "always()"
    jobs_of "$d/ci.yml" >"$d/rows"
    if grep -qxF "$(printf 'c\tC (advisory)\ta\ttrue\t')" "$d/rows"; then
        pass=$((pass + 1))
        echo "ok   the job parser reads name, needs and continue-on-error"
    else
        fail=$((fail + 1))
        echo "FAIL the job parser reads name, needs and continue-on-error"
        sed 's/^/     /' "$d/rows"
    fi
    sed -i 's/^    name: B job$/    name: B job (${{ matrix.shard }})/' "$d/ci.yml"
    jobs_of "$d/ci.yml" >"$d/rows"
    if grep -q "$(printf '^b\tB job\t')" "$d/rows"; then
        pass=$((pass + 1))
        echo "ok   the job parser drops a matrix suffix from the name"
    else
        fail=$((fail + 1))
        echo "FAIL the job parser drops a matrix suffix from the name"
        sed 's/^/     /' "$d/rows"
    fi
    rm -rf "${d:?}"
    printf 'ci-gate: self-test: %d passed, %d failed\n' "$pass" "$fail"
    [ "$fail" -eq 0 ]
}

case "${1:-}" in
    --results) check_results "${NEEDS_JSON:-}" ;;
    --check-workflow) check_workflow "${2:-$WORKFLOW}" ;;
    --required)
        needs=$(jobs_of "${2:-$WORKFLOW}" | awk -F'\t' '$1 == "gate" { print $3 }')
        [ -n "$needs" ] || exit 1
        for id in ${needs//,/ }; do jobs_of "${2:-$WORKFLOW}" | awk -F'\t' -v j="$id" -v OFS='\t' '$1 == j { print $1, $2 }'; done
        ;;
    --advisory) jobs_of "${2:-$WORKFLOW}" | awk -F'\t' -v OFS='\t' '$4 == "true" { print $1, $2 }' ;;
    --self-test) self_test ;;
    *)
        echo "usage: ci_gate.sh --results | --check-workflow [ci.yml] | --required [ci.yml] | --advisory [ci.yml] | --self-test" >&2
        exit 2
        ;;
esac
