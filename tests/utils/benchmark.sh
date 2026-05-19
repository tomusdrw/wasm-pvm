#!/usr/bin/env bash
# Benchmark script: measures JAM file size, gas usage, and execution time.
#
# Usage:
#   ./tests/utils/benchmark.sh [--no-opt] [--base <branch>] [--current <branch>]
#
# --no-opt: Compile JAM files with all PVM-level optimizations disabled.
#           Without this flag (default), uses the standard optimized build.
#
# Without --base/--current, builds and benchmarks the current code.
# With --base/--current, compares two branches side by side.
#
# Prerequisites: cargo, bun, node, python3 must be in PATH.
# Run from the project root directory.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ANAN_CLI="$PROJECT_ROOT/vendor/anan-as/dist/bin/index.js"
DEFAULT_JAM_DIR="$PROJECT_ROOT/tests/build/jam"
GAS_BUDGET=100000000
OUTER_GAS=10000000000
INNER_GAS=100000000

# Will be set based on --no-opt flag
JAM_DIR=""
COMPILER_JAM=""
NO_OPT=false

# Tunable optimizations only. `--debug-skip-llvm-passes` is intentionally NOT in this
# list: it disables the entire LLVM pass pipeline including `mem2reg`, which
# the PVM backend requires (otherwise IR retains `alloca` and lowering fails
# with `Unsupported WASM feature: LLVM opcode Alloca`). It is a frontend
# debugging escape hatch, not an optimization toggle.
NO_OPT_FLAGS=(
  --no-peephole
  --no-register-cache
  --no-icmp-fusion
  --no-shrink-wrap
  --no-dead-store-elim
  --no-const-prop
  --no-inline
  --no-cross-block-cache
  --no-aggressive-regalloc
)

# Representative benchmarks: (jam_basename, args, pc, description, wasm_source)
# jam_basename: name of the JAM file in JAM_DIR, or "EXT:<path>" for external JAM files.
# wasm_source: "wat:<path>" for WAT files, "wasm:<path>" for WASM files, empty to skip.
# pc: initial program counter (default 0).
BENCHMARKS=(
  "add|0500000007000000|0|add(5,7)|wat:tests/fixtures/wat/add.jam.wat"
  "fibonacci|14000000|0|fib(20)|wat:tests/fixtures/wat/fibonacci.jam.wat"
  "factorial|0a000000|0|factorial(10)|wat:tests/fixtures/wat/factorial.jam.wat"
  "is-prime|19000000|0|is_prime(25)|wat:tests/fixtures/wat/is-prime.jam.wat"
  "as-fibonacci|0a000000|0|AS fib(10)|wasm:tests/build/wasm/fibonacci.wasm"
  "as-factorial|07000000|0|AS factorial(7)|wasm:tests/build/wasm/factorial.wasm"
  "as-gcd|00e10700c8000000|0|AS gcd(2017,200)|wasm:tests/build/wasm/gcd.wasm"
  "as-decoder-test|00000000|0|AS decoder|wasm:tests/build/wasm/decoder-test.wasm"
  "as-array-test|00000000|0|AS array|wasm:tests/build/wasm/array-test.wasm"
  "regalloc-two-loops|f4010000|0|regalloc two loops(500)|wat:tests/fixtures/wat/regalloc-two-loops.jam.wat"
  "aslan-fib|2a0000|5|aslan-fib accumulate|wat:tests/fixtures/wat/aslan-fib.jam.wat"
  "host-call-log|00000000|0|host-call-log|wat:tests/fixtures/wat/host-call-log.jam.wat"
  "anan-as-compiler||0|anan-as PVM interpreter|wasm:vendor/anan-as/dist/build/compiler.wasm"
  "aslan-ecalli|00|0|aslan-ecalli accumulate|wat:tests/fixtures/wat/aslan-ecalli.jam.wat"
  "blake2b|2000000000000000616263|0|blake2b(\"abc\",32)|wat:tests/fixtures/wat/blake2b.jam.wat"
  "sha512|616263|0|sha512(\"abc\")|wat:tests/fixtures/wat/sha512.jam.wat"
  # u128 microbenchmarks: 1000 iterations (args = 1000u32 LE = 0xe8030000)
  # measure dynamic gas savings from `libcall_recognition` (__multi3 /
  # __udivti3 body replacement). Fast variants always take the b_hi
  # specialization fast path; the -slow variant always takes the slow path.
  "u128-mul-bench|e8030000|0|u128 mul x1000|wat:tests/fixtures/wat/u128-mul-bench.jam.wat"
  "u128-div-bench|e8030000|0|u128 div(fast) x1000|wat:tests/fixtures/wat/u128-div-bench.jam.wat"
  "u128-div-bench-slow|e8030000|0|u128 div(slow) x1000|wat:tests/fixtures/wat/u128-div-bench-slow.jam.wat"
  "sha512|616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161|0|sha512(240x\"a\" — multi-block + two-pad)|wat:tests/fixtures/wat/sha512.jam.wat"
)

# Polkadot fellowship runtime benchmarks (polkadot-fellows/runtimes v2.2.2).
# Each is a size-only entry — Polkadot runtimes need substrate to actually run,
# so the benchmark only reports JAM/code size. WASMs are NOT checked into the
# repo; populate them once with `cd examples/polkadot && ./compile.sh` (which
# downloads from the GitHub release, strips the Substrate magic header, and
# zstd-decompresses). After that, every benchmark run picks them up
# automatically via `compile_polkadot_jams` below. If a runtime's WASM is
# missing it's silently skipped, so a fresh clone still benchmarks cleanly.
POLKADOT_RUNTIMES=(
  asset-hub-kusama_runtime-v2002002
  asset-hub-polkadot_runtime-v2002002
  bridge-hub-kusama_runtime-v2002002
  bridge-hub-polkadot_runtime-v2002002
  bulletin-polkadot_runtime-v2002002
  collectives-polkadot_runtime-v2002002
  coretime-kusama_runtime-v2002002
  coretime-polkadot_runtime-v2002002
  encointer-kusama_runtime-v2002002
  glutton-kusama_runtime-v2002002
  kusama_runtime-v2002002
  people-kusama_runtime-v2002002
  people-polkadot_runtime-v2002002
  polkadot_runtime-v2002002
)

# Per-runtime compile timeout, in seconds. Pre-#225 versions of the compiler
# stall indefinitely on real-world modules, so a cap keeps `--base/--current`
# branch comparisons bounded.
POLKADOT_COMPILE_TIMEOUT="${POLKADOT_COMPILE_TIMEOUT:-300}"

# Polkadot runtimes do not get one row per benchmark — that's 14 long-name rows
# of large numbers that drown out the rest of the suite. Instead, after the
# standard `BENCHMARKS` loop finishes, `polkadot_summary_row` emits a single
# row whose WASM / JAM / Code columns are the *sum* across all populated
# runtimes. Any per-runtime regression (or improvement) shifts the sum, so
# the row is still useful for change detection; per-runtime detail lives in
# `examples/polkadot/README.md` after running `compile.sh` there.

# Return "imports_path|adapter_path" for benchmarks that need them, or empty.
benchmark_imports_for() {
  local basename="$1"
  case "$basename" in
    anan-as-compiler)
      echo "tests/fixtures/imports/anan-as-compiler.imports|tests/fixtures/imports/anan-as-compiler.adapter.wat"
      ;;
    aslan-fib)
      echo "|tests/fixtures/imports/aslan-fib.adapter.wat"
      ;;
    aslan-ecalli)
      echo "tests/fixtures/imports/aslan-ecalli.imports|"
      ;;
    *)
      echo ""
      ;;
  esac
}

# PVM-in-PVM benchmarks: (jam_basename, args, pc, description)
# These run the JAM file inside the anan-as PVM interpreter (pvm-in-pvm).
# Use "TRAP" as jam_basename for a synthetic 1-byte TRAP program.
# Use "EXT:<path>" as jam_basename for an external JAM file (path relative to PROJECT_ROOT).
# pc: initial program counter (default 0).
PVM_IN_PVM_BENCHMARKS=(
  "TRAP||0|PiP TRAP"
  "add|0500000007000000|0|PiP add(5,7)"
  "host-call-log|00000000|0|PiP host-call-log"
  "as-fibonacci|0a000000|0|PiP AS fib(10)"
  "EXT:tests/fixtures/external/jam-sdk-fib.jam|0100000002000000030000000000000000000000|5|PiP JAM-SDK fib(10)"
  "EXT:tests/fixtures/external/jambrains-fib.jam|0100000002000000030000000000000000000000|5|PiP Jambrains fib(10)"
  "EXT:tests/fixtures/external/jade-fib.jam|0100000002000000030000000000000000000000|5|PiP JADE fib(10)"
  "aslan-fib|2a0000|5|PiP aslan-fib accumulate"
  "blake2b|2000000000000000616263|0|PiP blake2b(\"abc\",32)"
  "sha512|616263|0|PiP sha512(\"abc\")"
  "sha512|616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161|0|PiP sha512(240x\"a\")"
)

# Extract raw PVM code size (instruction bytes only) from a JAM file.
# Strips metadata prefix, SPI header, RO/RW data, jump table, and mask.
jam_code_size() {
  local jam_file="$1"
  if [ ! -f "$jam_file" ]; then
    echo "-"
    return
  fi
  python3 -c "
import sys

data = open(sys.argv[1], 'rb').read()
if len(data) < 15:
    print('-')
    sys.exit(0)
off = 0

# PVM varint: first byte determines length, remaining bytes are little-endian
def read_varint(d, o):
    b = d[o]
    if b < 0x80: return b, 1
    elif b < 0xc0: return ((b-0x80)<<8)|d[o+1], 2
    elif b < 0xe0: return ((b-0xc0)<<16)|d[o+1]|(d[o+2]<<8), 3
    else: return ((b-0xe0)<<24)|d[o+1]|(d[o+2]<<8)|(d[o+3]<<16), 4

try:
    meta_len, n = read_varint(data, off)
    off += n + meta_len
    ro_len = data[off]|(data[off+1]<<8)|(data[off+2]<<16); off += 3
    rw_len = data[off]|(data[off+1]<<8)|(data[off+2]<<16)
    off += 3 + 2 + 3
    off += ro_len + rw_len + 4
    jt_len, n = read_varint(data, off); off += n
    off += 1
    code_len, _ = read_varint(data, off)
    print(code_len)
except (IndexError, ValueError):
    print('-')
" "$jam_file"
}

# Get WASM size from a source spec ("wat:<path>" or "wasm:<path>")
wasm_size() {
  local spec="$1"
  if [ -z "$spec" ]; then
    echo "-"
    return
  fi

  local kind="${spec%%:*}"
  local filepath="${spec#*:}"
  filepath="$PROJECT_ROOT/$filepath"

  if [ ! -f "$filepath" ]; then
    echo "-"
    return
  fi

  if [ "$kind" = "wat" ]; then
    local tmp_wasm
    tmp_wasm=$(mktemp "${TMPDIR:-/tmp}/wasm-size-XXXXXX")
    if wat2wasm "$filepath" -o "$tmp_wasm" 2>/dev/null; then
      wc -c < "$tmp_wasm" | tr -d ' '
      rm -f "$tmp_wasm"
    else
      rm -f "$tmp_wasm"
      echo "-"
    fi
  elif [ "$kind" = "wasm" ]; then
    wc -c < "$filepath" | tr -d ' '
  else
    echo "-"
  fi
}

# Compile a single benchmark source to JAM with the given extra flags.
# Args: wasm_source_spec, output_jam_path, extra_flags...
compile_benchmark() {
  local spec="$1"
  local output="$2"
  local basename="$3"
  shift 3
  local extra_flags=("$@")

  local kind="${spec%%:*}"
  local filepath="${spec#*:}"
  filepath="$PROJECT_ROOT/$filepath"

  if [ ! -f "$filepath" ]; then
    echo "SKIP: source not found: $filepath" >&2
    return 1
  fi

  local cmd=("$PROJECT_ROOT/target/release/wasm-pvm" compile "$filepath" -o "$output")

  # Add imports/adapter if configured for this benchmark
  local imp_spec
  imp_spec=$(benchmark_imports_for "$basename")
  if [ -n "$imp_spec" ]; then
    IFS='|' read -r imports_path adapter_path <<< "$imp_spec"
    if [ -n "$imports_path" ] && [ -f "$PROJECT_ROOT/$imports_path" ]; then
      cmd+=(--imports "$PROJECT_ROOT/$imports_path")
    fi
    if [ -n "$adapter_path" ] && [ -f "$PROJECT_ROOT/$adapter_path" ]; then
      cmd+=(--adapter "$PROJECT_ROOT/$adapter_path")
    fi
  fi

  cmd+=("${extra_flags[@]}")
  "${cmd[@]}" 2>&1
}

# Recompile all benchmark sources into a separate directory with no-opt flags.
compile_noopt_jams() {
  local noopt_dir="$1"
  rm -rf "$noopt_dir"
  mkdir -p "$noopt_dir"

  echo "Recompiling benchmarks without PVM optimizations..." >&2
  for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r basename _args _pc _desc wasm_src <<< "$entry"
    # Skip external JAM files (can't recompile) and entries without sources
    if [ -z "$wasm_src" ] || [[ "$basename" == EXT:* ]]; then
      continue
    fi
    # Polkadot entries get their own compile path (they need --trap-floats and
    # an auto-generated trap-all import map).
    if [[ "$basename" == polkadot-* ]]; then
      continue
    fi
    local output="$noopt_dir/$basename.jam"
    # Tolerate per-fixture compile failures (missing source, missing
    # vendor/anan-as build, etc.) so one bad entry doesn't kill the whole
    # table under `set -e`. `benchmark_one` renders SKIP rows for missing
    # JAMs, matching the polkadot path's `|| rc=$?` tolerance.
    compile_benchmark "$wasm_src" "$output" "$basename" "${NO_OPT_FLAGS[@]}" >&2 || true
  done
  compile_polkadot_jams "$noopt_dir" "${NO_OPT_FLAGS[@]}"
}

# Resolve a `timeout` binary or empty string if none is available.
polkadot_timeout_bin() {
  if command -v timeout >/dev/null 2>&1; then echo "timeout"; return; fi
  if command -v gtimeout >/dev/null 2>&1; then echo "gtimeout"; return; fi
  echo ""
}

# Compile each populated Polkadot runtime (`examples/polkadot/wasm/*.wasm`) to
# `<out_dir>/polkadot-<runtime>.jam` with `--trap-floats` and the auto-generated
# trap-all import map. Silently does nothing when no WASMs are present so a
# fresh clone benchmarks cleanly. `extra_flags` (optional) are forwarded to
# `wasm-pvm compile` after `--trap-floats`, so the no-opt mode can pass the
# usual `--no-peephole` etc.
compile_polkadot_jams() {
  local out_dir="$1"
  shift
  local extra_flags=("$@")
  local polkadot_dir="$PROJECT_ROOT/examples/polkadot"
  local wasm_dir="$polkadot_dir/wasm"
  local imports_dir="$polkadot_dir/imports"

  if [ ! -d "$wasm_dir" ]; then
    return 0
  fi

  shopt -s nullglob
  local wasms=("$wasm_dir"/*.wasm)
  shopt -u nullglob

  if [ ${#wasms[@]} -eq 0 ]; then
    return 0
  fi

  mkdir -p "$out_dir" "$imports_dir"
  local timeout_bin
  timeout_bin="$(polkadot_timeout_bin)"

  echo "Compiling ${#wasms[@]} Polkadot runtime(s) to $out_dir ..." >&2
  for wasm in "${wasms[@]}"; do
    local short imports out
    short="$(basename "$wasm" .wasm)"
    imports="$imports_dir/$short.imports"
    out="$out_dir/polkadot-$short.jam"

    # Auto-generate trap-all import map on first run (cached afterwards).
    if [ ! -f "$imports" ]; then
      if command -v wasm-tools >/dev/null 2>&1; then
        wasm-tools print "$wasm" 2>/dev/null \
          | awk -F\" '/^[[:space:]]*\(import "/ { print $4 }' \
          | awk '!seen[$0]++ { print $0 " = trap" }' \
          > "$imports"
      else
        echo "  skip $short: missing imports map and wasm-tools not installed" >&2
        continue
      fi
    fi

    local cmd=(
      "$PROJECT_ROOT/target/release/wasm-pvm" compile "$wasm" -o "$out"
      --imports "$imports" --trap-floats
    )
    # `set -u` chokes on `${empty[@]}` in older bashes, so guard the expansion.
    if [ ${#extra_flags[@]} -gt 0 ]; then
      cmd+=("${extra_flags[@]}")
    fi
    rm -f "$out"
    local rc=0
    if [ -n "$timeout_bin" ]; then
      "$timeout_bin" "$POLKADOT_COMPILE_TIMEOUT" "${cmd[@]}" >/dev/null 2>&1 || rc=$?
    else
      "${cmd[@]}" >/dev/null 2>&1 || rc=$?
    fi
    # Silent SKIPs leave `polkadot_summary_row` reporting `Σ N/14` with no
    # indication of which runtime didn't make it; surface a short reason
    # here. coreutils `timeout` exits 124 (or 137 on SIGKILL) on time-out.
    if [ "$rc" -ne 0 ]; then
      if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
        echo "  ${short}: timed out after ${POLKADOT_COMPILE_TIMEOUT}s — JAM missing from summary row" >&2
      else
        echo "  ${short}: compile failed (rc=${rc}) — JAM missing from summary row" >&2
      fi
    fi
  done
}

benchmark_one() {
  local jam_file="$1"
  local args="$2"
  local pc="$3"
  local desc="$4"
  local size code_size gas_used time_ms

  if [ ! -f "$jam_file" ]; then
    echo "SKIP|$desc|missing"
    return
  fi

  size=$(wc -c < "$jam_file" | tr -d ' ')
  code_size=$(jam_code_size "$jam_file")

  if [ -z "$args" ]; then
    # Size-only benchmark (e.g. large compiler)
    echo "OK|$desc|$size|$code_size|-|-"
    return
  fi

  # Run 3 times and take the median time
  local times=()
  local gas_remaining=""
  for _i in 1 2 3; do
    local start_ns end_ns elapsed_ms output exit_code
    start_ns=$(python3 -c "import time; print(int(time.time_ns()))")
    exit_code=0
    output=$(node "$ANAN_CLI" run --spi --no-logs --gas=$GAS_BUDGET --pc="$pc" "$jam_file" "0x$args" 2>&1) || exit_code=$?
    end_ns=$(python3 -c "import time; print(int(time.time_ns()))")
    elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))

    if [ "$exit_code" -ne 0 ] && ! echo "$output" | grep -q 'Gas remaining:'; then
      continue
    fi

    times+=("$elapsed_ms")

    if [ -z "$gas_remaining" ]; then
      gas_remaining=$(echo "$output" | grep -o 'Gas remaining: [0-9]*' | grep -o '[0-9]*' || echo "")
    fi
  done

  if [ "${#times[@]}" -eq 0 ]; then
    echo "SKIP|$desc|$size (run failed)"
    return
  fi

  # Sort and pick median (middle element)
  IFS=$'\n' read -r -d '' -a sorted < <(printf '%s\n' "${times[@]}" | sort -n; printf '\0') || true
  local mid=$(( ${#sorted[@]} / 2 ))
  time_ms="${sorted[$mid]}"

  if [ -n "$gas_remaining" ]; then
    gas_used=$((GAS_BUDGET - gas_remaining))
  else
    gas_used="error"
  fi

  echo "OK|$desc|$size|$code_size|$gas_used|${time_ms}ms"
}

# Build binary args file for pvm-in-pvm execution.
# Format: gas(8LE) + pc(4LE) + program_len(4LE) + inner_args_len(4LE) + program + inner_args
build_pvm_in_pvm_args() {
  local jam_file="$1"
  local inner_args_hex="$2"
  local out_file="$3"
  local pc="${4:-0}"

  local program_len
  program_len=$(wc -c < "$jam_file" | tr -d ' ')

  local inner_args_bytes=""
  local inner_args_len=0
  if [ -n "$inner_args_hex" ]; then
    # Validate: must be even-length hex string
    if ! [[ "$inner_args_hex" =~ ^[0-9a-fA-F]*$ ]]; then
      echo "ERROR: inner_args_hex contains non-hex characters: $inner_args_hex" >&2
      return 1
    fi
    if (( ${#inner_args_hex} % 2 != 0 )); then
      echo "ERROR: inner_args_hex has odd length (${#inner_args_hex}): $inner_args_hex" >&2
      return 1
    fi
    inner_args_bytes=$(echo -n "$inner_args_hex" | sed 's/../\\x&/g')
    inner_args_len=$(( ${#inner_args_hex} / 2 ))
  fi

  # Build header (20 bytes) using python3 for reliable LE encoding
  python3 -c "
import struct,sys
sys.stdout.buffer.write(struct.pack('<QIII',${INNER_GAS},${pc},${program_len},${inner_args_len}))
" > "$out_file"

  # Append program bytes
  cat "$jam_file" >> "$out_file"

  # Append inner args (use %b to expand \xNN escapes without treating % as a format specifier)
  if [ -n "$inner_args_hex" ]; then
    printf '%b' "$inner_args_bytes" >> "$out_file"
  fi
}

# Benchmark a JAM file running through pvm-in-pvm.
# Reports outer gas used (interpreter overhead + inner execution).
benchmark_pvm_in_pvm() {
  local jam_file="$1"
  local args="$2"
  local pc="$3"
  local desc="$4"
  local size code_size gas_used time_ms

  if [ ! -f "$jam_file" ]; then
    echo "SKIP|$desc|missing"
    return
  fi

  if [ ! -f "$COMPILER_JAM" ]; then
    echo "SKIP|$desc|no compiler.jam"
    return
  fi

  size=$(wc -c < "$jam_file" | tr -d ' ')
  code_size=$(jam_code_size "$jam_file")

  # Build args binary file (trap ensures cleanup even on early exit)
  local tmp_args
  tmp_args=$(mktemp "${TMPDIR:-/tmp}/pvm-bench-args-XXXXXX")
  # shellcheck disable=SC2064  # tmp_args is intentionally expanded now, not at trap time
  trap "rm -f '$tmp_args'" RETURN
  build_pvm_in_pvm_args "$jam_file" "$args" "$tmp_args" "$pc"

  # Run 3 times and take the median time
  local times=()
  local gas_remaining=""
  for _i in 1 2 3; do
    local start_ns end_ns elapsed_ms output exit_code
    start_ns=$(python3 -c "import time; print(int(time.time_ns()))")
    exit_code=0
    output=$(node "$ANAN_CLI" run --spi --no-logs --gas=$OUTER_GAS "$COMPILER_JAM" "$tmp_args" 2>&1) || exit_code=$?
    end_ns=$(python3 -c "import time; print(int(time.time_ns()))")
    elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))

    # Use grep -a to force text mode (output may contain binary from file path warnings)
    if [ "$exit_code" -ne 0 ] && ! echo "$output" | grep -aq 'Gas remaining:'; then
      continue
    fi

    times+=("$elapsed_ms")

    if [ -z "$gas_remaining" ]; then
      gas_remaining=$(echo "$output" | grep -ao 'Gas remaining: [0-9]*' | grep -o '[0-9]*' || echo "")
    fi
  done

  rm -f "$tmp_args"

  if [ "${#times[@]}" -eq 0 ]; then
    echo "SKIP|$desc|$size (run failed)"
    return
  fi

  # Sort and pick median (middle element)
  IFS=$'\n' read -r -d '' -a sorted < <(printf '%s\n' "${times[@]}" | sort -n; printf '\0') || true
  local mid=$(( ${#sorted[@]} / 2 ))
  time_ms="${sorted[$mid]}"

  if [ -n "$gas_remaining" ]; then
    gas_used=$((OUTER_GAS - gas_remaining))
  else
    gas_used="error"
  fi

  echo "OK|$desc|$size|$code_size|$gas_used|${time_ms}ms"
}

# Emit one row summarising every populated Polkadot runtime as
# WASM/JAM/Code *sums*. Returns nothing (silently skips) when no runtimes
# are populated, so a fresh clone benchmarks cleanly.
polkadot_summary_row() {
  local total_wasm=0 total_jam=0 total_code=0 count=0
  for rt in "${POLKADOT_RUNTIMES[@]}"; do
    local wasm="$PROJECT_ROOT/examples/polkadot/wasm/$rt.wasm"
    local jam="$JAM_DIR/polkadot-$rt.jam"
    [ -f "$wasm" ] || continue
    [ -f "$jam" ] || continue
    local w j c
    w=$(filesize "$wasm")
    j=$(filesize "$jam")
    c=$(jam_code_size "$jam")
    total_wasm=$((total_wasm + w))
    total_jam=$((total_jam + j))
    if [ "$c" != "-" ]; then
      total_code=$((total_code + c))
    fi
    count=$((count + 1))
  done
  if [ "$count" -gt 0 ]; then
    local total=${#POLKADOT_RUNTIMES[@]}
    local desc="polkadot v2.2.2 (Σ ${count}/${total})"
    printf "| %-20s | %10s | %10s | %10s | %10s | %10s |\n" \
      "$desc" "$total_wasm" "$total_jam" "$total_code" "-" "-"
  fi
}

# Portable file size in bytes (BSD stat on macOS, GNU stat elsewhere).
filesize() {
  if stat -f%z "$1" >/dev/null 2>&1; then stat -f%z "$1"; else stat -c%s "$1"; fi
}

run_benchmarks() {
  local label="$1"
  echo "## $label"
  echo ""
  echo "| Benchmark | WASM Size | JAM Size | Code Size | Gas Used | Time (median of 3) |"
  echo "|-----------|-----------|----------|-----------|----------|-------------------|"

  for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r basename args pc desc wasm_src <<< "$entry"
    pc="${pc:-0}"
    local jam_file
    if [[ "$basename" == EXT:* ]]; then
      local ext_path="${basename#EXT:}"
      jam_file="$PROJECT_ROOT/$ext_path"
    else
      jam_file="$JAM_DIR/$basename.jam"
    fi
    local wsize
    wsize=$(wasm_size "$wasm_src")
    local result
    result=$(benchmark_one "$jam_file" "$args" "$pc" "$desc")

    IFS='|' read -r status rdesc size code_size gas time <<< "$result"
    if [ "$status" = "OK" ]; then
      printf "| %-20s | %10s | %10s | %10s | %10s | %10s |\n" "$rdesc" "$wsize" "$size" "$code_size" "$gas" "$time"
    else
      printf "| %-20s | %10s | %10s | %10s | %10s | %10s |\n" "$rdesc" "$wsize" "SKIP" "-" "-" "-"
    fi
  done
  polkadot_summary_row
  echo ""

  # PVM-in-PVM benchmarks
  if [ -f "$COMPILER_JAM" ]; then
    echo "### PVM-in-PVM"
    echo ""
    echo "| Benchmark | JAM Size | Code Size | Outer Gas Used | Time (median of 3) |"
    echo "|-----------|----------|-----------|----------------|-------------------|"

    for entry in "${PVM_IN_PVM_BENCHMARKS[@]}"; do
      IFS='|' read -r basename args pc desc <<< "$entry"
      pc="${pc:-0}"
      local jam_file cleanup_jam=false
      if [ "$basename" = "TRAP" ]; then
        # Minimal valid SPI blob with just a TRAP instruction in the code section.
        # Format: metadata(varint 0) + SPI header(11 bytes) + code_blob_len(u32) + PVM blob
        # PVM blob: jump_table_len(varint 0) + item_bytes(0) + code_len(varint 1) + code(0x00) + mask(0x01)
        jam_file=$(mktemp "${TMPDIR:-/tmp}/pvm-bench-trap-XXXXXX")
        python3 -c "
import sys, struct
out = bytearray()
out.append(0)           # metadata_len = 0 (varint)
out.extend(b'\x00\x00\x00')  # ro_data_len = 0 (u24 LE)
out.extend(b'\x00\x00\x00')  # rw_data_len = 0 (u24 LE)
out.extend(b'\x00\x00')      # heap_pages = 0 (u16 LE)
out.extend(b'\x00\x00\x00')  # stack_size = 0 (u24 LE)
# no RO/RW data
# PVM blob: jt_len(0) + item_bytes(0) + code_len(1) + code(TRAP=0x00) + mask(0x01)
blob = bytearray([0, 0, 1, 0x00, 0x01])
out.extend(struct.pack('<I', len(blob)))  # code_blob_len
out.extend(blob)
sys.stdout.buffer.write(out)
" > "$jam_file"
        cleanup_jam=true
      elif [[ "$basename" == EXT:* ]]; then
        # External JAM file (path relative to PROJECT_ROOT)
        local ext_path="${basename#EXT:}"
        jam_file="$PROJECT_ROOT/$ext_path"
      else
        jam_file="$JAM_DIR/$basename.jam"
      fi
      local result
      result=$(benchmark_pvm_in_pvm "$jam_file" "$args" "$pc" "$desc")
      if [ "$cleanup_jam" = true ]; then
        rm -f "$jam_file"
      fi

      IFS='|' read -r status rdesc size code_size gas time <<< "$result"
      if [ "$status" = "OK" ]; then
        printf "| %-20s | %10s | %10s | %14s | %10s |\n" "$rdesc" "$size" "$code_size" "$gas" "$time"
      else
        printf "| %-20s | %10s | %10s | %14s | %10s |\n" "$rdesc" "SKIP" "-" "-" "-"
      fi
    done
    echo ""
  fi
}

build_and_benchmark() {
  local label="$1"
  echo "Building $label..." >&2
  cargo build --release --quiet >&2
  rm -rf "$PROJECT_ROOT/tests/build/wasm"
  (cd "$PROJECT_ROOT/tests" && bun build.ts >&2)

  if [ "$NO_OPT" = true ]; then
    local noopt_dir="$PROJECT_ROOT/tests/build/jam-noopt"
    compile_noopt_jams "$noopt_dir"
    JAM_DIR="$noopt_dir"
    COMPILER_JAM="$noopt_dir/anan-as-compiler.jam"
  else
    JAM_DIR="$DEFAULT_JAM_DIR"
    COMPILER_JAM="$DEFAULT_JAM_DIR/anan-as-compiler.jam"
    # `bun build.ts` only knows about the standard fixtures, so compile the
    # populated Polkadot runtimes (if any) into the same JAM dir using the
    # current toolchain. Silently no-ops if WASMs aren't present.
    compile_polkadot_jams "$JAM_DIR"
  fi

  run_benchmarks "$label"
}

compare_branches() {
  local base_branch="$1"
  local current_branch="$2"
  local orig_branch
  orig_branch=$(git rev-parse --abbrev-ref HEAD)
  # Expand orig_branch now; EXIT runs after function locals are out of scope.
  trap "git checkout '$orig_branch' --quiet 2>/dev/null || true" EXIT

  local base_output current_output

  # Benchmark base
  echo "Checking out $base_branch..." >&2
  git checkout "$base_branch" --quiet
  base_output=$(build_and_benchmark "Baseline ($base_branch)")

  # Benchmark current
  echo "Checking out $current_branch..." >&2
  git checkout "$current_branch" --quiet
  current_output=$(build_and_benchmark "Current ($current_branch)")

  # Restore handled by EXIT trap
  echo "$base_output"
  echo "$current_output"

  # Generate comparison table using Python (avoids bash associative array issues)
  generate_comparison "$base_output" "$current_output" "$base_branch" "$current_branch"
}

generate_comparison() {
  local base_output="$1"
  local current_output="$2"
  local base_name="$3"
  local current_name="$4"

  python3 - "$base_name" "$current_name" <<'PYEOF' "$base_output" "$current_output"
import sys, re

base_name = sys.argv[1]
current_name = sys.argv[2]
base_text = sys.argv[3]
current_text = sys.argv[4]

def parse_table(text):
    rows = {}
    mode = None
    for line in text.split('\n'):
        if line.startswith('| Benchmark | WASM Size | JAM Size | Code Size | Gas Used |'):
            mode = "direct"
            continue
        if line.startswith('| Benchmark | JAM Size | Code Size | Outer Gas Used |'):
            mode = "pip"
            continue
        if line.startswith('|---'):
            continue
        if not line.startswith('|'):
            mode = None
            continue
        parts = [p.strip() for p in line.split('|')]
        if mode == "direct" and len(parts) >= 8:
            desc, size, code_size, gas = parts[1], parts[3], parts[4], parts[5]
            if desc and size:
                rows[desc] = (size, code_size, gas)
        elif mode == "pip" and len(parts) >= 7:
            desc, size, code_size, gas = parts[1], parts[2], parts[3], parts[4]
            if desc and size:
                rows[desc] = (size, code_size, gas)
    return rows

base = parse_table(base_text)
current = parse_table(current_text)

def pct(before, after):
    try:
        b, a = int(before), int(after)
        if b == 0: return "-"
        diff = a - b
        p = (diff / b) * 100
        return f"{diff:+d} ({p:+.1f}%)"
    except (ValueError, TypeError):
        return "-"

print(f"## Comparison: {base_name} vs {current_name}")
print()
print("| Benchmark | JAM Size (before) | JAM Size (after) | Size Change | Code (before) | Code (after) | Code Change | Gas (before) | Gas (after) | Gas Change |")
print("|-----------|--------------|-------------|-------------|--------------|-------------|-------------|-------------|------------|------------|")

for desc in base:
    bs, bcs, bg = base[desc]
    cs, ccs, cg = current.get(desc, ("?", "?", "?"))
    sc = pct(bs, cs)
    cc = pct(bcs, ccs)
    gc = pct(bg, cg)
    print(f"| {desc:<20s} | {bs:>12s} | {cs:>12s} | {sc:>14s} | {bcs:>12s} | {ccs:>12s} | {cc:>14s} | {bg:>10s} | {cg:>10s} | {gc:>14s} |")
print()
PYEOF
}

# Parse arguments
BASE_BRANCH=""
CURRENT_BRANCH=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --no-opt) NO_OPT=true; shift ;;
    --base) BASE_BRANCH="$2"; shift 2 ;;
    --current) CURRENT_BRANCH="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 [--no-opt] [--base <branch>] [--current <branch>]"
      echo ""
      echo "Options:"
      echo "  --no-opt              Compile JAMs with all PVM-level optimizations disabled"
      echo "  --base <branch>       Base branch for comparison"
      echo "  --current <branch>    Current branch for comparison"
      echo ""
      echo "Without arguments: build and benchmark current code (optimized)"
      echo "With --no-opt: build and benchmark with PVM optimizations disabled"
      echo "With --base/--current: compare two branches"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [ -n "$BASE_BRANCH" ] && [ -n "$CURRENT_BRANCH" ]; then
  compare_branches "$BASE_BRANCH" "$CURRENT_BRANCH"
elif [ -n "$BASE_BRANCH" ] || [ -n "$CURRENT_BRANCH" ]; then
  echo "Error: must specify both --base and --current, or neither"
  exit 1
else
  if [ "$NO_OPT" = true ]; then
    build_and_benchmark "Current Build (no PVM optimizations)"
  else
    build_and_benchmark "Current Build"
  fi
fi
