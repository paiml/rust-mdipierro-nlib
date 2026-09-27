#!/usr/bin/env bash
# lean_gate.sh — the Lean proofs build, and every theorem a contract cites is one of them (GH-5).
#
# `pv proof-status` credits an equation with L4 when its `lean_theorem:` names a file under
# lean/ProvableContracts/Theorems that holds no `sorry`. It does not run Lean. This gate does:
#   1. every `lean_theorem: Theorems.<Domain>.<name>` and every obligation's
#      `module: ProvableContracts.Theorems.<Domain>.<name>` in contracts/*.yaml resolves to
#      lean/ProvableContracts/Theorems/<Domain>/<name>.lean, which declares `theorem <name>`;
#   2. every file under Theorems/ is imported by lean/ProvableContracts.lean, so `lake build`
#      checks it (a file nothing imports is never compiled, and would prove nothing);
#   3. no Lean source holds `sorry`, `admit`, a new `axiom` or `native_decide` (which trusts the
#      compiler instead of the kernel);
#   4. `lake build` succeeds with the toolchain pinned in lean/lean-toolchain and Mathlib pinned in
#      lean/lake-manifest.json. The lakefile makes every warning an error, `sorry` included.
# Mathlib comes from its prebuilt cache (`lake exe cache get`), never from source.
#
# usage: lean_gate.sh [--static [dir] | --self-test]
#   (no argument)  static checks, then fetch the Mathlib cache and `lake build`
#   --static       the static checks 1-3 only, on dir (default .)
#   --self-test    the static checks against planted faults in a fixture tree
set -euo pipefail

FAILS=0
fail() {
    printf 'lean-gate: FAIL %s\n' "$*"
    FAILS=$((FAILS + 1))
}

# static_checks <dir> — checks 1-3 on the tree at dir; prints one FAIL line per fault.
static_checks() {
    local dir=$1 root lib refs ref dom name f n=0 files rel mod bad hits
    lib="$dir/lean/ProvableContracts"
    root="$dir/lean/ProvableContracts.lean"
    [ -f "$root" ] || {
        fail "$root is missing"
        return 0
    }
    refs=$( {
        grep -hoE '^[[:space:]]*lean_theorem:[[:space:]]*Theorems\.[A-Za-z]+\.[A-Za-z0-9_]+' "$dir"/contracts/*.yaml \
            | sed -E 's/.*Theorems\.//' || true
        grep -hoE '^[[:space:]]*module:[[:space:]]*ProvableContracts\.Theorems\.[A-Za-z]+\.[A-Za-z0-9_]+' "$dir"/contracts/*.yaml \
            | sed -E 's/.*ProvableContracts\.Theorems\.//' || true
    } | sort -u)
    [ -n "$refs" ] || fail "no contract cites a Lean theorem (nothing measured is a decline, not a pass)"
    while IFS=. read -r dom name; do
        [ -n "$dom" ] || continue
        n=$((n + 1))
        f="$lib/Theorems/$dom/$name.lean"
        if [ ! -f "$f" ]; then
            fail "a contract cites Theorems.$dom.$name, but ${f#"$dir"/} does not exist"
        elif ! grep -Eq "^theorem $name([[:space:]]|$)" "$f"; then
            fail "${f#"$dir"/} declares no theorem $name"
        fi
    done <<<"$refs"
    files=$(find "$lib/Theorems" -name '*.lean' 2>/dev/null | sort)
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        rel=${f#"$dir"/lean/}
        mod=${rel%.lean}
        mod=${mod//\//.}
        grep -qxF "import $mod" "$root" || fail "$rel is not imported by lean/ProvableContracts.lean, so lake build never checks it"
    done <<<"$files"
    bad='(^|[^A-Za-z0-9_.])(sorry|admit|native_decide)([^A-Za-z0-9_]|$)|^[[:space:]]*(private[[:space:]]+)?axiom[[:space:]]'
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        hits=$(grep -nE "$bad" "$f" | grep -vE '^[0-9]+:[[:space:]]*--' || true)
        [ -z "$hits" ] || fail "an unproved step or a trusted shortcut in ${f#"$dir"/}: ${hits//$'\n'/; }"
    done < <(find "$dir/lean" -name '*.lean' -not -path '*/.lake/*' | sort)
    printf 'lean-gate: %s theorem reference(s) in contracts/, %s theorem file(s) under lean/, each checked\n' "$n" "$(grep -c . <<<"$files" || true)"
}

build() {
    command -v lake >/dev/null || {
        fail "lake is not installed (install elan; lean/lean-toolchain pins the version)"
        return 0
    }
    (
        cd lean
        lake exe cache get
        lake build
    ) || fail "lake build failed"
}

self_test() {
    local d pass=0 fail_n=0 t=ProvableContracts/Theorems/Dom
    d=$(mktemp -d)
    fixture() {
        rm -rf "${d:?}/t"
        mkdir -p "$d/t/contracts" "$d/t/lean/$t"
        printf 'equations:\n  e:\n    lean_theorem: Theorems.Dom.good\n' >"$d/t/contracts/c.yaml"
        printf 'import ProvableContracts.Theorems.Dom.good\n' >"$d/t/lean/ProvableContracts.lean"
        printf -- '-- a comment may say sorry\ntheorem good : 1 = 1 := rfl\n' >"$d/t/lean/$t/good.lean"
    }
    expect() { # expect <0|1> <message>
        local got
        FAILS=0
        static_checks "$d/t" >"$d/out" 2>&1
        got=$((FAILS > 0 ? 1 : 0))
        if [ "$got" = "$1" ]; then
            pass=$((pass + 1))
            printf 'ok   %s\n' "$2"
        else
            fail_n=$((fail_n + 1))
            printf 'FAIL %s\n' "$2"
            sed 's/^/     /' "$d/out"
        fi
    }
    fixture
    expect 0 "a cited, imported, sorry-free theorem passes (a comment may say sorry)"
    fixture
    printf 'theorem good : 1 = 1 := by sorry\n' >"$d/t/lean/$t/good.lean"
    expect 1 "a sorry fails"
    fixture
    printf 'theorem good : 1 = 1 := by admit\n' >"$d/t/lean/$t/good.lean"
    expect 1 "an admit fails"
    fixture
    printf 'axiom cheat : False\ntheorem good : 1 = 1 := rfl\n' >"$d/t/lean/$t/good.lean"
    expect 1 "a new axiom fails"
    fixture
    printf 'theorem good : 1 = 1 := by native_decide\n' >"$d/t/lean/$t/good.lean"
    expect 1 "native_decide fails"
    fixture
    printf 'equations:\n  e:\n    lean_theorem: Theorems.Dom.gone\n' >"$d/t/contracts/c.yaml"
    expect 1 "a lean_theorem naming no file fails"
    fixture
    printf 'lemma good : 1 = 1 := rfl\n' >"$d/t/lean/$t/good.lean"
    expect 1 "a file that declares no theorem of that name fails"
    fixture
    printf 'proof_obligations:\n- lean:\n    module: ProvableContracts.Theorems.Dom.other\n' >>"$d/t/contracts/c.yaml"
    expect 1 "an obligation's module naming no file fails"
    fixture
    printf 'theorem extra : 2 = 2 := rfl\n' >"$d/t/lean/$t/extra.lean"
    expect 1 "a theorem file the root does not import fails"
    fixture
    printf 'no theorems\n' >"$d/t/contracts/c.yaml"
    expect 1 "no contract citing any theorem is a decline, not a pass"
    rm -rf "${d:?}"
    printf 'lean-gate: self-test: %d passed, %d failed\n' "$pass" "$fail_n"
    [ "$fail_n" -eq 0 ]
}

case "${1:-}" in
    --self-test)
        self_test
        exit $?
        ;;
    --static)
        static_checks "${2:-.}"
        ;;
    "")
        static_checks .
        build
        ;;
    *)
        echo "usage: lean_gate.sh [--static [dir] | --self-test]" >&2
        exit 2
        ;;
esac
if [ "$FAILS" -gt 0 ]; then
    printf 'lean-gate: FAIL %d fault(s)\n' "$FAILS"
    exit 1
fi
echo "lean-gate: PASS"
