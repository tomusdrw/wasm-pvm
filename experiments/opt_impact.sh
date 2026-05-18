#!/usr/bin/env bash
# opt_impact.sh — per-flag optimization impact measurement.
#
# For every --no-X optimization flag, recompile each input with ONLY that
# flag disabled (all others on) and record JAM size + PVM code size deltas
# vs. the all-on baseline. A flag whose row is all-zero across inputs is
# provably dead in practice.
#
# Usage:
#   ./experiments/opt_impact.sh             # full run (fixtures + polkadot if populated)
#   ./experiments/opt_impact.sh --fixtures  # fixtures only (~15 min)
#   ./experiments/opt_impact.sh --polkadot  # polkadot runtimes only (~90 min)
#   ./experiments/opt_impact.sh --aslan     # as-lan only (~2 min)
#
# Output:
#   experiments/results/baseline.csv   one row per input
#   experiments/results/deltas.csv     one row per (flag, input)
#   experiments/results/summary.md     human-readable matrix
#
# Prerequisites: cargo, bun, node, python3, wasm-tools (for polkadot import maps)

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_PVM="$PROJECT_ROOT/target/release/wasm-pvm"
RESULTS_DIR="$PROJECT_ROOT/experiments/results"
TMP_JAM_DIR="$(mktemp -d -t wasm-pvm-optexp-XXXXXX)"
trap 'rm -rf "$TMP_JAM_DIR"' EXIT

# Per-input compile timeout (seconds). Polkadot runtimes can be slow; smaller
# fixtures finish in <1s. macOS users need gtimeout (coreutils) or this stays
# unbounded.
COMPILE_TIMEOUT="${COMPILE_TIMEOUT:-300}"

PHASE_FIXTURES=1
PHASE_POLKADOT=1
PHASE_ASLAN=1
case "${1:-}" in
  --fixtures)  PHASE_POLKADOT=0; PHASE_ASLAN=0 ;;
  --polkadot)  PHASE_FIXTURES=0; PHASE_ASLAN=0 ;;
  --aslan)     PHASE_FIXTURES=0; PHASE_POLKADOT=0 ;;
  --help|-h)   sed -n '2,21p' "$0"; exit 0 ;;
  "") ;;
  *) echo "Unknown arg: $1 (try --help)" >&2; exit 1 ;;
esac

# -----------------------------------------------------------------------------
# Optimization flag list — each entry toggles ONE optimization off.
# -----------------------------------------------------------------------------
OPT_FLAGS=(
  --no-llvm-passes
  --no-peephole
  --no-register-cache
  --no-icmp-fusion
  --no-shrink-wrap
  --no-dead-store-elim
  --no-const-prop
  --no-inline
  --no-cross-block-cache
  --no-register-alloc
  --no-dead-function-elim
  --no-fallthrough-jumps
  --no-aggressive-regalloc
  --no-scratch-reg-alloc
  --no-caller-saved-alloc
  --no-lazy-spill
  --no-libcall-recognition
  --no-mergefunc
)

# -----------------------------------------------------------------------------
# Inputs: "name|source|imports|adapter"
# imports/adapter may be empty. All paths are relative to PROJECT_ROOT.
# Mirrors the BENCHMARKS array in tests/utils/benchmark.sh plus extras
# (aslan-debug, every hand-crafted WAT, every AS-built WASM).
# -----------------------------------------------------------------------------
FIXTURE_INPUTS=(
  # Hand-crafted WAT (curated benchmark set; rest auto-discovered below)
  "add|tests/fixtures/wat/add.jam.wat||"
  "fibonacci|tests/fixtures/wat/fibonacci.jam.wat||"
  "factorial|tests/fixtures/wat/factorial.jam.wat||"
  "is-prime|tests/fixtures/wat/is-prime.jam.wat||"
  "regalloc-two-loops|tests/fixtures/wat/regalloc-two-loops.jam.wat||"
  "blake2b|tests/fixtures/wat/blake2b.jam.wat||"
  "sha512|tests/fixtures/wat/sha512.jam.wat||"
  "u128-mul-bench|tests/fixtures/wat/u128-mul-bench.jam.wat||"
  "u128-div-bench|tests/fixtures/wat/u128-div-bench.jam.wat||"
  "u128-div-bench-slow|tests/fixtures/wat/u128-div-bench-slow.jam.wat||"
  "host-call-log|tests/fixtures/wat/host-call-log.jam.wat||"
  "memory-copy-word|tests/fixtures/wat/memory-copy-word.jam.wat||"
  "memory-copy-overlap|tests/fixtures/wat/memory-copy-overlap.jam.wat||"
  # AS-built WASM (require `cd tests && bun build.ts` first)
  "as-fibonacci|tests/build/wasm/fibonacci.wasm||"
  "as-factorial|tests/build/wasm/factorial.wasm||"
  "as-gcd|tests/build/wasm/gcd.wasm||"
  "as-decoder-test|tests/build/wasm/decoder-test.wasm||"
  "as-array-test|tests/build/wasm/array-test.wasm||"
  "as-alloc-test|tests/build/wasm/alloc-test.wasm||"
  "as-string-test|tests/build/wasm/string-test.wasm||"
)

ASLAN_INPUTS=(
  "aslan-fib|tests/fixtures/wat/aslan-fib.jam.wat||tests/fixtures/imports/aslan-fib.adapter.wat"
  "aslan-ecalli|tests/fixtures/wat/aslan-ecalli.jam.wat|tests/fixtures/imports/aslan-ecalli.imports|"
  "aslan-debug|tests/fixtures/external/aslan-debug.wasm||tests/fixtures/imports/aslan-fib.adapter.wat"
  # anan-as is the PVM-in-PVM interpreter — large hand-written AS service,
  # the closest thing we have to a representative "as-lan service".
  "anan-as-compiler|vendor/anan-as/dist/build/compiler.wasm|tests/fixtures/imports/anan-as-compiler.imports|tests/fixtures/imports/anan-as-compiler.adapter.wat"
)

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

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------
timeout_bin() {
  if command -v timeout >/dev/null 2>&1; then echo "timeout"; return; fi
  if command -v gtimeout >/dev/null 2>&1; then echo "gtimeout"; return; fi
  echo ""
}
TIMEOUT_BIN="$(timeout_bin)"

filesize() {
  if stat -f%z "$1" >/dev/null 2>&1; then stat -f%z "$1"; else stat -c%s "$1"; fi
}

# Extract PVM code size (instruction bytes only) from a JAM file.
# Same parser as tests/utils/benchmark.sh.
jam_code_size() {
  python3 - "$1" <<'PY'
import sys
try:
    data = open(sys.argv[1], 'rb').read()
    if len(data) < 15:
        print('-'); sys.exit(0)
    off = 0
    def vi(d, o):
        b = d[o]
        if b < 0x80: return b, 1
        elif b < 0xc0: return ((b-0x80)<<8)|d[o+1], 2
        elif b < 0xe0: return ((b-0xc0)<<16)|d[o+1]|(d[o+2]<<8), 3
        else: return ((b-0xe0)<<24)|d[o+1]|(d[o+2]<<8)|(d[o+3]<<16), 4
    meta_len, n = vi(data, off); off += n + meta_len
    ro_len = data[off]|(data[off+1]<<8)|(data[off+2]<<16); off += 3
    rw_len = data[off]|(data[off+1]<<8)|(data[off+2]<<16); off += 3 + 2 + 3
    off += ro_len + rw_len + 4
    jt_len, n = vi(data, off); off += n + 1
    code_len, _ = vi(data, off)
    print(code_len)
except Exception:
    print('-')
PY
}

# Compile one input. Args: source imports adapter output_path [extra_flags...]
compile_one() {
  local source="$1" imports="$2" adapter="$3" output="$4"
  shift 4
  # `${extra[@]}` on an empty array trips `set -u` in older bashes; guard with
  # explicit length check before expanding.
  local extra=()
  if [ "$#" -gt 0 ]; then
    extra=("$@")
  fi
  local cmd=("$WASM_PVM" compile "$PROJECT_ROOT/$source" -o "$output")
  if [ -n "$imports" ] && [ -f "$PROJECT_ROOT/$imports" ]; then
    cmd+=(--imports "$PROJECT_ROOT/$imports")
  fi
  if [ -n "$adapter" ] && [ -f "$PROJECT_ROOT/$adapter" ]; then
    cmd+=(--adapter "$PROJECT_ROOT/$adapter")
  fi
  if [ "${#extra[@]}" -gt 0 ]; then
    cmd+=("${extra[@]}")
  fi
  if [ -n "$TIMEOUT_BIN" ]; then
    "$TIMEOUT_BIN" "$COMPILE_TIMEOUT" "${cmd[@]}" >/dev/null 2>&1
  else
    "${cmd[@]}" >/dev/null 2>&1
  fi
}

# -----------------------------------------------------------------------------
# Discover Polkadot inputs (only if WASMs are populated).
# -----------------------------------------------------------------------------
build_polkadot_inputs() {
  local result=()
  local wasm_dir="$PROJECT_ROOT/examples/polkadot/wasm"
  local imports_dir="$PROJECT_ROOT/examples/polkadot/imports"
  for rt in "${POLKADOT_RUNTIMES[@]}"; do
    local wasm="$wasm_dir/$rt.wasm"
    [ -f "$wasm" ] || continue
    local imports="$imports_dir/$rt.imports"
    # Auto-generate trap-all import map if missing
    if [ ! -f "$imports" ]; then
      if command -v wasm-tools >/dev/null 2>&1; then
        mkdir -p "$imports_dir"
        wasm-tools print "$wasm" 2>/dev/null \
          | awk -F\" '/^[[:space:]]*\(import "/ { print $4 }' \
          | awk '!seen[$0]++ { print $0 " = trap" }' \
          > "$imports"
      else
        echo "  WARN: missing $imports and wasm-tools not installed — skipping $rt" >&2
        continue
      fi
    fi
    result+=("polkadot-$rt|examples/polkadot/wasm/$rt.wasm|examples/polkadot/imports/$rt.imports|")
  done
  if [ "${#result[@]}" -gt 0 ]; then
    printf '%s\n' "${result[@]}"
  fi
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
echo "==> Building release binary"
(cd "$PROJECT_ROOT" && cargo build --release -p wasm-pvm-cli >/dev/null 2>&1) || {
  echo "ERROR: cargo build failed" >&2; exit 1;
}
[ -x "$WASM_PVM" ] || { echo "ERROR: missing $WASM_PVM" >&2; exit 1; }

# Build test fixtures so AS WASMs exist.
if [ "$PHASE_FIXTURES" = "1" ] || [ "$PHASE_ASLAN" = "1" ]; then
  echo "==> Building test fixtures (AS->WASM via bun)"
  if [ -d "$PROJECT_ROOT/tests" ]; then
    (cd "$PROJECT_ROOT/tests" && bun build.ts >/dev/null 2>&1) || {
      echo "  WARN: bun build.ts failed — AS-built inputs may be missing" >&2
    }
  fi
fi

mkdir -p "$RESULTS_DIR"

# Polkadot: trap-floats is mandatory for these to compile at all.
extra_for_input() {
  case "$1" in
    polkadot-*) echo "--trap-floats" ;;
    *) echo "" ;;
  esac
}

# Assemble the full input list.
INPUTS=()
if [ "$PHASE_FIXTURES" = "1" ]; then
  INPUTS+=("${FIXTURE_INPUTS[@]}")
fi
if [ "$PHASE_ASLAN" = "1" ]; then
  INPUTS+=("${ASLAN_INPUTS[@]}")
fi
if [ "$PHASE_POLKADOT" = "1" ]; then
  while IFS= read -r line; do
    [ -n "$line" ] && INPUTS+=("$line")
  done < <(build_polkadot_inputs)
fi

# Filter out inputs whose source doesn't exist (lets users run with partial setups).
KEPT_INPUTS=()
for entry in "${INPUTS[@]}"; do
  IFS='|' read -r name source imports adapter <<< "$entry"
  if [ -f "$PROJECT_ROOT/$source" ]; then
    KEPT_INPUTS+=("$entry")
  else
    echo "  skip $name: source $source not found" >&2
  fi
done

echo "==> ${#KEPT_INPUTS[@]} input(s) to process"
echo "==> ${#OPT_FLAGS[@]} optimization flag(s) to toggle"

BASELINE_CSV="$RESULTS_DIR/baseline.csv"
DELTAS_CSV="$RESULTS_DIR/deltas.csv"
SUMMARY_MD="$RESULTS_DIR/summary.md"

echo "name,source,jam_size,code_size,compile_ok" > "$BASELINE_CSV"
echo "flag,input,baseline_jam,delta_jam,baseline_code,delta_code,compile_ok" > "$DELTAS_CSV"

# -----------------------------------------------------------------------------
# Phase A: baseline compile for every input
# -----------------------------------------------------------------------------
echo "==> Phase A: baseline (all opts on)"
declare -a BL_JAM BL_CODE BL_OK
for i in "${!KEPT_INPUTS[@]}"; do
  entry="${KEPT_INPUTS[$i]}"
  IFS='|' read -r name source imports adapter <<< "$entry"
  out="$TMP_JAM_DIR/baseline-$name.jam"
  extra=""
  extra="$(extra_for_input "$name")"
  printf "  [%2d/%d] %s " "$((i+1))" "${#KEPT_INPUTS[@]}" "$name"
  ok=1
  if [ -n "$extra" ]; then
    compile_one "$source" "$imports" "$adapter" "$out" "$extra" || ok=0
  else
    compile_one "$source" "$imports" "$adapter" "$out" || ok=0
  fi
  if [ "$ok" = "1" ] && [ -f "$out" ]; then
    sz=$(filesize "$out")
    cs=$(jam_code_size "$out")
    BL_JAM[$i]=$sz; BL_CODE[$i]=$cs; BL_OK[$i]=1
    printf "  jam=%s code=%s\n" "$sz" "$cs"
  else
    BL_JAM[$i]=0; BL_CODE[$i]=0; BL_OK[$i]=0
    printf "  FAILED\n"
  fi
  echo "$name,$source,${BL_JAM[$i]},${BL_CODE[$i]},${BL_OK[$i]}" >> "$BASELINE_CSV"
done

# -----------------------------------------------------------------------------
# Phase B: for each opt flag, compile each input with ONLY that flag off
# -----------------------------------------------------------------------------
echo "==> Phase B: per-flag deltas"
for flag in "${OPT_FLAGS[@]}"; do
  echo "  --- $flag ---"
  for i in "${!KEPT_INPUTS[@]}"; do
    entry="${KEPT_INPUTS[$i]}"
    IFS='|' read -r name source imports adapter <<< "$entry"
    if [ "${BL_OK[$i]}" != "1" ]; then
      echo "$flag,$name,0,NA,0,NA,baseline_failed" >> "$DELTAS_CSV"
      continue
    fi
    out="$TMP_JAM_DIR/$(echo "$flag" | tr / _)-$name.jam"
    extra="$(extra_for_input "$name")"
    flags=("$flag")
    [ -n "$extra" ] && flags+=("$extra")
    ok=1
    compile_one "$source" "$imports" "$adapter" "$out" "${flags[@]}" || ok=0
    if [ "$ok" = "1" ] && [ -f "$out" ]; then
      sz=$(filesize "$out")
      cs=$(jam_code_size "$out")
      djam=$((sz - ${BL_JAM[$i]}))
      dcode="NA"
      if [ "$cs" != "-" ] && [ "${BL_CODE[$i]}" != "-" ]; then
        dcode=$((cs - ${BL_CODE[$i]}))
      fi
      echo "$flag,$name,${BL_JAM[$i]},$djam,${BL_CODE[$i]},$dcode,1" >> "$DELTAS_CSV"
      [ "$djam" != "0" ] && printf "    %s: jam=%+d code=%s\n" "$name" "$djam" "$dcode"
    else
      echo "$flag,$name,${BL_JAM[$i]},NA,${BL_CODE[$i]},NA,0" >> "$DELTAS_CSV"
      echo "    $name: FAILED to compile with $flag"
    fi
    rm -f "$out"
  done
done

# -----------------------------------------------------------------------------
# Phase C: summary matrix
# -----------------------------------------------------------------------------
echo "==> Phase C: summarize"
python3 - "$DELTAS_CSV" "$SUMMARY_MD" <<'PY'
import csv, sys
from collections import defaultdict
deltas_path, out_path = sys.argv[1], sys.argv[2]

# rows[flag] = list of (input, djam, dcode, ok)
rows = defaultdict(list)
inputs_order = []
seen = set()
with open(deltas_path) as f:
    for r in csv.DictReader(f):
        flag = r['flag']
        inp = r['input']
        if inp not in seen:
            inputs_order.append(inp); seen.add(inp)
        djam = r['delta_jam']
        dcode = r['delta_code']
        ok = r['compile_ok']
        rows[flag].append((inp, djam, dcode, ok))

with open(out_path, 'w') as f:
    f.write("# Optimization Impact Summary\n\n")
    f.write(f"Inputs measured: {len(inputs_order)}\n\n")

    # Per-flag aggregate.
    f.write("## Per-flag aggregate (negative delta = optimization saves bytes when on)\n\n")
    f.write("| Flag | Inputs Changed | Total JAM Delta | Total Code Delta | Inputs Failed |\n")
    f.write("|------|---------------:|----------------:|-----------------:|--------------:|\n")
    summary_rows = []
    for flag in rows:
        changed = sum(1 for _, dj, _, ok in rows[flag] if ok == '1' and dj not in ('0', 'NA'))
        total_jam = sum(int(dj) for _, dj, _, ok in rows[flag] if ok == '1' and dj not in ('NA',))
        total_code = sum(int(dc) for _, _, dc, ok in rows[flag] if ok == '1' and dc not in ('NA',))
        failed = sum(1 for _, _, _, ok in rows[flag] if ok != '1')
        summary_rows.append((flag, changed, total_jam, total_code, failed))
    # Sort by inputs_changed ASC so dead flags float to the top.
    summary_rows.sort(key=lambda r: (r[1], r[0]))
    for flag, changed, tj, tc, fail in summary_rows:
        f.write(f"| `{flag}` | {changed} | {tj:+d} | {tc:+d} | {fail} |\n")

    # Dead flag callout.
    dead = [r for r in summary_rows if r[1] == 0 and r[4] == 0]
    if dead:
        f.write("\n### Provably dead on this input set\n\n")
        for flag, *_ in dead:
            f.write(f"- `{flag}` — zero impact on any input, no compile failures\n")
    else:
        f.write("\n_All flags affect at least one input._\n")

    # Per-input matrix.
    f.write("\n## Per-input JAM delta matrix\n\n")
    f.write("| Flag |")
    for inp in inputs_order: f.write(f" {inp} |")
    f.write("\n|------|")
    for _ in inputs_order: f.write("---:|")
    f.write("\n")
    by_flag = {flag: {inp: dj for inp, dj, _, _ in rs} for flag, rs in rows.items()}
    for flag, *_ in summary_rows:
        f.write(f"| `{flag}` |")
        for inp in inputs_order:
            dj = by_flag[flag].get(inp, '-')
            f.write(f" {dj} |")
        f.write("\n")

print(f"Summary written to {out_path}", file=sys.stderr)
PY

echo
echo "==> Done."
echo "    Baseline:    $BASELINE_CSV"
echo "    Per-flag:    $DELTAS_CSV"
echo "    Summary:     $SUMMARY_MD"
echo
echo "Paste $SUMMARY_MD (and optionally $DELTAS_CSV if interesting deltas warrant a deeper look) back to the assistant."
