#!/usr/bin/env python3
"""Mine PVM instruction-stream patterns from `disasm` output.

Usage:
  cargo run --release --example disasm -- file.jam > file.dis
  python3 experiments/mine_patterns.py file.dis

Reports:
  - opcode histogram (count + encoded bytes)
  - abstracted n-gram patterns (registers renamed canonically, immediates bucketed)
  - targeted detectors for known-suspect idioms (spill traffic, extension idioms,
    move chains, redundant reloads, branch-over-jump, ...)
"""
import re
import sys
from collections import Counter, defaultdict

LINE_RE = re.compile(r"^(\d+)\t(\d+)\t(\w+)(?: \{ (.*) \})?$")
FIELD_RE = re.compile(r"(\w+): (-?\d+)")

TERMINATORS = {
    "Jump", "JumpFixed", "JumpInd", "LoadImmJumpInd", "Trap", "Fallthrough",
}
BRANCHES = {
    "BranchEq", "BranchNe", "BranchLtU", "BranchLtS", "BranchGeU", "BranchGeS",
    "BranchEqImm", "BranchNeImm", "BranchLtUImm", "BranchLeUImm", "BranchGeUImm",
    "BranchGtUImm", "BranchLtSImm", "BranchLeSImm", "BranchGeSImm", "BranchGtSImm",
    "LoadImmJump",
}

STORES_IND = {"StoreIndU8", "StoreIndU16", "StoreIndU32", "StoreIndU64"}
LOADS_IND = {"LoadIndU8", "LoadIndI8", "LoadIndU16", "LoadIndI16", "LoadIndU32",
             "LoadIndI32", "LoadIndU64"}


class Instr:
    __slots__ = ("off", "size", "op", "args")

    def __init__(self, off, size, op, args):
        self.off = off
        self.size = size
        self.op = op
        self.args = args

    def __repr__(self):
        return f"{self.op}{self.args}"

    # Destination register written by this instruction, or None.
    def dst(self):
        a = self.args
        if self.op in STORES_IND or self.op.startswith("Store"):
            return None
        if "dst" in a:
            return a["dst"]
        if "reg" in a and self.op in ("LoadImm", "LoadImm64", "LoadImmJump"):
            return a["reg"]
        return None

    # Registers read by this instruction.
    def srcs(self):
        a = self.args
        out = []
        for k in ("src", "src1", "src2", "base", "cond"):
            if k in a:
                out.append(a[k])
        if self.op in STORES_IND or (self.op.startswith("Store") and "src" in a):
            pass  # src already collected
        if self.op == "JumpInd":
            out.append(a["reg"])
        return out


def parse(path):
    instrs = []
    jt = []
    with open(path) as f:
        for line in f:
            line = line.rstrip("\n")
            if line.startswith("# jump_table_entry "):
                jt.append(int(line.split()[-1]))
                continue
            m = LINE_RE.match(line)
            if not m:
                continue
            off, size, op, rest = int(m[1]), int(m[2]), m[3], m[4]
            args = {k: int(v) for k, v in FIELD_RE.findall(rest or "")}
            instrs.append(Instr(off, size, op, args))
    return instrs, jt


def compute_leaders(instrs, jt):
    starts = {i.off for i in instrs}
    leaders = set(jt)
    by_off = {i.off: i for i in instrs}
    n = len(instrs)
    for idx, i in enumerate(instrs):
        if i.op in BRANCHES or i.op == "Jump":
            off = i.args.get("offset")
            if off is not None:
                # offsets are relative to instruction start
                t = i.off + off
                if t in starts:
                    leaders.add(t)
        if (i.op in TERMINATORS or i.op in BRANCHES) and idx + 1 < n:
            leaders.add(instrs[idx + 1].off)
    return leaders


def blocks(instrs, leaders):
    cur = []
    for i in instrs:
        if i.off in leaders and cur:
            yield cur
            cur = []
        cur.append(i)
    if cur:
        yield cur


def imm_bucket(v):
    if v == 0:
        return "0"
    if -16 <= v <= 16:
        return "s"  # small
    return "L"


def abstract(seq):
    """Canonical pattern key for an instruction sequence: registers renamed in
    first-use order, immediates bucketed."""
    reg_map = {}

    def reg(r):
        if r not in reg_map:
            reg_map[r] = f"R{len(reg_map)}"
        return reg_map[r]

    parts = []
    for i in seq:
        fields = []
        for k, v in i.args.items():
            if k in ("dst", "src", "src1", "src2", "base", "reg", "cond"):
                fields.append(f"{k}={reg(v)}")
            elif k in ("value", "offset", "imm"):
                fields.append(f"{k}~{imm_bucket(v)}")
            else:
                fields.append(f"{k}~{imm_bucket(v)}")
        parts.append(f"{i.op}({','.join(fields)})")
    return " ; ".join(parts)


def main():
    path = sys.argv[1]
    instrs, jt = parse(path)
    total = len(instrs)
    total_bytes = sum(i.size for i in instrs)
    leaders = compute_leaders(instrs, jt)
    blks = list(blocks(instrs, leaders))

    print(f"file: {path}")
    print(f"instructions: {total:,}   bytes: {total_bytes:,}   blocks: {len(blks):,}")
    print()

    # --- opcode histogram ---
    freq, byts = Counter(), Counter()
    for i in instrs:
        freq[i.op] += 1
        byts[i.op] += i.size
    print("=== Opcode histogram (top 30) ===")
    for op, c in freq.most_common(30):
        print(f"  {op:<22} {c:>9,} ({100*c/total:5.2f}%)  {byts[op]:>11,} B ({100*byts[op]/total_bytes:5.2f}%)")
    print()

    # --- targeted detectors ---
    det_count = Counter()
    det_bytes = Counter()

    def hit(name, seq):
        det_count[name] += 1
        det_bytes[name] += sum(i.size for i in seq)

    sp = 1  # stack pointer register
    for blk in blks:
        for idx, i in enumerate(blk):
            nxt = blk[idx + 1] if idx + 1 < len(blk) else None
            nx2 = blk[idx + 2] if idx + 2 < len(blk) else None

            # spill traffic: SP-relative load/store
            if i.op in STORES_IND and i.args.get("base") == sp:
                hit("sp_store (spill write)", [i])
            if i.op in LOADS_IND and i.args.get("base") == sp:
                hit("sp_load (spill read)", [i])

            # store X -> load same slot immediately
            if (nxt and i.op in STORES_IND and nxt.op in LOADS_IND
                    and i.args.get("base") == nxt.args.get("base")
                    and i.args.get("offset") == nxt.args.get("offset")):
                hit("store_then_load_same_slot", [i, nxt])

            # same slot loaded twice in a row
            if (nxt and i.op in LOADS_IND and nxt.op in LOADS_IND
                    and i.args.get("base") == nxt.args.get("base")
                    and i.args.get("offset") == nxt.args.get("offset")):
                hit("double_load_same_slot", [i, nxt])

            # zext32 via shl32/shr32 (reg form with preceding LoadImm 32, or imm form)
            if (nxt and i.op == "ShloLImm64" and nxt.op == "ShloRImm64"
                    and i.args.get("value") == 32 and nxt.args.get("value") == 32):
                hit("zext32_via_shifts_imm", [i, nxt])
            if (nx2 and i.op == "LoadImm" and i.args.get("value") == 32
                    and nxt and nxt.op == "ShloL64" and nx2.op == "ShloR64"):
                hit("zext32_via_shifts_reg", [i, nxt, nx2])

            # sign-extend-32 move: AddImm32 dst,src,0
            if i.op == "AddImm32" and i.args.get("value") == 0:
                if i.args.get("dst") == i.args.get("src"):
                    hit("sext32_inplace (AddImm32 r,r,0)", [i])
                else:
                    hit("sext32_move (AddImm32 d,s,0)", [i])

            # move chains
            if i.op == "MoveReg":
                hit("move_reg", [i])
                if nxt and nxt.op == "MoveReg" and nxt.args.get("src") == i.args.get("dst"):
                    hit("move_chain (mv a<-b; mv c<-a)", [i, nxt])

            # add 0 (64-bit nop-ish move)
            if i.op == "AddImm64" and i.args.get("value") == 0:
                hit("addimm64_zero (move)", [i])

            # LoadImm + reg-reg ALU that has an imm form
            if (nxt and i.op == "LoadImm"
                    and nxt.op in ("Add64", "Add32", "And", "Or", "Xor", "Mul64", "Mul32",
                                   "ShloL64", "ShloR64", "SharR64", "ShloL32", "ShloR32",
                                   "SetLtU", "SetLtS")
                    and i.args.get("reg") in (nxt.args.get("src1"), nxt.args.get("src2"))):
                hit(f"loadimm_then_{nxt.op} (imm-form exists)", [i, nxt])

            # branch over jump: Branch* skipping a Jump that lands right after
            if i.op in BRANCHES and nxt and nxt.op == "Jump":
                t = i.off + i.args.get("offset", 0)
                if nx2 is not None and t == nx2.off:
                    hit("branch_over_jump", [i, nxt])

            # LoadImm64 (10 bytes!)
            if i.op == "LoadImm64":
                hit("loadimm64_10B", [i])

    print("=== Targeted detectors ===")
    for name, c in det_count.most_common():
        b = det_bytes[name]
        print(f"  {name:<40} {c:>9,} ({100*c/total:5.2f}% instrs)  {b:>11,} B ({100*b/total_bytes:5.2f}%)")
    print()

    # --- abstracted n-grams ---
    for n in (2, 3):
        grams = Counter()
        gbytes = Counter()
        for blk in blks:
            for idx in range(len(blk) - n + 1):
                seq = blk[idx:idx + n]
                key = abstract(seq)
                grams[key] += 1
                gbytes[key] += sum(i.size for i in seq)
        print(f"=== Top abstracted {n}-grams ===")
        for key, c in grams.most_common(25):
            print(f"  {c:>8,} ({100*n*c/total:5.2f}% instr-slots, {gbytes[key]:>10,} B)  {key}")
        print()


if __name__ == "__main__":
    main()
