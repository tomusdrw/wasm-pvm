#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI_BIN="$ROOT_DIR/target/release/wasm-pvm"
COMPILER_WASM="$ROOT_DIR/vendor/anan-as/dist/build/compiler.wasm"
COMPILER_JAM="$ROOT_DIR/tests/build/jam/anan-as-compiler.jam"
COMPILER_IMPORTS="$ROOT_DIR/tests/fixtures/imports/anan-as-compiler.imports"
COMPILER_ADAPTER="$ROOT_DIR/tests/fixtures/imports/anan-as-compiler.adapter.wat"

if [[ ! -x "$CLI_BIN" ]]; then
  echo "Missing $CLI_BIN; build with: cargo build -p wasm-pvm-cli --release" >&2
  exit 1
fi
if [[ ! -f "$COMPILER_WASM" ]]; then
  echo "Missing $COMPILER_WASM" >&2
  exit 1
fi

# Recompile the anan-as compiler JAM with optional optimization flags.
"$CLI_BIN" compile "$COMPILER_WASM" -o "$COMPILER_JAM" --imports "$COMPILER_IMPORTS" --adapter "$COMPILER_ADAPTER" "$@"

cd "$ROOT_DIR/tests"

# Run only the two failing cases from pvm-in-pvm suite.
PATTERN='AS: varU32 decode'
bun test layer5/pvm-in-pvm.test.ts --test-name-pattern "$PATTERN"
