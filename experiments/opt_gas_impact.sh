#!/usr/bin/env bash
# opt_gas_impact.sh — per-flag gas + JAM-size impact on runnable benchmarks.
#
# Companion to opt_impact.sh. For each --no-X optimization flag, recompiles
# every runnable fixture (every BENCHMARKS entry in tests/utils/benchmark.sh
# with non-empty args) with ONLY that flag disabled, then runs the resulting
# JAM via anan-as and parses `Gas remaining:` from stdout. Records both
# delta_jam and delta_gas vs. the all-on baseline so cells where size and gas
# move in opposite directions stand out.
#
# Gas is deterministic for a given (program, args), so 1 run per cell is
# enough — no median smoothing needed (unlike benchmark.sh which measures
# wall time).
#
# Usage:
#   ./experiments/opt_gas_impact.sh
#
# Output:
#   experiments/results/gas_baseline.csv   one row per runnable input
#   experiments/results/gas_deltas.csv     one row per (flag, input)
#   experiments/results/gas_summary.md     human-readable per-flag report
#
# Prerequisites: cargo, bun, node, python3.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_PVM="$PROJECT_ROOT/target/release/wasm-pvm"
ANAN_CLI="$PROJECT_ROOT/vendor/anan-as/dist/bin/index.js"
RESULTS_DIR="$PROJECT_ROOT/experiments/results"
TMP_JAM_DIR="$(mktemp -d -t wasm-pvm-gasexp-XXXXXX)"
trap 'rm -rf "$TMP_JAM_DIR"' EXIT

GAS_BUDGET=100000000
COMPILE_TIMEOUT="${COMPILE_TIMEOUT:-300}"
RUN_TIMEOUT="${RUN_TIMEOUT:-60}"

# -----------------------------------------------------------------------------
# Flag list — mirrors opt_impact.sh, sans --debug-skip-llvm-passes (known to
# break every input because the PVM backend requires mem2reg).
# -----------------------------------------------------------------------------
OPT_FLAGS=(
  --no-peephole
  --no-register-cache
  --no-icmp-fusion
  --no-shrink-wrap
  --no-dead-store-elim
  --no-const-prop
  --no-inline
  --no-cross-block-cache
  --no-register-alloc
  --no-fallthrough-jumps
  --no-aggressive-regalloc
  --no-scratch-reg-alloc
  --no-caller-saved-alloc
  --no-lazy-spill
  --no-libcall-recognition
  --no-mergefunc
)

# -----------------------------------------------------------------------------
# Inputs: "name|source|imports|adapter|args|pc"
# Mirrors the runnable subset of BENCHMARKS in tests/utils/benchmark.sh
# (every entry with non-empty `args`, excluding EXT: external JAM files).
# -----------------------------------------------------------------------------
SHA512_240A="616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161616161"

RUNNABLE_INPUTS=(
  "add|tests/fixtures/wat/add.jam.wat|||0500000007000000|0"
  "fibonacci|tests/fixtures/wat/fibonacci.jam.wat|||14000000|0"
  "factorial|tests/fixtures/wat/factorial.jam.wat|||0a000000|0"
  "is-prime|tests/fixtures/wat/is-prime.jam.wat|||19000000|0"
  "regalloc-two-loops|tests/fixtures/wat/regalloc-two-loops.jam.wat|||f4010000|0"
  "blake2b|tests/fixtures/wat/blake2b.jam.wat|||2000000000000000616263|0"
  "sha512|tests/fixtures/wat/sha512.jam.wat|||616263|0"
  "sha512-240a|tests/fixtures/wat/sha512.jam.wat|||${SHA512_240A}|0"
  "u128-mul-bench|tests/fixtures/wat/u128-mul-bench.jam.wat|||e8030000|0"
  "u128-div-bench|tests/fixtures/wat/u128-div-bench.jam.wat|||e8030000|0"
  "u128-div-bench-slow|tests/fixtures/wat/u128-div-bench-slow.jam.wat|||e8030000|0"
  "host-call-log|tests/fixtures/wat/host-call-log.jam.wat|||00000000|0"
  "as-fibonacci|tests/build/wasm/fibonacci.wasm|||0a000000|0"
  "as-factorial|tests/build/wasm/factorial.wasm|||07000000|0"
  "as-gcd|tests/build/wasm/gcd.wasm|||00e10700c8000000|0"
  "as-decoder-test|tests/build/wasm/decoder-test.wasm|||00000000|0"
  "as-array-test|tests/build/wasm/array-test.wasm|||00000000|0"
  "aslan-fib|tests/fixtures/wat/aslan-fib.jam.wat||tests/fixtures/imports/aslan-fib.adapter.wat|2a0000|5"
  "aslan-ecalli|tests/fixtures/wat/aslan-ecalli.jam.wat|tests/fixtures/imports/aslan-ecalli.imports||00|0"
)

# -----------------------------------------------------------------------------
# Helpers (parallel to opt_impact.sh; duplicated to keep the script self-
# contained — benchmark.sh and opt_impact.sh have differing argument-parsing
# epilogues that make `source`ing them error-prone).
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

# Compile one input. Args: source imports adapter output_path [extra_flags...]
compile_one() {
  local source="$1" imports="$2" adapter="$3" output="$4"
  shift 4
  local extra=()
  if [ "$#" -gt 0 ]; then extra=("$@"); fi
  local cmd=("$WASM_PVM" compile "$PROJECT_ROOT/$source" -o "$output")
  if [ -n "$imports" ] && [ -f "$PROJECT_ROOT/$imports" ]; then
    cmd+=(--imports "$PROJECT_ROOT/$imports")
  fi
  if [ -n "$adapter" ] && [ -f "$PROJECT_ROOT/$adapter" ]; then
    cmd+=(--adapter "$PROJECT_ROOT/$adapter")
  fi
  if [ "${#extra[@]}" -gt 0 ]; then cmd+=("${extra[@]}"); fi
  if [ -n "$TIMEOUT_BIN" ]; then
    "$TIMEOUT_BIN" "$COMPILE_TIMEOUT" "${cmd[@]}" >/dev/null 2>&1
  else
    "${cmd[@]}" >/dev/null 2>&1
  fi
}

# Run a JAM under anan-as and return "status|gas_used".
#   status ∈ ok | failed
#   gas_used = (GAS_BUDGET - parsed `Gas remaining: N`); NA if not parsed.
# Args: jam_path args_hex pc
run_one_gas() {
  local jam="$1" args="$2" pc="$3"
  local output exit_code=0
  local node_cmd=(node "$ANAN_CLI" run --spi --no-logs --gas="$GAS_BUDGET" --pc="$pc" "$jam" "0x$args")
  if [ -n "$TIMEOUT_BIN" ]; then
    output=$("$TIMEOUT_BIN" "$RUN_TIMEOUT" "${node_cmd[@]}" 2>&1) || exit_code=$?
  else
    output=$("${node_cmd[@]}" 2>&1) || exit_code=$?
  fi
  local gas_remaining
  gas_remaining=$(echo "$output" | grep -ao 'Gas remaining: [0-9]*' | grep -o '[0-9]*' | head -n1 || echo "")
  if [ -z "$gas_remaining" ]; then
    echo "failed|NA"
    return
  fi
  local gas_used=$((GAS_BUDGET - gas_remaining))
  # Treat any non-zero exit as a "failed" run only if it's not a normal halt.
  # anan-as prints `Gas remaining:` for normal halt; if exit_code != 0 but the
  # line is present, treat as a trap (still record gas so caller can decide).
  if [ "$exit_code" -ne 0 ]; then
    echo "trap|$gas_used"
  else
    echo "ok|$gas_used"
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

# Build test fixtures so AS WASMs exist. `bun build.ts` skips AS->WASM when
# the .wasm already exists; wipe to ensure we measure the latest source.
echo "==> Building test fixtures (AS->WASM via bun, after cache wipe)"
if [ -d "$PROJECT_ROOT/tests" ]; then
  rm -f "$PROJECT_ROOT/tests/build/wasm/"*.wasm
  (cd "$PROJECT_ROOT/tests" && bun build.ts >/dev/null 2>&1) || {
    echo "  WARN: bun build.ts failed — AS-built inputs may be missing" >&2
  }
fi

[ -f "$ANAN_CLI" ] || { echo "ERROR: missing $ANAN_CLI (run vendor/anan-as build first)" >&2; exit 1; }

mkdir -p "$RESULTS_DIR"

# Filter out inputs whose source doesn't exist (lets users run partial setups).
KEPT_INPUTS=()
for entry in "${RUNNABLE_INPUTS[@]}"; do
  IFS='|' read -r name source imports adapter args pc <<< "$entry"
  if [ -f "$PROJECT_ROOT/$source" ]; then
    KEPT_INPUTS+=("$entry")
  else
    echo "  skip $name: source $source not found" >&2
  fi
done

echo "==> ${#KEPT_INPUTS[@]} runnable input(s) to process"
echo "==> ${#OPT_FLAGS[@]} optimization flag(s) to toggle"

BASELINE_CSV="$RESULTS_DIR/gas_baseline.csv"
DELTAS_CSV="$RESULTS_DIR/gas_deltas.csv"
SUMMARY_MD="$RESULTS_DIR/gas_summary.md"

echo "name,source,jam_size,gas_used,run_status,compile_ok" > "$BASELINE_CSV"
echo "flag,input,baseline_jam,delta_jam,baseline_gas,delta_gas,compile_ok,run_status" > "$DELTAS_CSV"

# -----------------------------------------------------------------------------
# Phase A: baseline compile + run for every input.
# -----------------------------------------------------------------------------
echo "==> Phase A: baseline (all opts on)"
declare -a BL_JAM BL_GAS BL_OK BL_RUN
for i in "${!KEPT_INPUTS[@]}"; do
  entry="${KEPT_INPUTS[$i]}"
  IFS='|' read -r name source imports adapter args pc <<< "$entry"
  out="$TMP_JAM_DIR/baseline-$name.jam"
  printf "  [%2d/%d] %s " "$((i+1))" "${#KEPT_INPUTS[@]}" "$name"
  ok=1
  compile_one "$source" "$imports" "$adapter" "$out" || ok=0
  if [ "$ok" = "1" ] && [ -f "$out" ]; then
    sz=$(filesize "$out")
    res=$(run_one_gas "$out" "$args" "$pc")
    IFS='|' read -r status gas_used <<< "$res"
    BL_JAM[$i]=$sz
    BL_GAS[$i]=$gas_used
    BL_OK[$i]=1
    BL_RUN[$i]="$status"
    printf "  jam=%s gas=%s status=%s\n" "$sz" "$gas_used" "$status"
  else
    BL_JAM[$i]=0; BL_GAS[$i]="NA"; BL_OK[$i]=0; BL_RUN[$i]="compile_failed"
    printf "  COMPILE FAILED\n"
  fi
  echo "$name,$source,${BL_JAM[$i]},${BL_GAS[$i]},${BL_RUN[$i]},${BL_OK[$i]}" >> "$BASELINE_CSV"
done

# -----------------------------------------------------------------------------
# Phase B: per-flag deltas.
# -----------------------------------------------------------------------------
echo "==> Phase B: per-flag deltas"
for flag in "${OPT_FLAGS[@]}"; do
  echo "  --- $flag ---"
  for i in "${!KEPT_INPUTS[@]}"; do
    entry="${KEPT_INPUTS[$i]}"
    IFS='|' read -r name source imports adapter args pc <<< "$entry"
    if [ "${BL_OK[$i]}" != "1" ]; then
      echo "$flag,$name,0,NA,NA,NA,0,baseline_failed" >> "$DELTAS_CSV"
      continue
    fi
    out="$TMP_JAM_DIR/$(echo "$flag" | tr / _)-$name.jam"
    ok=1
    compile_one "$source" "$imports" "$adapter" "$out" "$flag" || ok=0
    if [ "$ok" = "1" ] && [ -f "$out" ]; then
      sz=$(filesize "$out")
      djam=$((sz - ${BL_JAM[$i]}))
      res=$(run_one_gas "$out" "$args" "$pc")
      IFS='|' read -r status gas_used <<< "$res"
      dgas="NA"
      if [ "$gas_used" != "NA" ] && [ "${BL_GAS[$i]}" != "NA" ]; then
        dgas=$((gas_used - ${BL_GAS[$i]}))
      fi
      echo "$flag,$name,${BL_JAM[$i]},$djam,${BL_GAS[$i]},$dgas,1,$status" >> "$DELTAS_CSV"
      if [ "$djam" != "0" ] || { [ "$dgas" != "NA" ] && [ "$dgas" != "0" ]; }; then
        printf "    %s: jam=%+d gas=%s status=%s\n" "$name" "$djam" "$dgas" "$status"
      fi
    else
      echo "$flag,$name,${BL_JAM[$i]},NA,${BL_GAS[$i]},NA,0,compile_failed" >> "$DELTAS_CSV"
      echo "    $name: FAILED to compile with $flag"
    fi
    rm -f "$out"
  done
done

# -----------------------------------------------------------------------------
# Phase C: summary.
# -----------------------------------------------------------------------------
echo "==> Phase C: summarize"
python3 - "$BASELINE_CSV" "$DELTAS_CSV" "$SUMMARY_MD" <<'PY'
import csv, sys
from collections import defaultdict

baseline_path, deltas_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]

# Load baseline so the summary can show baseline gas/jam per fixture.
baseline = {}
with open(baseline_path) as f:
    for r in csv.DictReader(f):
        baseline[r['name']] = r

# rows[flag] = list of dict with delta_jam, delta_gas, etc.
rows = defaultdict(list)
inputs_order = []
seen = set()
with open(deltas_path) as f:
    for r in csv.DictReader(f):
        flag = r['flag']
        inp = r['input']
        if inp not in seen:
            inputs_order.append(inp); seen.add(inp)
        rows[flag].append(r)

def to_int(s):
    try:
        return int(s)
    except (TypeError, ValueError):
        return None

GAS_BUDGET = 100000000  # keep in sync with the bash-level GAS_BUDGET above.

# Fixtures whose baseline gas equals the budget exhausted the gas budget under
# baseline. Their delta_gas across variants is structurally pinned at 0 (every
# variant also OOMs) and not interpretable as an optimization-quality signal.
oom_inputs = {
    name for name, r in baseline.items()
    if (g := to_int(r.get('gas_used'))) is not None and g >= GAS_BUDGET
}

with open(out_path, 'w') as f:
    f.write("# Optimization Gas + Size Impact Summary\n\n")
    f.write(f"Runnable inputs measured: {len(inputs_order)}\n\n")
    f.write("Positive delta = optimization saves bytes / gas when **on** "
            "(turning it off makes the JAM bigger / costs more gas).\n\n")
    if oom_inputs:
        f.write("**Baseline OOM (gas-impact unmeasurable):** "
                + ", ".join(f"`{n}`" for n in sorted(oom_inputs))
                + ". These fixtures exhausted the gas budget under the all-opts-on baseline, "
                  "so every variant also OOMs and delta_gas is structurally 0 — only "
                  "delta_jam is meaningful here.\n\n")

    # ---------- Per-flag aggregate ----------
    f.write("## Per-flag aggregate\n\n")
    f.write("| Flag | Total ΔJAM | Total ΔGas | "
            "JAM regr. | Gas regr. | Sign disagreements | Run failures |\n")
    f.write("|------|-----------:|-----------:|----------:|----------:|"
            "-------------------:|-------------:|\n")
    summary_rows = []
    for flag in rows:
        total_jam = 0; total_gas = 0
        jam_regr = 0; gas_regr = 0; disagree = 0; runfail = 0
        for r in rows[flag]:
            if r['compile_ok'] != '1':
                continue
            dj = to_int(r['delta_jam'])
            dg = to_int(r['delta_gas'])
            if dj is not None:
                total_jam += dj
                if dj < 0:
                    jam_regr += 1
            if r['run_status'] not in ('ok',):
                runfail += 1
                continue
            # Skip gas math for OOM-capped fixtures (deltas always pin to 0).
            if r['input'] in oom_inputs:
                continue
            if dg is not None:
                total_gas += dg
                if dg < 0:
                    gas_regr += 1
            if dj is not None and dg is not None and dj != 0 and dg != 0:
                if (dj > 0) != (dg > 0):
                    disagree += 1
        summary_rows.append((flag, total_jam, total_gas, jam_regr, gas_regr,
                             disagree, runfail))
    # Sort by abs(disagreements) DESC so most-conflicted flags float up.
    summary_rows.sort(key=lambda r: (-r[5], -abs(r[2]), r[0]))
    for flag, tj, tg, jr, gr, dis, rf in summary_rows:
        f.write(f"| `{flag}` | {tj:+d} | {tg:+d} | {jr} | {gr} | {dis} | {rf} |\n")

    # ---------- Sign-disagreement detail ----------
    f.write("\n## Sign-disagreement cells (optimization helps one axis, hurts the other)\n\n")
    f.write("Rows where ΔJAM and ΔGas have opposite signs (both non-zero). "
            "These are the cases where 'is this opt worth keeping?' depends "
            "on which axis you care about.\n\n")
    f.write("| Flag | Fixture | ΔJAM | ΔGas | Baseline JAM | Baseline Gas | Status |\n")
    f.write("|------|---------|-----:|-----:|-------------:|-------------:|--------|\n")
    any_disagree = False
    for flag in (s[0] for s in summary_rows):
        for r in rows[flag]:
            if r['compile_ok'] != '1' or r['run_status'] != 'ok':
                continue
            if r['input'] in oom_inputs:
                continue
            dj = to_int(r['delta_jam'])
            dg = to_int(r['delta_gas'])
            if dj is None or dg is None or dj == 0 or dg == 0:
                continue
            if (dj > 0) == (dg > 0):
                continue
            any_disagree = True
            bj = r['baseline_jam']
            bg = r['baseline_gas']
            f.write(f"| `{flag}` | {r['input']} | {dj:+d} | {dg:+d} | {bj} | {bg} | {r['run_status']} |\n")
    if not any_disagree:
        f.write("\n_(none — every (flag, fixture) cell either agrees on direction or has zero delta on at least one axis.)_\n")

    # ---------- Per-fixture detail (gas-only view) ----------
    f.write("\n## Per-fixture gas deltas (positive = opt saves gas)\n\n")
    f.write("| Flag |")
    for inp in inputs_order: f.write(f" {inp} |")
    f.write("\n|------|")
    for _ in inputs_order: f.write("---:|")
    f.write("\n")
    by_flag_gas = {flag: {r['input']: r['delta_gas'] for r in rs} for flag, rs in rows.items()}
    for flag, *_ in summary_rows:
        f.write(f"| `{flag}` |")
        for inp in inputs_order:
            dg = by_flag_gas[flag].get(inp, '-')
            f.write(f" {dg} |")
        f.write("\n")

    # ---------- Per-fixture detail (JAM-only view) ----------
    f.write("\n## Per-fixture JAM deltas (positive = opt saves bytes)\n\n")
    f.write("| Flag |")
    for inp in inputs_order: f.write(f" {inp} |")
    f.write("\n|------|")
    for _ in inputs_order: f.write("---:|")
    f.write("\n")
    by_flag_jam = {flag: {r['input']: r['delta_jam'] for r in rs} for flag, rs in rows.items()}
    for flag, *_ in summary_rows:
        f.write(f"| `{flag}` |")
        for inp in inputs_order:
            dj = by_flag_jam[flag].get(inp, '-')
            f.write(f" {dj} |")
        f.write("\n")

print(f"Summary written to {out_path}", file=sys.stderr)
PY

echo
echo "==> Done."
echo "    Baseline:    $BASELINE_CSV"
echo "    Per-flag:    $DELTAS_CSV"
echo "    Summary:     $SUMMARY_MD"
