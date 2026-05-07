#!/usr/bin/env bash
# Compile every *.wasm file in this directory to PVM (.jam), reporting status.
#
# Set TRAP_FLOATS=1 to pass --trap-floats and convert f32/f64 ops into runtime
# traps. Useful for triaging which modules use floats vs. which use other
# unsupported features.
#
# This directory ships without WASM binaries — drop your own runtime *.wasm
# files alongside this script before running.
#
# Usage:
#   ./compile.sh              # default mode (floats are a hard error)
#   TRAP_FLOATS=1 ./compile.sh # convert float ops to runtime traps
#   COMPILE_ARGS="--no-inline" ./compile.sh   # extra flags forwarded to wasm-pvm

set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." &>/dev/null && pwd)"

CLI_BIN="${REPO_ROOT}/target/release/wasm-pvm"
if [[ ! -x "${CLI_BIN}" ]]; then
    echo "Building release binary..."
    (cd "${REPO_ROOT}" && cargo build --release -p wasm-pvm-cli) || exit $?
fi

EXTRA_FLAGS=()
if [[ "${TRAP_FLOATS:-0}" == "1" ]]; then
    EXTRA_FLAGS+=("--trap-floats")
fi
if [[ -n "${COMPILE_ARGS:-}" ]]; then
    # shellcheck disable=SC2206
    EXTRA_FLAGS+=(${COMPILE_ARGS})
fi

OUT_DIR="${SCRIPT_DIR}/dist"
mkdir -p "${OUT_DIR}"

shopt -s nullglob
WASM_FILES=("${SCRIPT_DIR}"/*.wasm)
shopt -u nullglob

if [[ ${#WASM_FILES[@]} -eq 0 ]]; then
    echo "No *.wasm files found in ${SCRIPT_DIR}." >&2
    echo "Drop runtime WASM binaries here and re-run." >&2
    exit 1
fi

ok=0
fail=0
declare -a failures=()

for wasm in "${WASM_FILES[@]}"; do
    name="$(basename "${wasm}" .wasm)"
    out="${OUT_DIR}/${name}.jam"
    log="${OUT_DIR}/${name}.log"

    printf '%-40s ' "${name}"
    if "${CLI_BIN}" compile "${wasm}" -o "${out}" "${EXTRA_FLAGS[@]}" >"${log}" 2>&1; then
        size=$(wc -c <"${out}")
        printf 'OK   %s bytes\n' "${size}"
        ok=$((ok + 1))
    else
        # Surface the most informative line from the error: the new diagnostic
        # wraps unsupported-feature errors with function name + byte offset.
        first_err=$(grep -m 1 -E 'Unsupported|Float|in function' "${log}" || true)
        printf 'FAIL %s\n' "${first_err}"
        failures+=("${name}")
        fail=$((fail + 1))
    fi
done

echo
echo "Summary: ${ok} compiled, ${fail} failed"
if [[ ${fail} -gt 0 ]]; then
    echo "Failures:"
    for f in "${failures[@]}"; do
        echo "  ${f}  → see ${OUT_DIR}/${f}.log"
    done
fi
