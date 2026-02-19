#!/usr/bin/env bash
# Benchmark script: measures JAM file size, gas usage, and execution time.
#
# Usage:
#   ./tests/utils/benchmark.sh [--base <branch>] [--current <branch>]
#
# Without arguments, builds and benchmarks the current code.
# With --base/--current, compares two branches side by side.
#
# Prerequisites: cargo, bun, node, python3 must be in PATH.
# Run from the project root directory.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ANAN_CLI="$PROJECT_ROOT/vendor/anan-as/dist/bin/index.js"
JAM_DIR="$PROJECT_ROOT/tests/build/jam"
GAS_BUDGET=100000000

# Representative benchmarks: (jam_basename, args, description)
BENCHMARKS=(
  "add|0500000007000000|add(5,7)"
  "fibonacci|14000000|fib(20)"
  "factorial|0a000000|factorial(10)"
  "is-prime|19000000|is_prime(25)"
  "as-fibonacci|0a000000|AS fib(10)"
  "as-factorial|07000000|AS 7!"
  "as-gcd|00e10700c8000000|AS gcd(2017,200)"
  "as-decoder-test|00000000|AS decoder"
  "as-array-test|00000000|AS array"
  "anan-as-compiler||AS compiler (size only)"
)

benchmark_one() {
  local jam_file="$1"
  local args="$2"
  local desc="$3"
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
    output=$(node "$ANAN_CLI" run --spi --no-logs --gas=$GAS_BUDGET "$jam_file" "0x$args" 2>&1) || exit_code=$?
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

run_benchmarks() {
  local label="$1"
  echo "## $label"
  echo ""
  echo "| Benchmark | JAM Size | Gas Used | Time (median of 3) |"
  echo "|-----------|----------|----------|-------------------|"

  for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r basename args desc <<< "$entry"
    local jam_file="$JAM_DIR/$basename.jam"
    local result
    result=$(benchmark_one "$jam_file" "$args" "$desc")

    IFS='|' read -r status rdesc size gas time <<< "$result"
    if [ "$status" = "OK" ]; then
      printf "| %-20s | %10s | %10s | %10s |\n" "$rdesc" "$size" "$gas" "$time"
    else
      printf "| %-20s | %10s | %10s | %10s |\n" "$rdesc" "SKIP" "-" "-"
    fi
  done
  echo ""
}

build_and_benchmark() {
  local label="$1"
  echo "Building $label..." >&2
  cargo build --release --quiet >&2
  rm -rf "$PROJECT_ROOT/tests/build/wasm"
  (cd "$PROJECT_ROOT/tests" && bun build.ts >&2)
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
    --base) BASE_BRANCH="$2"; shift 2 ;;
    --current) CURRENT_BRANCH="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 [--base <branch>] [--current <branch>]"
      echo ""
      echo "Without arguments: build and benchmark current code"
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
  build_and_benchmark "Current Build"
fi
