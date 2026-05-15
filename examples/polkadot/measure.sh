#!/usr/bin/env bash
# Recompile glutton-kusama with the freshly built CLI and report code-size +
# instruction-count metrics. Intended for per-commit verification while
# iterating on optimizations.
set -euo pipefail

THIS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$THIS_DIR/../.." && pwd)"
WASM_PVM="$PROJECT_ROOT/target/release/wasm-pvm"

WASM="$THIS_DIR/wasm/glutton-kusama.wasm"
IMPORTS="$THIS_DIR/imports/glutton-kusama.imports"
JAM="$THIS_DIR/jam/glutton-kusama.jam"

[ -x "$WASM_PVM" ] || { echo "ERROR: build the release binary first"; exit 1; }
[ -f "$WASM" ]      || { echo "ERROR: missing $WASM";                exit 1; }
[ -f "$IMPORTS" ]   || { echo "ERROR: missing $IMPORTS";             exit 1; }

# Recompile (logs to stderr; quiet stdout).
"$WASM_PVM" compile "$WASM" -o "$JAM" --imports "$IMPORTS" --trap-floats \
  >/tmp/glutton-compile.log 2>&1 || {
    echo "COMPILE FAILED:"
    tail -30 /tmp/glutton-compile.log
    exit 1
  }

# Pull the headline numbers out of the analyzer's first ~5 lines.
# Capture the output first so `head` exiting early doesn't SIGPIPE the
# producer (which would trip `set -o pipefail` and abort the script
# before the final JAM-size echo runs).
ANALYZE_OUT="$(python3 "$THIS_DIR/analyze-jam.py" "$JAM")"
printf '%s\n' "$ANALYZE_OUT" | head -5
echo "JAM bytes: $(stat -f%z "$JAM" 2>/dev/null || stat -c%s "$JAM")"
