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

NO_OPT_FLAGS=(
  --no-peephole
  --no-register-cache
  --no-icmp-fusion
  --no-shrink-wrap
  --no-dead-store-elim
  --no-const-prop
  --no-cross-block-cache
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
  "as-factorial|07000000|0|AS 7!|wasm:tests/build/wasm/factorial.wasm"
  "as-gcd|00e10700c8000000|0|AS gcd(2017,200)|wasm:tests/build/wasm/gcd.wasm"
  "as-decoder-test|00000000|0|AS decoder|wasm:tests/build/wasm/decoder-test.wasm"
  "as-array-test|00000000|0|AS array|wasm:tests/build/wasm/array-test.wasm"
  "anan-as-compiler||0|anan-as PVM interpreter|wasm:vendor/anan-as/dist/build/compiler.wasm"
)

# Return "imports_path|adapter_path" for benchmarks that need them, or empty.
benchmark_imports_for() {
  local basename="$1"
  case "$basename" in
    anan-as-compiler)
      echo "tests/fixtures/imports/anan-as-compiler.imports|tests/fixtures/imports/anan-as-compiler.adapter.wat"
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
  "as-fibonacci|0a000000|0|PiP AS fib(10)"
  "EXT:tests/fixtures/external/jam-sdk-fib.jam|0100000002000000030000000000000000000000|5|PiP JAM-SDK fib(10)"
)

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
  mkdir -p "$noopt_dir"

  echo "Recompiling benchmarks without PVM optimizations..." >&2
  for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r basename _args _pc _desc wasm_src <<< "$entry"
    # Skip external JAM files (can't recompile) and entries without sources
    if [ -z "$wasm_src" ] || [[ "$basename" == EXT:* ]]; then
      continue
    fi
    local output="$noopt_dir/$basename.jam"
    compile_benchmark "$wasm_src" "$output" "$basename" "${NO_OPT_FLAGS[@]}" >&2 || true
  done
}

benchmark_one() {
  local jam_file="$1"
  local args="$2"
  local pc="$3"
  local desc="$4"
  local size gas_used time_ms

  if [ ! -f "$jam_file" ]; then
    echo "SKIP|$desc|missing"
    return
  fi

  size=$(wc -c < "$jam_file" | tr -d ' ')

  if [ -z "$args" ]; then
    # Size-only benchmark (e.g. large compiler)
    echo "OK|$desc|$size|-|-"
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

  echo "OK|$desc|$size|$gas_used|${time_ms}ms"
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
  local size gas_used time_ms

  if [ ! -f "$jam_file" ]; then
    echo "SKIP|$desc|missing"
    return
  fi

  if [ ! -f "$COMPILER_JAM" ]; then
    echo "SKIP|$desc|no compiler.jam"
    return
  fi

  size=$(wc -c < "$jam_file" | tr -d ' ')

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

  echo "OK|$desc|$size|$gas_used|${time_ms}ms"
}

run_benchmarks() {
  local label="$1"
  echo "## $label"
  echo ""
  echo "| Benchmark | WASM Size | JAM Size | Gas Used | Time (median of 3) |"
  echo "|-----------|-----------|----------|----------|-------------------|"

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

    IFS='|' read -r status rdesc size gas time <<< "$result"
    if [ "$status" = "OK" ]; then
      printf "| %-20s | %10s | %10s | %10s | %10s |\n" "$rdesc" "$wsize" "$size" "$gas" "$time"
    else
      printf "| %-20s | %10s | %10s | %10s | %10s |\n" "$rdesc" "$wsize" "SKIP" "-" "-"
    fi
  done
  echo ""

  # PVM-in-PVM benchmarks
  if [ -f "$COMPILER_JAM" ]; then
    echo "### PVM-in-PVM"
    echo ""
    echo "| Benchmark | JAM Size | Outer Gas Used | Time (median of 3) |"
    echo "|-----------|----------|----------------|-------------------|"

    for entry in "${PVM_IN_PVM_BENCHMARKS[@]}"; do
      IFS='|' read -r basename args pc desc <<< "$entry"
      pc="${pc:-0}"
      local jam_file cleanup_jam=false
      if [ "$basename" = "TRAP" ]; then
        # Synthetic 1-byte TRAP program (opcode 0x00)
        jam_file=$(mktemp "${TMPDIR:-/tmp}/pvm-bench-trap-XXXXXX")
        printf '\x00' > "$jam_file"
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

      IFS='|' read -r status rdesc size gas time <<< "$result"
      if [ "$status" = "OK" ]; then
        printf "| %-20s | %10s | %14s | %10s |\n" "$rdesc" "$size" "$gas" "$time"
      else
        printf "| %-20s | %10s | %14s | %10s |\n" "$rdesc" "SKIP" "-" "-"
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
  fi

  run_benchmarks "$label"
}

compare_branches() {
  local base_branch="$1"
  local current_branch="$2"
  local orig_branch
  orig_branch=$(git rev-parse --abbrev-ref HEAD)
  trap 'git checkout "$orig_branch" --quiet 2>/dev/null || true' EXIT

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
    for line in text.split('\n'):
        if not line.startswith('|') or 'Benchmark' in line or '---' in line:
            continue
        parts = [p.strip() for p in line.split('|')]
        if len(parts) >= 6:
            desc, size, gas = parts[1], parts[2], parts[3]
            if desc and size:
                rows[desc] = (size, gas)
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
print("| Benchmark | Size (before) | Size (after) | Size Change | Gas (before) | Gas (after) | Gas Change |")
print("|-----------|--------------|-------------|-------------|-------------|------------|------------|")

for desc in base:
    bs, bg = base[desc]
    cs, cg = current.get(desc, ("?", "?"))
    sc = pct(bs, cs)
    gc = pct(bg, cg)
    print(f"| {desc:<20s} | {bs:>12s} | {cs:>12s} | {sc:>14s} | {bg:>10s} | {cg:>10s} | {gc:>14s} |")
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
