#!/usr/bin/env bash
# lint-provenance.sh — ONT-001 v4.17 R-10: a claim with a number carries its mark.
# usage: lint-provenance.sh [--self-test] <file>...
# exit 0 clean · 1 unmarked claims (or a file with nothing examined) · 2 usage
#
# A claim is a Markdown list or table row, or a YAML `key: value` line, that
# holds a digit. Its mark is [V|C|A|U|X ...] on the same line: V measured (with
# the date), C cited, A assumed, U unverified, X refuted. Identifier keys (a
# schema URI, a git ref, a name, the command that counted) are not claims.
#
# A file in which NO claim was examined is a failure, not a pass: a guard that
# cannot see the form it is pointed at reports "0 unmarked" over a file it never
# read (the defect aprender's first version of this script had).
set -euo pipefail

MARKS='\[(V|C|A|U|X)([] ][^]]*)?\]'
EXEMPT='^(schema|ref|repo|name|mark|counted_by|item_type|id|path)$'
CLAIM_FORMS='^[[:space:]]*(-|\|)[[:space:]]*[^[:space:]]|^[[:space:]]*[A-Za-z_][A-Za-z0-9_]*[[:space:]]*:[[:space:]]*[^[:space:]]'
SCANNED=0

lint_file() {
    local f="$1" n=0 scanned=0 line key
    [ -r "$f" ] || {
        printf 'lint-provenance: %s is not readable\n' "$f" >&2
        return 2
    }
    while IFS= read -r line; do
        case "$line" in \#* | '') continue ;; esac
        key=$(printf '%s' "$line" | sed -n 's/^[[:space:]]*\([A-Za-z_][A-Za-z0-9_]*\)[[:space:]]*:.*/\1/p')
        if [ -n "$key" ] && printf '%s' "$key" | grep -Eq "$EXEMPT"; then continue; fi
        printf '%s' "$line" | grep -q '[0-9]' || continue
        # A Markdown table's delimiter row (|---|---:|) holds no claim.
        printf '%s' "$line" | grep -Eq '^[[:space:]]*\|[-:| ]+\|[[:space:]]*$' && continue
        scanned=$((scanned + 1))
        printf '%s' "$line" | grep -Eq "$MARKS" && continue
        printf 'unmarked: %s: %s\n' "$f" "$(printf '%s' "$line" | cut -c1-100)"
        n=$((n + 1))
    done < <(grep -E "$CLAIM_FORMS" "$f" || true)
    printf 'lint-provenance: %s: %d numeric claim(s) examined, %d unmarked\n' "$f" "$scanned" "$n"
    SCANNED=$scanned
    [ "$scanned" -gt 0 ] || {
        printf 'lint-provenance: %s: 0 claims examined; zero is a decline, not a pass\n' "$f" >&2
        return 1
    }
    [ "$n" -eq 0 ]
}

self_test() {
    local t rc=0 pass=0 fail=0
    t=$(mktemp -d)
    printf -- '- the corpus holds 22 contracts [V 2026-09-27]\n' >"$t/green.md"
    printf -- '- the corpus holds 22 contracts\n' >"$t/red.md"
    printf -- '| sort | 7 | PASS [V 2026-09-27] |\n|---|---:|---|\n' >"$t/green-table.md"
    printf -- '| sort | 7 | PASS |\n' >"$t/red-table.md"
    printf -- 'n_files: 19  # [V 2026-09-27]\n' >"$t/green.yaml"
    printf -- 'n_files: 19\n' >"$t/red.yaml"
    printf -- 'schema: ont.paiml.dev/external-corpora/v1alpha1\nref: 5db3a42f\nn_files: 1 [C spec]\n' >"$t/exempt.yaml"
    printf -- 'plain prose with 3 numbers is not a claim form\n' >"$t/empty.md"
    printf -- '- see [V2 of the api] for 3 more\n' >"$t/fake-mark.md"
    expect() { # expect <0|1> <fixture> <message>
        local got=0
        lint_file "$t/$2" >/dev/null 2>&1 || got=1
        if [ "$got" = "$1" ]; then pass=$((pass + 1)); else
            printf 'self-test FAIL: %s\n' "$3"
            rc=1
            fail=$((fail + 1))
        fi
    }
    expect 0 green.md 'a marked Markdown list claim passes'
    expect 1 red.md 'an unmarked Markdown list claim fails'
    expect 0 green-table.md 'a marked table row passes and the delimiter row is not a claim'
    expect 1 red-table.md 'an unmarked table row fails'
    expect 0 green.yaml 'a marked YAML claim passes'
    expect 1 red.yaml 'an unmarked YAML claim fails'
    expect 0 exempt.yaml 'schema and ref are identifiers, not claims'
    expect 1 empty.md 'a file with no claim form examined is a decline, not a pass'
    expect 1 fake-mark.md 'a bracket that is not a mark does not mark a claim'
    lint_file "$t/red.yaml" >/dev/null 2>&1 || true
    if [ "$SCANNED" -ge 1 ]; then pass=$((pass + 1)); else
        printf 'self-test FAIL: the YAML fixture examined %s claims\n' "$SCANNED"
        rc=1
        fail=$((fail + 1))
    fi
    rm -rf "${t:?}"
    printf 'lint-provenance: self-test: %d passed, %d failed\n' "$pass" "$fail"
    return "$rc"
}

[ $# -gt 0 ] || {
    printf 'usage: lint-provenance.sh [--self-test] <file>...\n' >&2
    exit 2
}
if [ "$1" = --self-test ]; then
    self_test
    exit $?
fi
rc=0
for f in "$@"; do lint_file "$f" || rc=1; done
exit "$rc"
