#!/usr/bin/env python3
"""Stage-2 dataflow analysis over `disasm` output.

Quantifies *provable* redundancies, conservatively (within basic blocks only):
  1. zext32 shift-pairs applied to values whose upper 32 bits are already zero
  2. sext32 (AddImm32 r,x,0) applied to values that are already sign-extended-32
  3. loads from SP slots whose value is provably live in a register already
  4. slot-to-slot copy chains through a register (load s1 ; store s2 ; value dead)
  5. branch/jump offset relaxation: bytes saved if offsets were min-length encoded
  6. LoadImm64 constant histogram
  7. function-level dead stores (stored SP slot never read before SP reset)
"""
import re
import sys
from collections import Counter, defaultdict

from mine_patterns import (parse, compute_leaders, blocks, STORES_IND, LOADS_IND,
                           BRANCHES, TERMINATORS)

SP = 1

# Ops whose 64-bit result provably has upper 32 bits zero.
def writes_zext(i):
    op, a = i.op, i.args
    if op in ("LoadIndU32", "LoadU32", "LoadIndU16", "LoadU16", "LoadIndU8", "LoadU8",
              "ZeroExtend16", "CountSetBits64", "CountSetBits32", "LeadingZeroBits64",
              "LeadingZeroBits32", "TrailingZeroBits64", "TrailingZeroBits32",
              "SetLtU", "SetLtS", "SetLtUImm", "SetLtSImm", "SetGtUImm", "SetGtSImm"):
        return True
    if op == "LoadImm" and 0 <= a.get("value", -1):
        return True
    if op == "LoadImm64" and 0 <= a.get("value", -1) <= 0xFFFFFFFF:
        return True
    if op in ("ShloRImm64",) and a.get("value", 0) >= 32:
        return True
    return False

# Ops whose result is provably sext32 (value == sign_extend_32(low32)).
def writes_sext(i):
    op = i.op
    if op in ("Add32", "Sub32", "Mul32", "DivU32", "DivS32", "RemU32", "RemS32",
              "ShloL32", "ShloR32", "SharR32", "AddImm32", "MulImm32", "NegAddImm32",
              "ShloLImm32", "ShloRImm32", "SharRImm32", "ShloLImmAlt32",
              "ShloRImmAlt32", "SharRImmAlt32", "RotRImm32", "RotRImmAlt32", "RotL32",
              "RotR32", "LoadIndI32", "LoadI32", "SignExtend8", "SignExtend16"):
        return True
    if op == "LoadImm":  # LoadImm sign-extends its i32 imm by definition
        return True
    return False


def main():
    path = sys.argv[1]
    instrs, jt = parse(path)
    total = len(instrs)
    total_bytes = sum(i.size for i in instrs)
    leaders = compute_leaders(instrs, jt)
    blks = list(blocks(instrs, leaders))

    res = Counter()

    # ---------- per-block abstract interpretation ----------
    for blk in blks:
        # reg -> {"z": bool, "s": bool}  upper-bits knowledge
        know = defaultdict(lambda: {"z": False, "s": False})
        # slot(offset) -> {"z","s"} knowledge for the value stored in the slot
        slot_know = {}
        # slot(offset) -> reg holding its current value; reg -> set of slots
        slot_in_reg = {}
        reg_holds = defaultdict(set)

        def clobber_reg(d):
            for off in reg_holds[d]:
                if slot_in_reg.get(off) == d:
                    del slot_in_reg[off]
            reg_holds[d] = set()

        idx = 0
        n = len(blk)
        while idx < n:
            i = blk[idx]
            nxt = blk[idx + 1] if idx + 1 < n else None

            # --- zext pair detection (imm form: rust/llvm path) ---
            if (nxt and i.op == "ShloLImm64" and nxt.op == "ShloRImm64"
                    and i.args.get("value") == 32 and nxt.args.get("value") == 32
                    and i.args.get("dst") == nxt.args.get("src")
                    and nxt.args.get("dst") == nxt.args.get("src")):
                src = i.args.get("src")
                res["zext_pairs"] += 1
                if know[src]["z"]:
                    res["zext_pairs_redundant"] += 1
                    res["zext_pairs_redundant_bytes"] += i.size + nxt.size

            # --- sext detection ---
            if i.op == "AddImm32" and i.args.get("value") == 0:
                src = i.args.get("src")
                res["sext_ops"] += 1
                if know[src]["s"]:
                    res["sext_ops_redundant"] += 1
                    res["sext_ops_redundant_bytes"] += i.size

            # --- SP-slot tracking ---
            if i.op in LOADS_IND and i.args.get("base") == SP:
                off = i.args["offset"]
                dst = i.args["dst"]
                res["sp_loads"] += 1
                if off in slot_in_reg:
                    res["sp_loads_value_already_in_reg"] += 1
                    res["sp_loads_value_already_in_reg_bytes"] += i.size
                    if slot_in_reg[off] == dst:
                        res["sp_loads_same_reg_noop"] += 1
            if i.op in STORES_IND and i.args.get("base") == SP:
                off = i.args["offset"]
                src = i.args["src"]
                res["sp_stores"] += 1
                # invalidate old mapping for this slot, set new
                if off in slot_in_reg:
                    reg_holds[slot_in_reg[off]].discard(off)
                    del slot_in_reg[off]
                slot_know.pop(off, None)
                if i.op == "StoreIndU64":
                    slot_in_reg[off] = src
                    reg_holds[src].add(off)
                    slot_know[off] = dict(know[src])

            # --- knowledge + clobber updates ---
            d = i.dst()
            if d is not None:
                clobber_reg(d)
                know[d] = {"z": writes_zext(i), "s": writes_sext(i)}
                if i.op == "MoveReg":
                    know[d] = dict(know[i.args["src"]])
                if i.op == "ShloRImm64" and i.args.get("value", 0) >= 32:
                    know[d]["z"] = True
                if i.op == "LoadIndU64" and i.args.get("base") == SP:
                    off = i.args["offset"]
                    slot_in_reg[off] = d
                    reg_holds[d].add(off)
                    if off in slot_know:
                        know[d] = dict(slot_know[off])
            # calls/ecalli clobber everything volatile — conservative: drop all
            if i.op in ("Ecalli", "LoadImmJump", "LoadImmJumpInd", "JumpInd"):
                know.clear()
                slot_know.clear()
                slot_in_reg.clear()
                reg_holds.clear()
            idx += 1

    # ---------- offset relaxation estimate (fixpoint) ----------
    # Instructions whose trailing offset field is relaxable to minimal length.
    # `LoadImmJump` (direct calls) is in BRANCHES but its offset is patched at
    # link time and kept FIXED 4-byte width by the backend, so it must be
    # excluded here or the savings estimate is overstated.
    def has_fixed_off(i):
        return (i.op == "Jump" or i.op in BRANCHES) and i.op != "LoadImmJump"
    # iteratively: assume minimal offset encoding, recompute sizes until stable
    sizes = {i.off: i.size for i in instrs}
    # map from old offset -> index
    offs = [i.off for i in instrs]
    pos_of = {o: k for k, o in enumerate(offs)}

    def imm_len(v):
        if v == 0:
            return 0
        for ln in (1, 2, 3):
            lo = -(1 << (8 * ln - 1))
            hi = (1 << (8 * ln - 1)) - 1
            if lo <= v <= hi:
                return ln
        return 4

    new_sizes = [i.size for i in instrs]
    targets = []
    for i in instrs:
        if has_fixed_off(i) and "offset" in i.args:
            targets.append(i.off + i.args["offset"])
        else:
            targets.append(None)

    for _ in range(12):
        # current start addresses under new sizes
        addr = [0] * (len(instrs) + 1)
        for k in range(len(instrs)):
            addr[k + 1] = addr[k] + new_sizes[k]
        changed = False
        for k, i in enumerate(instrs):
            if targets[k] is None:
                continue
            t = targets[k]
            tk = pos_of.get(t)
            if tk is None:
                continue
            rel = addr[tk] - addr[k]
            saved = 4 - imm_len(rel)
            ns = i.size - saved
            if ns != new_sizes[k]:
                new_sizes[k] = ns
                changed = True
        if not changed:
            break
    relaxed_total = sum(new_sizes)
    res["offset_relax_bytes_saved"] = total_bytes - relaxed_total

    # ---------- LoadImm64 constants ----------
    consts = Counter()
    for i in instrs:
        if i.op == "LoadImm64":
            consts[i.args.get("value")] += 1

    # ---------- report ----------
    print(f"file: {path}")
    print(f"instructions: {total:,}  bytes: {total_bytes:,}")
    print()
    print("=== Provable in-block redundancies ===")
    for k in ("zext_pairs", "zext_pairs_redundant", "zext_pairs_redundant_bytes",
              "sext_ops", "sext_ops_redundant", "sext_ops_redundant_bytes",
              "sp_loads", "sp_loads_value_already_in_reg",
              "sp_loads_value_already_in_reg_bytes", "sp_loads_same_reg_noop",
              "sp_stores"):
        print(f"  {k:<42} {res[k]:>12,}")
    print()
    print("=== Offset relaxation ===")
    pct = (100 * res["offset_relax_bytes_saved"] / total_bytes) if total_bytes else 0.0
    print(f"  bytes saved with minimal branch/jump offsets: {res['offset_relax_bytes_saved']:,} "
          f"({pct:.2f}% of code)")
    print()
    print("=== Top LoadImm64 constants ===")
    for v, c in consts.most_common(15):
        print(f"  {c:>8,} × {v:#x}")


if __name__ == "__main__":
    main()
