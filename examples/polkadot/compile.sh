#!/usr/bin/env bash
# Compile Polkadot fellowship runtimes to PVM and write README.md with results.
#
# Pipeline per runtime:
#   1. Download .compact.compressed.wasm from the GitHub release.
#   2. Strip 8-byte Substrate magic header (52 BC 53 76 46 DB 8E 05).
#   3. zstd-decompress the remainder into a plain WASM module.
#   4. Verify the result starts with \0asm\01\00\00\00 (WebAssembly magic).
#   5. Auto-generate a trap-all import map covering every (env "...") import.
#   6. Invoke `wasm-pvm compile`, capture stdout/stderr to logs/.
#   7. Record outcome (jam size on success, first error line on failure).
#
# Output: examples/polkadot/README.md (overwritten each run).
#
# Prerequisites: bash, curl, zstd, xxd, awk, wasm-tools
#                (`cargo install wasm-tools` if missing — needed to
#                 generate the trap-all import map), and cargo (release
#                 build of wasm-pvm-cli is invoked automatically).

set -euo pipefail

THIS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$THIS_DIR/../.." && pwd)"

RELEASE_TAG="${RELEASE_TAG:-v2.2.2}"
RELEASE_URL_BASE="https://github.com/polkadot-fellows/runtimes/releases/download/${RELEASE_TAG}"

# The asset list (`RUNTIME_FILES` below) is currently pinned to the `v2002002`
# basenames shipped with `v2.2.2`. Reusing those names against any other tag
# either 404s on download or — worse — silently reuses a previous tag's cached
# downloads while the regenerated README links the new release. Until the
# asset list is derived from `RELEASE_TAG` (e.g. by querying the GitHub
# release API at startup), fail fast on anything else.
SUPPORTED_RELEASE_TAG="v2.2.2"
if [ "$RELEASE_TAG" != "$SUPPORTED_RELEASE_TAG" ]; then
  echo "Error: RELEASE_TAG='$RELEASE_TAG' is not yet supported." >&2
  echo "       This script is currently pinned to ${SUPPORTED_RELEASE_TAG}'s" >&2
  echo "       asset basenames. Drop the override or use" >&2
  echo "       RELEASE_TAG=${SUPPORTED_RELEASE_TAG}." >&2
  exit 1
fi

# Substrate "compact compressed" magic (8 bytes prepended before the zstd stream).
SUBSTRATE_MAGIC_HEX="52bc537646db8e05"

RUNTIMES_DIR="$THIS_DIR/runtimes"
WASM_DIR="$THIS_DIR/wasm"
JAM_DIR="$THIS_DIR/jam"
LOGS_DIR="$THIS_DIR/logs"
IMPORTS_DIR="$THIS_DIR/imports"
README="$THIS_DIR/README.md"

WASM_PVM="$PROJECT_ROOT/target/release/wasm-pvm"

# Per-runtime compile timeout (seconds). LLVM passes scale poorly with module size,
# and a hung run shouldn't block the whole script. macOS lacks GNU `timeout`, so we
# fall back to `gtimeout` (coreutils) or skip the timeout entirely.
COMPILE_TIMEOUT="${COMPILE_TIMEOUT:-300}"

# When TRAP_FLOATS=1 (the default), pass --trap-floats so f32/f64 ops are
# replaced with runtime traps and compilation can finish past the float wall.
# Set TRAP_FLOATS=0 to surface the first float op as a hard error instead.
TRAP_FLOATS="${TRAP_FLOATS:-1}"
TRAP_FLOATS_ARGS=()
if [ "$TRAP_FLOATS" = "1" ]; then
  TRAP_FLOATS_ARGS=(--trap-floats)
fi

# Asset list (matches polkadot-fellows/runtimes v2.2.2). One filename per line.
RUNTIME_FILES=(
  "asset-hub-kusama_runtime-v2002002.compact.compressed.wasm"
  "asset-hub-polkadot_runtime-v2002002.compact.compressed.wasm"
  "bridge-hub-kusama_runtime-v2002002.compact.compressed.wasm"
  "bridge-hub-polkadot_runtime-v2002002.compact.compressed.wasm"
  "bulletin-polkadot_runtime-v2002002.compact.compressed.wasm"
  "collectives-polkadot_runtime-v2002002.compact.compressed.wasm"
  "coretime-kusama_runtime-v2002002.compact.compressed.wasm"
  "coretime-polkadot_runtime-v2002002.compact.compressed.wasm"
  "encointer-kusama_runtime-v2002002.compact.compressed.wasm"
  "glutton-kusama_runtime-v2002002.compact.compressed.wasm"
  "kusama_runtime-v2002002.compact.compressed.wasm"
  "people-kusama_runtime-v2002002.compact.compressed.wasm"
  "people-polkadot_runtime-v2002002.compact.compressed.wasm"
  "polkadot_runtime-v2002002.compact.compressed.wasm"
)

mkdir -p "$RUNTIMES_DIR" "$WASM_DIR" "$JAM_DIR" "$LOGS_DIR" "$IMPORTS_DIR"

log()  { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" >&2; }
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

# Resolve a `timeout` binary, or empty string if none available.
resolve_timeout_cmd() {
  if command -v timeout >/dev/null 2>&1; then echo "timeout"; return; fi
  if command -v gtimeout >/dev/null 2>&1; then echo "gtimeout"; return; fi
  echo ""
}
TIMEOUT_BIN="$(resolve_timeout_cmd)"

# Build the CLI in release if it doesn't exist or sources changed.
build_compiler() {
  log "Building wasm-pvm CLI (release)..."
  (cd "$PROJECT_ROOT" && cargo build --release -p wasm-pvm-cli) >&2
  [ -x "$WASM_PVM" ] || fail "CLI binary not found at $WASM_PVM after build"
}

download_runtime() {
  local file="$1"
  local dest="$RUNTIMES_DIR/$file"
  if [ -f "$dest" ]; then
    return 0
  fi
  log "Downloading $file"
  curl --fail --location --silent --show-error \
    --output "$dest.partial" \
    "$RELEASE_URL_BASE/$file"
  mv "$dest.partial" "$dest"
}

# Strip the 8-byte Substrate magic header. Returns 0 if header matched, 1 otherwise.
verify_substrate_magic() {
  local file="$1"
  local prefix
  prefix="$(xxd -p -l 8 "$file" | tr -d '\n')"
  [ "$prefix" = "$SUBSTRATE_MAGIC_HEX" ]
}

decompress_runtime() {
  local file="$1"      # path to .compact.compressed.wasm
  local out="$2"       # path to plain .wasm
  if [ -f "$out" ]; then
    return 0
  fi
  log "Decompressing $(basename "$file")"
  verify_substrate_magic "$file" \
    || fail "Substrate magic header missing in $file (expected $SUBSTRATE_MAGIC_HEX)"
  # Strip first 8 bytes, pipe rest to zstd.
  tail -c +9 "$file" | zstd -d -q -o "$out"
  # Verify resulting WASM starts with \0asm\01\00\00\00.
  local wasm_magic
  wasm_magic="$(xxd -p -l 8 "$out" | tr -d '\n')"
  if [ "$wasm_magic" != "0061736d01000000" ]; then
    rm -f "$out"
    fail "Decompressed file is not a valid WASM module ($wasm_magic != 0061736d01000000): $out"
  fi
}

# Generate "<name> = trap" lines for every (import "env" "<name>" ...) declaration
# in the WASM. The compiler also requires mappings for non-env imports; we conservatively
# trap every import regardless of module name.
#
# We use `wasm-tools print` if available (most readable). Otherwise we bail.
generate_imports_file() {
  local wasm="$1"
  local imports_path="$2"
  if [ -f "$imports_path" ]; then
    return 0
  fi
  if ! command -v wasm-tools >/dev/null 2>&1; then
    fail "wasm-tools is required to generate import maps. Install it with: cargo install wasm-tools"
  fi
  # Match (import "<module>" "<name>" ...) and emit "<name> = trap".
  # Names are deduplicated to avoid duplicate keys.
  wasm-tools print "$wasm" \
    | awk -F\" '/^[[:space:]]*\(import "/ { print $4 }' \
    | awk '!seen[$0]++ { print $0 " = trap" }' \
    > "$imports_path"
}

# Convert "12345" bytes to "12.06 KiB" / "11.77 MiB" for compact display.
human_size() {
  local bytes="$1"
  awk -v b="$bytes" 'BEGIN {
    if (b >= 1048576) printf("%.2f MiB", b / 1048576);
    else if (b >= 1024) printf("%.2f KiB", b / 1024);
    else printf("%d B", b);
  }'
}

filesize() {
  # Portable file size in bytes (BSD stat on macOS, GNU stat elsewhere).
  if stat -f%z "$1" >/dev/null 2>&1; then stat -f%z "$1"; else stat -c%s "$1"; fi
}

# Count `f32.*` / `f64.*` op kinds in the WASM. Returns "0" or a comma-separated list.
float_op_kinds() {
  local wasm="$1"
  if ! command -v wasm-tools >/dev/null 2>&1; then
    echo "?"; return
  fi
  local kinds
  # The regex has two alternatives so we capture both:
  #   1. `(f32|f64).<anything>`  — every plain f32/f64 op, including
  #      mnemonics whose tail contains digits (e.g. `f32.convert_i32_s`,
  #      `f64.promote_f32`).
  #   2. `i{32,64}.(trunc[_sat]?|reinterpret)_f{32,64}[_su]?` — the
  #      integer-result float ops (`i32.trunc_f64_s`,
  #      `i32.trunc_sat_f64_u`, `i64.reinterpret_f64`, …) which consume a
  #      float and would still trap under `--trap-floats`. Without #2 the
  #      table silently under-reports.
  kinds="$(wasm-tools print "$wasm" 2>/dev/null \
            | grep -oE '(f32|f64)\.[a-z0-9_]+|i(32|64)\.(trunc(_sat)?|reinterpret)_(f32|f64)(_[su])?' \
            | sort -u \
            | tr '\n' ',' \
            | sed 's/,$//')"
  if [ -z "$kinds" ]; then echo "none"; else echo "$kinds"; fi
}

count_imports() {
  local wasm="$1"
  if ! command -v wasm-tools >/dev/null 2>&1; then
    echo "?"; return
  fi
  wasm-tools print "$wasm" 2>/dev/null | grep -cE '^[[:space:]]*\(import '
}

# Run the compiler with optional timeout. Writes stdout+stderr to "$log_path".
# Echoes a status word: "ok", "fail", or "timeout".
run_compile() {
  local wasm="$1"
  local jam="$2"
  local imports="$3"
  local log_path="$4"
  local rc=0

  rm -f "$jam"

  if [ -n "$TIMEOUT_BIN" ]; then
    "$TIMEOUT_BIN" "$COMPILE_TIMEOUT" "$WASM_PVM" compile "$wasm" -o "$jam" \
      --imports "$imports" "${TRAP_FLOATS_ARGS[@]}" \
      >"$log_path" 2>&1 || rc=$?
    if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
      echo "timeout"; return 0
    fi
  else
    "$WASM_PVM" compile "$wasm" -o "$jam" \
      --imports "$imports" "${TRAP_FLOATS_ARGS[@]}" \
      >"$log_path" 2>&1 || rc=$?
  fi

  if [ "$rc" -eq 0 ] && [ -f "$jam" ]; then
    echo "ok"
  else
    echo "fail"
  fi
}

# Pull the most informative error line out of a compile log. Two failure paths:
#   1. CLI prints "Error: Compilation failed\nCaused by:\n    <real reason>"
#      — we want the line after "Caused by:".
#   2. LLVM aborts the process via abort() with "LLVM ERROR: ..." straight to
#      stderr (e.g. instcombine fixpoint failures). No "Caused by:" line; grab
#      the LLVM ERROR line directly.
extract_error_reason() {
  local log_path="$1"
  local reason
  reason=$(awk '/^Caused by:/ { getline; sub(/^[[:space:]]+/, ""); print; exit }' "$log_path" 2>/dev/null)
  if [ -z "$reason" ]; then
    reason=$(grep -m 1 -E '^LLVM ERROR:' "$log_path" 2>/dev/null || true)
  fi
  echo "$reason"
}

# ----------------------------------------------------------------------------
# Main
# ----------------------------------------------------------------------------

build_compiler

# Per-runtime results. Each entry is a pipe-separated row used to render the
# README: name|compressed|wasm|imports|float_ops|status|jam|reason|elapsed.
declare -a RESULTS

for file in "${RUNTIME_FILES[@]}"; do
  short="${file%.compact.compressed.wasm}"
  log "Processing $short"

  download_runtime "$file"

  compressed_path="$RUNTIMES_DIR/$file"
  wasm_path="$WASM_DIR/$short.wasm"
  jam_path="$JAM_DIR/$short.jam"
  imports_path="$IMPORTS_DIR/$short.imports"
  log_path="$LOGS_DIR/$short.log"

  decompress_runtime "$compressed_path" "$wasm_path"
  generate_imports_file "$wasm_path" "$imports_path"

  compressed_size=$(filesize "$compressed_path")
  wasm_size=$(filesize "$wasm_path")
  import_count="$(count_imports "$wasm_path")"
  floats="$(float_op_kinds "$wasm_path")"

  start_ts=$(date +%s)
  status="$(run_compile "$wasm_path" "$jam_path" "$imports_path" "$log_path")"
  end_ts=$(date +%s)
  elapsed=$((end_ts - start_ts))

  jam_size="-"
  reason=""
  case "$status" in
    ok)
      jam_size=$(filesize "$jam_path")
      reason=""
      ;;
    fail)
      reason="$(extract_error_reason "$log_path")"
      [ -z "$reason" ] && reason="(see $(basename "$log_path"))"
      ;;
    timeout)
      reason="compile exceeded ${COMPILE_TIMEOUT}s timeout"
      ;;
  esac

  RESULTS+=("$short|$compressed_size|$wasm_size|$import_count|$floats|$status|$jam_size|$reason|${elapsed}s")
done

# ----------------------------------------------------------------------------
# Render README
# ----------------------------------------------------------------------------

log "Writing README to $README"

{
  echo "# Polkadot Runtimes — WASM → PVM Compilation"
  echo
  echo "Compilation results for the [polkadot-fellows/runtimes ${RELEASE_TAG}](https://github.com/polkadot-fellows/runtimes/releases/tag/${RELEASE_TAG}) release, produced by \`./compile.sh\`."
  echo
  echo "Runtimes ship as Substrate \"compact compressed\" blobs: an 8-byte magic header (\`52 BC 53 76 46 DB 8E 05\`) followed by zstd-compressed WASM. The script strips the header, decompresses, verifies the WebAssembly magic (\`\\0asm\`), generates a trap-all import map, then invokes \`wasm-pvm compile\`."
  echo
  if [ "$TRAP_FLOATS" = "1" ]; then
    echo "Compiled with **\`--trap-floats\`** (f32/f64 ops replaced with runtime traps so compilation can finish past the float wall)."
  else
    echo "Compiled with \`--trap-floats\` disabled — the first f32/f64 op rejects compilation. Re-run with \`TRAP_FLOATS=1\` (the script default) to replace float ops with runtime traps and skip past the float wall."
  fi
  echo
  echo "## Results"
  echo
  echo "| Runtime | Compressed | WASM | Imports | Status | JAM | Time | Reason |"
  echo "|---------|-----------:|-----:|--------:|--------|----:|-----:|--------|"
  for row in "${RESULTS[@]}"; do
    IFS='|' read -r name comp wasm imports floats status jam reason elapsed <<< "$row"
    comp_h="$(human_size "$comp")"
    wasm_h="$(human_size "$wasm")"
    if [ "$jam" != "-" ]; then jam_h="$(human_size "$jam")"; else jam_h="-"; fi
    case "$status" in
      ok)      status_md=":white_check_mark: ok" ;;
      fail)    status_md=":x: fail" ;;
      timeout) status_md=":hourglass: timeout" ;;
      *)       status_md="$status" ;;
    esac
    # Escape pipes in reason so they don't break the table.
    reason_escaped="${reason//|/\\|}"
    printf '| %s | %s | %s | %s | %s | %s | %s | %s |\n' \
      "$name" "$comp_h" "$wasm_h" "$imports" "$status_md" "$jam_h" "$elapsed" "$reason_escaped"
  done
  echo
  echo "## Float operations per runtime"
  echo
  echo "The compiler rejects \`f32\`/\`f64\` instructions (PVM has no floating-point support). This table lists the float-op kinds present in each module."
  echo
  echo "| Runtime | Float ops |"
  echo "|---------|-----------|"
  for row in "${RESULTS[@]}"; do
    IFS='|' read -r name _ _ _ floats _ _ _ _ <<< "$row"
    printf '| %s | %s |\n' "$name" "${floats:-?}"
  done
  echo
  echo "## How to reproduce"
  echo
  echo '```bash'
  echo "cd examples/polkadot && ./compile.sh"
  echo '```'
  echo
  echo "Set \`COMPILE_TIMEOUT=600\` to relax the per-runtime time budget, or \`TRAP_FLOATS=0\` to disable the \`--trap-floats\` flag (so the first f32/f64 op surfaces as a hard error instead of becoming a runtime trap). The pipeline is currently pinned to release ${RELEASE_TAG}; overriding \`RELEASE_TAG\` is rejected until the asset list is derived from the tag."
  echo
  echo "Compressed downloads land in \`runtimes/\`, decompressed modules in \`wasm/\`, generated trap-all import maps in \`imports/\`, JAM outputs in \`jam/\`, and full compile logs in \`logs/\`. All five directories are gitignored."
} > "$README"

log "Done."
