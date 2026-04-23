#!/usr/bin/env bash
# Determinism check: compile the same input N times in separate processes and
# assert byte-identical output. Catches non-determinism that a single-process
# cargo test cannot (e.g. HashMap hasher randomness is seeded per process, so
# iterating twice within one process shares a seed).
#
# Usage:
#   ./tests/utils/check-determinism.sh [N]
# N defaults to 10.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CLI="$PROJECT_ROOT/target/release/wasm-pvm"
FIXTURES="$PROJECT_ROOT/tests/fixtures/wat"
RUNS="${1:-10}"
if ! [[ "$RUNS" =~ ^[0-9]+$ ]] || [ "$RUNS" -lt 2 ]; then
    echo "Usage: ./tests/utils/check-determinism.sh [N>=2]" >&2
    exit 2
fi

# Always rebuild: cargo no-ops when the tree is up-to-date, but this guards
# against the footgun of editing a source file and running the script before
# remembering to rebuild — which would happily validate determinism of a stale
# binary.
echo "Building release CLI..."
(cd "$PROJECT_ROOT" && cargo build --release --quiet)

# Diverse set: short fixtures, loops, phi-heavy, call-heavy, memory-heavy,
# crypto. Exercises value-slot ordering across LLVM value arenas, loop
# extension, predecessor intersection, regalloc eviction.
FIXTURES_TO_CHECK=(
    add
    bit-ops
    blake2b
    sha512
    fibonacci
    factorial
    gcd
    is-prime
    phi-swap
    phi-dependent
    nested-loop-test
    regalloc-loop-with-call
    regalloc-nested-loops
    call-indirect
    memory-copy-overlap
    memory-copy-word
    recursive
)

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

FAIL=0
MISSING=0
CHECKED=0
for name in "${FIXTURES_TO_CHECK[@]}"; do
    fixture="$FIXTURES/$name.jam.wat"
    if [ ! -f "$fixture" ]; then
        printf "  MISSING  %-32s  (fixture %s not found)\n" "$name" "$fixture"
        MISSING=$((MISSING + 1))
        continue
    fi
    CHECKED=$((CHECKED + 1))
    for i in $(seq 1 "$RUNS"); do
        "$CLI" compile "$fixture" -o "$TMPDIR/${name}_${i}.jam" >/dev/null 2>&1
    done
    unique=$(shasum -a 256 "$TMPDIR/${name}_"*.jam | awk '{print $1}' | sort -u | wc -l | tr -d ' ')
    if [ "$unique" = "1" ]; then
        printf "  PASS     %-32s  %d/%d identical\n" "$name" "$RUNS" "$RUNS"
    else
        printf "  FAIL     %-32s  %s distinct outputs across %d runs\n" "$name" "$unique" "$RUNS"
        FAIL=$((FAIL + 1))
    fi
    rm -f "$TMPDIR/${name}_"*.jam
done

echo
if [ "$FAIL" -eq 0 ] && [ "$MISSING" -eq 0 ]; then
    echo "Determinism check: OK ($CHECKED fixtures × $RUNS runs each)"
else
    echo "Determinism check: FAILED ($FAIL nondeterministic, $MISSING missing)"
    exit 1
fi
