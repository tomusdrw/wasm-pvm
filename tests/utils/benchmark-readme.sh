#!/usr/bin/env bash
# Generates the benchmark comparison table for README.md.
#
# Runs benchmarks twice (optimized and no-opt) and produces a markdown table
# comparing JAM sizes and gas usage with/without PVM-level optimizations.
#
# Usage:
#   ./tests/utils/benchmark-readme.sh
#
# Prerequisites: cargo, bun, node, python3, wat2wasm must be in PATH.
# Run from the project root directory.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BENCHMARK_SH="$SCRIPT_DIR/benchmark.sh"

echo "=== Running optimized benchmarks ===" >&2
opt_output=$("$BENCHMARK_SH" 2>/dev/null)

echo "=== Running no-opt benchmarks ===" >&2
noopt_output=$("$BENCHMARK_SH" --no-opt 2>/dev/null)

# Parse both outputs and generate the README comparison table
python3 - <<'PYEOF' "$opt_output" "$noopt_output"
import sys

opt_text = sys.argv[1]
noopt_text = sys.argv[2]

def parse_main_table(text):
    """Parse the main benchmark table. Returns dict of desc -> (wasm_size, jam_size, gas)."""
    rows = {}
    in_main = False
    for line in text.split('\n'):
        if '| Benchmark' in line and 'WASM Size' in line:
            in_main = True
            continue
        if in_main and line.startswith('|---'):
            continue
        if in_main and line.startswith('|'):
            parts = [p.strip() for p in line.split('|')]
            # parts: ['', desc, wasm_size, jam_size, gas, time, '']
            if len(parts) >= 6:
                desc = parts[1]
                wasm_size = parts[2]
                jam_size = parts[3]
                gas = parts[4]
                if desc:
                    rows[desc] = (wasm_size, jam_size, gas)
        elif in_main and not line.startswith('|'):
            in_main = False
    return rows

def parse_pip_table(text):
    """Parse the PVM-in-PVM table. Returns dict of desc -> (jam_size, outer_gas)."""
    rows = {}
    in_pip = False
    for line in text.split('\n'):
        if '| Benchmark' in line and 'Outer Gas' in line:
            in_pip = True
            continue
        if in_pip and line.startswith('|---'):
            continue
        if in_pip and line.startswith('|'):
            parts = [p.strip() for p in line.split('|')]
            # parts: ['', desc, jam_size, outer_gas, time, '']
            if len(parts) >= 5:
                desc = parts[1]
                jam_size = parts[2]
                outer_gas = parts[3]
                if desc:
                    rows[desc] = (jam_size, outer_gas)
        elif in_pip and not line.startswith('|'):
            in_pip = False
    return rows

def fmt_size(b):
    """Format byte count as human-readable size."""
    try:
        n = int(b)
        if n >= 1024:
            return f"{n/1024:.1f} KB"
        return f"{n} B"
    except (ValueError, TypeError):
        return b

def pct(before, after):
    """Compute percentage change string."""
    try:
        b, a = int(before), int(after)
        if b == 0:
            return "-"
        p = (a - b) / b * 100
        return f"{p:+.0f}%"
    except (ValueError, TypeError):
        return "-"

def fmt_with_pct(noopt_val, opt_val, is_size=False):
    """Format an opt value with percentage change from noopt."""
    p = pct(noopt_val, opt_val)
    if is_size:
        return f"{fmt_size(opt_val)} ({p})" if p != "-" else fmt_size(opt_val)
    try:
        formatted = f"{int(opt_val):,}"
        return f"{formatted} ({p})" if p != "-" else formatted
    except (ValueError, TypeError):
        return str(opt_val)

# Parse tables
opt_main = parse_main_table(opt_text)
noopt_main = parse_main_table(noopt_text)
opt_pip = parse_pip_table(opt_text)
noopt_pip = parse_pip_table(noopt_text)

# Print main comparison table
print("All PVM-level optimizations disabled vs enabled (default):")
print()
print("| Benchmark | WASM size | JAM (no opt) | JAM (opt) | Gas (no opt) | Gas (opt) |")
print("|-----------|----------|-------------|-----------|-------------|-----------|")

# Use noopt keys as the canonical order (both runs have the same benchmarks)
for desc in noopt_main:
    wasm_size, noopt_jam, noopt_gas = noopt_main[desc]
    _, opt_jam, opt_gas = opt_main.get(desc, ("?", "?", "?"))

    ws = fmt_size(wasm_size)
    nj = fmt_size(noopt_jam)
    oj = fmt_with_pct(noopt_jam, opt_jam, is_size=True)

    if noopt_gas == "-":
        ng = "-"
        og = "-"
    else:
        try:
            ng = f"{int(noopt_gas):,}"
        except (ValueError, TypeError):
            ng = noopt_gas
        og = fmt_with_pct(noopt_gas, opt_gas)

    print(f"| {desc} | {ws} | {nj} | {oj} | {ng} | {og} |")

print()

# Print PVM-in-PVM comparison table with direct gas and overhead
if noopt_pip and opt_pip:
    # Build a lookup from PiP desc -> direct gas (opt) by stripping "PiP " prefix
    # and matching against the main table descriptions
    def find_direct_gas(pip_desc):
        """Try to find direct execution gas for a PiP benchmark."""
        # Strip "PiP " prefix to get the program name
        if pip_desc.startswith("PiP "):
            prog_name = pip_desc[4:]
            # Look for exact match in main table
            if prog_name in opt_main:
                _, _, gas = opt_main[prog_name]
                try:
                    return int(gas)
                except (ValueError, TypeError):
                    pass
        return None

    # Get TRAP gas for overhead calculation
    trap_opt_gas = None
    if "PiP TRAP" in opt_pip:
        try:
            trap_opt_gas = int(opt_pip["PiP TRAP"][1])
        except (ValueError, TypeError):
            pass

    print("PVM-in-PVM: programs executed inside the anan-as PVM interpreter (outer gas cost):")
    print()
    print("| Benchmark | Gas (no opt) | Gas (opt) | Direct gas | Overhead |")
    print("|-----------|-------------|-----------|------------|----------|")

    for desc in noopt_pip:
        _, noopt_gas = noopt_pip[desc]
        _, opt_gas = opt_pip.get(desc, ("?", "?"))

        # Display name
        display = desc
        if desc == "PiP TRAP":
            display = "TRAP (interpreter overhead)"

        try:
            ng = f"{int(noopt_gas):,}"
        except (ValueError, TypeError):
            ng = noopt_gas
        og = fmt_with_pct(noopt_gas, opt_gas)

        # Direct gas and overhead
        direct = find_direct_gas(desc)
        if direct is not None and direct > 0:
            direct_str = f"{direct:,}"
            try:
                pip_gas = int(opt_gas)
                ratio = pip_gas / direct
                overhead_str = f"{ratio:,.0f}x"
            except (ValueError, TypeError):
                overhead_str = "-"
        elif desc == "PiP TRAP":
            direct_str = "-"
            overhead_str = "-"
        else:
            direct_str = "-"
            overhead_str = "-"

        print(f"| {display} | {ng} | {og} | {direct_str} | {overhead_str} |")

    print()
PYEOF
