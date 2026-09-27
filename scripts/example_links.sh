#!/usr/bin/env bash
# example_links.sh — every example has ONE README row whose links, in order, are (GH-1 scope b, c):
#
#   1. the Rust code        examples/<name>.rs
#   2. the Python original  https://github.com/mdipierro/nlib/blob/<ref>/<file>#L<n>
#                           <ref> is the pin in contracts/external-corpora.yaml, and line <n> of <file>
#                           at that ref must be `def <symbol>` or `class <symbol>`, <symbol> being the
#                           link text
#   3. the contract         contracts/example-<stem>-v1.yaml
#      and its shape        contracts/shapes.ttl#L<n>, the line declaring that contract's sh:NodeShape
#   4. its proof status     contracts/proof-status.json#L<n>, the line naming that stem; the link text
#                           is the level recorded there
#   5. its Lean theorem     lean/ProvableContracts/Theorems/<Domain>/<theorem>.lean (GH-5): the file declares
#                           `theorem <theorem>`, holds no `sorry` token, and a contract equation's
#                           `lean_theorem:` names Theorems.<Domain>.<theorem>. An example with no theorem has
#                           no sixth link and a cell `N/A: <reason>` or `not proved yet: <reason>` instead.
#
# Any example without a row, a row without an example, a missing or out-of-order link, or a target
# that does not exist or does not say what the link claims is a failure. A remote file that cannot
# be fetched is a failure too: an unverified link is not a verified one.
#
# usage: example_links.sh [--self-test] [README.md]    (run from the repository root)
# NLIB_ORIGIN_DIR=<dir> holds <file> paths of the pinned upstream; otherwise they are fetched.
set -euo pipefail

FAILS=0
CHECKED=0

fail() {
    printf 'example-links: FAIL %s\n' "$1"
    FAILS=$((FAILS + 1))
}

# stem_of <example> — the contract stem (snake case becomes kebab case).
stem_of() { printf 'example-%s-v1' "${1//_/-}"; }

# origin_file <ref> <path> — prints a local copy of <path> at <ref>, fetching it when needed.
origin_file() {
    local dir="${NLIB_ORIGIN_DIR:-${TMPDIR:-/tmp}/nlib-origin-$1}"
    if [ ! -s "$dir/$2" ]; then
        mkdir -p "$(dirname "$dir/$2")"
        if ! curl -fsSL "https://raw.githubusercontent.com/mdipierro/nlib/$1/$2" -o "$dir/$2.part"; then
            rm -f "$dir/$2.part"
            return 1
        fi
        mv "$dir/$2.part" "$dir/$2"
    fi
    printf '%s\n' "$dir/$2"
}

# line_of <file> <n> — prints line n, or fails when the file has no such line.
line_of() {
    [ -f "$1" ] || return 1
    case "$2" in '' | *[!0-9]*) return 1 ;; esac
    [ "$2" -ge 1 ] && [ "$2" -le "$(wc -l <"$1")" ] || return 1
    sed -n "${2}p" "$1"
}

check_row() { # check_row <example> <row> <ref>
    local ex=$1 row=$2 ref=$3 stem links n text target path anchor line f level dom
    stem=$(stem_of "$ex")
    links=$(printf '%s\n' "$row" | grep -oE '\[[^]]*\]\([^)]*\)' | sed -E 's/^\[([^]]*)\]\(([^)]*)\)$/\1\t\2/')
    n=$(printf '%s\n' "$links" | grep -c . || true)
    if [ "$n" != 5 ] && [ "$n" != 6 ]; then
        fail "$ex: $n link(s), want 5 or 6 (rust, python, contract, shape, proof status, then Lean)"
        return
    fi
    CHECKED=$((CHECKED + n))

    IFS=$'\t' read -r text target < <(sed -n 1p <<<"$links")
    [ "$target" = "examples/$ex.rs" ] || fail "$ex: link 1 is '$target', want examples/$ex.rs"
    [ -f "examples/$ex.rs" ] || fail "$ex: link 1 target examples/$ex.rs does not exist"

    IFS=$'\t' read -r text target < <(sed -n 2p <<<"$links")
    text=${text//\`/}
    if [[ "$target" =~ ^https://github\.com/mdipierro/nlib/blob/([0-9a-f]{40})/([A-Za-z0-9_./-]+)#L([0-9]+)$ ]]; then
        path=${BASH_REMATCH[2]}
        anchor=${BASH_REMATCH[3]}
        if [ "${BASH_REMATCH[1]}" != "$ref" ]; then
            fail "$ex: link 2 is pinned to ${BASH_REMATCH[1]}, want the external-corpora.yaml ref $ref"
        elif ! f=$(origin_file "$ref" "$path"); then
            fail "$ex: link 2 target $path at $ref could not be fetched, so it is unverified"
        elif ! line=$(line_of "$f" "$anchor"); then
            fail "$ex: link 2 anchor #L$anchor is past the end of $path"
        elif ! grep -Eq "^[[:space:]]*(def|class)[[:space:]]+${text}[[:space:](:]" <<<"$line"; then
            fail "$ex: link 2 $path#L$anchor is '$line', not def/class $text"
        fi
    else
        fail "$ex: link 2 '$target' is not a commit-pinned mdipierro/nlib blob URL with a #L anchor"
    fi

    IFS=$'\t' read -r text target < <(sed -n 3p <<<"$links")
    [ "$target" = "contracts/$stem.yaml" ] || fail "$ex: link 3 is '$target', want contracts/$stem.yaml"
    [ -f "contracts/$stem.yaml" ] || fail "$ex: link 3 target contracts/$stem.yaml does not exist"

    IFS=$'\t' read -r text target < <(sed -n 4p <<<"$links")
    if [[ "$target" =~ ^contracts/shapes\.ttl#L([0-9]+)$ ]]; then
        line=$(line_of contracts/shapes.ttl "${BASH_REMATCH[1]}") || line=
        grep -qF "shape/$stem> a sh:NodeShape" <<<"$line" \
            || fail "$ex: link 4 $target does not declare the shape of $stem"
    else
        fail "$ex: link 4 is '$target', want contracts/shapes.ttl#L<n>"
    fi

    IFS=$'\t' read -r text target < <(sed -n 5p <<<"$links")
    if [[ "$target" =~ ^contracts/proof-status\.json#L([0-9]+)$ ]]; then
        line=$(line_of contracts/proof-status.json "${BASH_REMATCH[1]}") || line=
        grep -qF "\"stem\": \"$stem\"" <<<"$line" \
            || fail "$ex: link 5 $target is not the proof-status row of $stem"
        level=$(jq -r --arg s "$stem" '.contracts[] | select(.stem == $s) | .proof_level' contracts/proof-status.json 2>/dev/null) || level=
        [ -n "$level" ] && [ "$text" = "$level" ] \
            || fail "$ex: link 5 shows '$text', but proof-status.json records '${level:-no row}'"
    else
        fail "$ex: link 5 is '$target', want contracts/proof-status.json#L<n>"
    fi

    # Link 6, the Lean theorem (GH-5), or no link and a cell that says why there is none.
    if [ "$n" = 6 ]; then
        IFS=$'\t' read -r text target < <(sed -n 6p <<<"$links")
        text=${text//\`/}
        if [[ "$target" =~ ^lean/ProvableContracts/Theorems/([A-Za-z]+)/([A-Za-z0-9_]+)\.lean$ ]]; then
            dom=${BASH_REMATCH[1]}
            if [ "${BASH_REMATCH[2]}" != "$text" ]; then
                fail "$ex: link 6 shows '$text' but targets $target"
            elif [ ! -f "$target" ]; then
                fail "$ex: link 6 target $target does not exist"
            elif ! grep -Eq "^theorem $text([[:space:]]|$)" "$target"; then
                fail "$ex: link 6 $target declares no theorem $text"
            elif grep -Eq '(^|[^A-Za-z0-9_])sorry([^A-Za-z0-9_]|$)' "$target"; then
                fail "$ex: link 6 $target holds a sorry token, so it proves nothing"
            elif ! grep -Eq "lean_theorem:[[:space:]]*Theorems\.${dom}\.${text}[[:space:]]*$" contracts/*.yaml; then
                fail "$ex: link 6 $text is named by no contract equation's lean_theorem (Theorems.$dom.$text)"
            fi
        else
            fail "$ex: link 6 is '$target', want lean/ProvableContracts/Theorems/<Domain>/<theorem>.lean"
        fi
    elif ! grep -Eq '\| (N/A|not proved yet): [^|]+\|' <<<"$row"; then
        fail "$ex: no Lean link, and no 'N/A: <reason>' or 'not proved yet: <reason>' cell"
    fi
}

check_readme() { # check_readme <README>
    local readme=$1 ref ex rows count linked f
    FAILS=0
    CHECKED=0
    ref=$(sed -n 's/^[[:space:]]*ref:[[:space:]]*\([0-9a-f]\{40\}\).*/\1/p' contracts/external-corpora.yaml | head -1)
    [ -n "$ref" ] || {
        fail "contracts/external-corpora.yaml pins no 40-hex ref"
        return 1
    }
    set -- examples/*.rs
    [ -f "$1" ] || {
        fail "no examples/*.rs: nothing to check is a decline, not a pass"
        return 1
    }
    for f in "$@"; do
        ex=$(basename "$f" .rs)
        rows=$(grep -E '^\|' "$readme" | grep -F "](examples/$ex.rs)" || true)
        count=$(printf '%s' "$rows" | grep -c . || true)
        if [ "$count" != 1 ]; then
            fail "$ex: $count README row(s) link examples/$ex.rs, want exactly 1"
            continue
        fi
        check_row "$ex" "$rows" "$ref"
    done
    # A row for an example that does not exist.
    while IFS= read -r linked; do
        [ -f "$linked" ] || fail "README links $linked, which does not exist"
    done < <(grep -E '^\|' "$readme" | grep -oE '\]\(examples/[^)]*\.rs\)' | sed -E 's/^\]\((.*)\)$/\1/' | sort -u)
    printf 'example-links: %d example(s), %d link(s) checked, %d failed\n' "$#" "$CHECKED" "$FAILS"
    [ "$FAILS" -eq 0 ]
}

self_test() {
    local d pass=0 fail=0 sha=0123456789abcdef0123456789abcdef01234567 other row
    other=$(printf 'f%.0s' {1..40})
    d=$(mktemp -d)
    mkdir -p "$d/examples" "$d/contracts" "$d/origin/src"
    : >"$d/examples/a_b.rs"
    printf 'equations:\n  e:\n    lean_theorem: Theorems.Dom.thm\n' >"$d/contracts/example-a-b-v1.yaml"
    mkdir -p "$d/lean/ProvableContracts/Theorems/Dom"
    printf 'theorem thm : 1 = 1 := rfl\n' >"$d/lean/ProvableContracts/Theorems/Dom/thm.lean"
    printf 'schema: x\ncorpora:\n  - name: nlib\n    ref: %s\n' "$sha" >"$d/contracts/external-corpora.yaml"
    printf '@prefix sh: <x> .\n<https://ont.paiml.dev/v1alpha1/shape/example-a-b-v1> a sh:NodeShape ;\n' >"$d/contracts/shapes.ttl"
    printf '{\n  "contracts": [\n    {\n      "stem": "example-a-b-v1",\n      "proof_level": "L3"\n    }\n  ]\n}\n' >"$d/contracts/proof-status.json"
    printf 'import math\ndef foo(x):\n    return x\n' >"$d/origin/src/nlib.py"
    local py="[\`foo\`](https://github.com/mdipierro/nlib/blob/$sha/src/nlib.py#L2)"
    row="| [a_b](examples/a_b.rs) | $py | [example-a-b-v1](contracts/example-a-b-v1.yaml) · [shape](contracts/shapes.ttl#L2) | [L3](contracts/proof-status.json#L4) | not proved yet: fixture |"
    local lean="[\`thm\`](lean/ProvableContracts/Theorems/Dom/thm.lean)"
    local lrow=${row/not proved yet: fixture/$lean}
    expect() { # expect <0|1> <message> <readme text>
        local got=0
        printf '%s\n' "$3" >"$d/README.md"
        (cd "$d" && NLIB_ORIGIN_DIR="$d/origin" check_readme README.md >"$d/out" 2>&1) || got=1
        if [ "$got" = "$1" ]; then
            pass=$((pass + 1))
            printf 'ok   %s\n' "$2"
        else
            fail=$((fail + 1))
            printf 'FAIL %s\n' "$2"
            sed 's/^/     /' "$d/out"
        fi
    }
    expect 0 "a row with all five links, each target verified, passes" "$row"
    expect 1 "a row with neither a Lean link nor a reason fails" "${row/ not proved yet: fixture |/}"
    expect 0 "a row whose sixth link is a cited, sorry-free theorem passes" "$lrow"
    expect 1 "a Lean link to a theorem file that does not exist fails" "${lrow//thm/nope}"
    expect 1 "a Lean link whose text names another theorem fails" "${lrow/\`thm\`/\`other\`}"
    expect 1 "an example with no row fails" "| nothing |"
    expect 1 "two rows for one example fail" "$row"$'\n'"$row"
    expect 1 "a row missing the Python link fails" "${row/"$py"/}"
    expect 1 "the Rust and contract links swapped (out of order) fail" \
        "| [example-a-b-v1](contracts/example-a-b-v1.yaml) | $py | [a_b](examples/a_b.rs) · [shape](contracts/shapes.ttl#L2) | [L3](contracts/proof-status.json#L4) |"
    expect 1 "a Python anchor on the wrong line fails" "${row/nlib.py#L2/nlib.py#L1}"
    expect 1 "a Python anchor past the end of the file fails" "${row/nlib.py#L2/nlib.py#L99}"
    expect 1 "a Python link to a symbol the line does not define fails" "${row/\`foo\`/\`bar\`}"
    expect 1 "a Python link not pinned to the external-corpora ref fails" "${row/$sha/$other}"
    expect 1 "a Python link to master, not a commit, fails" "${row/blob\/$sha/blob\/master}"
    expect 1 "a Python file that cannot be fetched fails" "${row/src\/nlib.py/src\/missing.py}"
    expect 1 "a contract link whose target does not exist fails" "${row//example-a-b-v1.yaml/example-a-c-v1.yaml}"
    expect 1 "a shape anchor off the shape declaration fails" "${row/shapes.ttl#L2/shapes.ttl#L1}"
    expect 1 "a proof-status anchor off the stem row fails" "${row/proof-status.json#L4/proof-status.json#L5}"
    expect 1 "a proof level that differs from proof-status.json fails" "${row/\[L3\]/[L4]}"
    expect 1 "a row for an example that does not exist fails" "$row"$'\n'"${row//a_b/z_z}"
    rm -f "$d/examples/a_b.rs"
    : >"$d/examples/other.rs"
    expect 1 "a row whose Rust file was deleted fails" "$row"
    rm -rf "${d:?}"
    printf 'example-links: self-test: %d passed, %d failed\n' "$pass" "$fail"
    [ "$fail" -eq 0 ]
}

if [ "${1:-}" = --self-test ]; then
    self_test
    exit $?
fi
check_readme "${1:-README.md}"
