#!/usr/bin/env python3
"""Join a PC-count profile with a disasm to attribute dynamic gas to pattern
categories.

Usage: profile_join.py <file.dis> <profile.txt>
"""
import sys
from collections import Counter

from mine_patterns import parse, STORES_IND, LOADS_IND, BRANCHES, TERMINATORS

SP = 1


def main():
    dis_path, prof_path = sys.argv[1], sys.argv[2]
    instrs, _jt = parse(dis_path)
    counts = {}
    total_steps = 0
    for line in open(prof_path):
        if line.startswith("#"):
            if "total_steps" in line:
                total_steps = int(line.split()[-1])
            continue
        parts = line.split()
        if len(parts) != 2 or not parts[0].isdigit():
            continue
        pc, c = parts
        counts[int(pc)] = int(c)

    by_off = {i.off: i for i in instrs}
    cat = Counter()
    for off, c in counts.items():
        i = by_off.get(off)
        if i is None:
            cat["<unknown pc>"] += c
            continue
        op = i.op
        if op in LOADS_IND and i.args.get("base") == SP:
            cat["spill load (SP)"] += c
        elif op in STORES_IND and i.args.get("base") == SP:
            cat["spill store (SP)"] += c
        elif op in LOADS_IND or (op.startswith("Load") and op not in ("LoadImm", "LoadImm64", "LoadImmJump", "LoadImmJumpInd")):
            cat["wasm memory load"] += c
        elif op in STORES_IND or op.startswith("Store"):
            cat["wasm memory store"] += c
        elif op in ("ShloLImm64", "ShloRImm64") and i.args.get("value") == 32:
            cat["zext32 shift-pair member"] += c
        elif op == "AddImm32" and i.args.get("value") == 0:
            cat["sext32 (AddImm32 r,x,0)"] += c
        elif op == "MoveReg":
            cat["MoveReg"] += c
        elif op in ("LoadImm", "LoadImm64"):
            cat["LoadImm/64"] += c
        elif op in BRANCHES or op in ("Jump", "JumpInd", "LoadImmJump", "LoadImmJumpInd"):
            cat["control flow"] += c
        elif op == "Fallthrough":
            cat["Fallthrough"] += c
        else:
            cat["real ALU/other"] += c

    print(f"total dynamic gas (steps): {total_steps:,}")
    if total_steps <= 0:
        print("  (no steps recorded — empty or missing profile header)")
        return
    for k, v in cat.most_common():
        print(f"  {k:<28} {v:>10,}  ({100*v/total_steps:5.2f}%)")


if __name__ == "__main__":
    main()
