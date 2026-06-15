#!/usr/bin/env bash
# Reproduce the anan-as interpreter core-lowering miscompile (issue #261).
#
# Builds the interpreter from CURRENT source with wasm-pvm, then runs the
# committed triggering inner program (`trigger.jam`) three ways via
# run-checks.ts and prints a verdict. Exit code 1 == bug reproduced,
# 0 == compiled interpreter ran it correctly (bug appears fixed).
#
# Prereqs: cargo, bun, node, and the anan-as submodule built (this script
# builds it if the artifacts are missing).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT"

CLI="$ROOT/target/release/wasm-pvm"
COMPILER_WASM="$ROOT/vendor/anan-as/dist/build/compiler.wasm"
TRIGGER="$HERE/trigger.jam"
INTERP_JAM="${INTERP_JAM:-$HERE/anan-as-compiler.jam}"

echo "==> Building wasm-pvm CLI (release)..."
cargo build --release -p wasm-pvm-cli >/dev/null

echo "==> Ensuring anan-as interpreter is built..."
if [ ! -f "$COMPILER_WASM" ] || [ ! -f "$ROOT/vendor/anan-as/build/release.js" ]; then
  echo "    building anan-as (submodule)..."
  ( cd "$ROOT/vendor/anan-as" && { [ -d node_modules ] || npm ci; } && npm run build )
fi
[ -f "$COMPILER_WASM" ] || { echo "ERROR: $COMPILER_WASM missing after build" >&2; exit 2; }
[ -f "$ROOT/vendor/anan-as/build/release.js" ] || { echo "ERROR: $ROOT/vendor/anan-as/build/release.js missing after build" >&2; exit 2; }

echo "==> Compiling the interpreter to PVM with wasm-pvm..."
"$CLI" compile "$COMPILER_WASM" -o "$INTERP_JAM" \
  --imports "$ROOT/tests/fixtures/imports/anan-as-compiler.imports" \
  --adapter "$ROOT/tests/fixtures/imports/anan-as-compiler.adapter.wat" \
  --max-memory 256 >/dev/null
echo "    -> $INTERP_JAM ($(wc -c < "$INTERP_JAM") bytes)"

echo "==> Running the 3-way check on trigger.jam ..."
echo
bun "$HERE/run-checks.ts" "$INTERP_JAM" "$TRIGGER"
