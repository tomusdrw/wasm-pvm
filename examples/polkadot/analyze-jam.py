#!/usr/bin/env python3
"""Analyze a JAM/SPI program blob: count opcode frequency and instruction sizes.

Pairs with the SPI parser in tests/utils/verify-jam.ts but produces a histogram
ranked by both frequency and total bytes contributed.
"""
import struct
import sys
from collections import Counter

OPCODES = {
    0: "Trap", 1: "Fallthrough", 10: "Ecalli", 20: "LoadImm64",
    30: "StoreImmU8", 31: "StoreImmU16", 32: "StoreImmU32", 33: "StoreImmU64",
    40: "Jump", 50: "JumpInd", 51: "LoadImm",
    52: "LoadU8", 53: "LoadI8", 54: "LoadU16", 55: "LoadI16",
    56: "LoadU32", 57: "LoadI32", 58: "LoadU64",
    59: "StoreU8", 60: "StoreU16", 61: "StoreU32", 62: "StoreU64",
    70: "StoreImmIndU8", 71: "StoreImmIndU16", 72: "StoreImmIndU32", 73: "StoreImmIndU64",
    80: "LoadImmJump",
    81: "BranchEqImm", 82: "BranchNeImm",
    83: "BranchLtUImm", 84: "BranchLeUImm", 85: "BranchGeUImm", 86: "BranchGtUImm",
    87: "BranchLtSImm", 88: "BranchLeSImm", 89: "BranchGeSImm", 90: "BranchGtSImm",
    100: "MoveReg", 101: "Sbrk",
    102: "CountSetBits64", 103: "CountSetBits32",
    104: "LeadingZeroBits64", 105: "LeadingZeroBits32",
    106: "TrailingZeroBits64", 107: "TrailingZeroBits32",
    108: "SignExtend8", 109: "SignExtend16", 110: "ZeroExtend16", 111: "ReverseBytes",
    120: "StoreIndU8", 121: "StoreIndU16", 122: "StoreIndU32", 123: "StoreIndU64",
    124: "LoadIndU8", 125: "LoadIndI8", 126: "LoadIndU16", 127: "LoadIndI16",
    128: "LoadIndU32", 129: "LoadIndI32", 130: "LoadIndU64",
    131: "AddImm32", 132: "AndImm", 133: "XorImm", 134: "OrImm",
    135: "MulImm32", 136: "SetLtUImm", 137: "SetLtSImm",
    138: "ShloLImm32", 139: "ShloRImm32", 140: "SharRImm32",
    141: "NegAddImm32", 142: "SetGtUImm", 143: "SetGtSImm",
    144: "ShloLImmAlt32", 145: "ShloRImmAlt32", 146: "SharRImmAlt32",
    147: "CmovIzImm", 148: "CmovNzImm",
    149: "AddImm64", 150: "MulImm64",
    151: "ShloLImm64", 152: "ShloRImm64", 153: "SharRImm64",
    154: "NegAddImm64",
    155: "ShloLImmAlt64", 156: "ShloRImmAlt64", 157: "SharRImmAlt64",
    158: "RotRImm64", 159: "RotRImmAlt64", 160: "RotRImm32", 161: "RotRImmAlt32",
    170: "BranchEq", 171: "BranchNe",
    172: "BranchLtU", 173: "BranchLtS", 174: "BranchGeU", 175: "BranchGeS",
    180: "LoadImmJumpInd",
    190: "Add32", 191: "Sub32", 192: "Mul32",
    193: "DivU32", 194: "DivS32", 195: "RemU32", 196: "RemS32",
    197: "ShloL32", 198: "ShloR32", 199: "SharR32",
    200: "Add64", 201: "Sub64", 202: "Mul64",
    203: "DivU64", 204: "DivS64", 205: "RemU64", 206: "RemS64",
    207: "ShloL64", 208: "ShloR64", 209: "SharR64",
    210: "And", 211: "Xor", 212: "Or",
    213: "MulUpperSS", 214: "MulUpperUU", 215: "MulUpperSU",
    216: "SetLtU", 217: "SetLtS",
    218: "CmovIz", 219: "CmovNz",
    220: "RotL64", 221: "RotL32", 222: "RotR64", 223: "RotR32",
    224: "AndInv", 225: "OrInv", 226: "Xnor",
    227: "Max", 228: "MaxU", 229: "Min", 230: "MinU",
}


def read_var_u32(data, off):
    fb = data[off]
    if fb < 0x80:
        return fb, 1
    if fb < 0xc0:
        return ((fb - 0x80) << 8) | data[off + 1], 2
    if fb < 0xe0:
        return ((fb - 0xc0) << 16) | data[off + 1] | (data[off + 2] << 8), 3
    if fb < 0xf0:
        return ((fb - 0xe0) << 24) | data[off + 1] | (data[off + 2] << 8) | (data[off + 3] << 16), 4
    return struct.unpack_from("<Q", data, off + 1)[0], 9


def parse_jam(path):
    with open(path, "rb") as f:
        data = f.read()

    off = 0
    # Metadata prefix: varint length + metadata bytes
    meta_len, n = read_var_u32(data, off)
    off += n
    off += meta_len
    ro_len = int.from_bytes(data[off:off+3], "little")
    off += 3
    rw_len = int.from_bytes(data[off:off+3], "little")
    off += 3
    heap_pages = int.from_bytes(data[off:off+2], "little")
    off += 2
    stack_size = int.from_bytes(data[off:off+3], "little")
    off += 3
    off += ro_len + rw_len  # skip ro/rw data
    code_total_len = int.from_bytes(data[off:off+4], "little")
    off += 4
    blob_end = off + code_total_len
    if blob_end > len(data):
        raise ValueError(
            f"Invalid JAM: declared code blob ({code_total_len} bytes) "
            f"exceeds remaining file size ({len(data) - off} bytes)"
        )

    blob_start = off
    jt_len, n = read_var_u32(data, off)
    off += n
    jt_item_bytes = data[off]
    off += 1
    blob_code_len, n = read_var_u32(data, off)
    off += n

    off += jt_len * jt_item_bytes  # skip jump table
    code_start = off
    code_end = code_start + blob_code_len
    if code_end > blob_end:
        raise ValueError(
            f"Invalid JAM: code section ({blob_code_len} bytes from "
            f"offset {code_start}) extends past the declared blob end"
        )
    code = data[code_start:code_end]
    # Mask is bounded by the declared code blob length so trailing bytes
    # (if any) don't get folded into instruction-start counting.
    mask = data[code_end:blob_end]
    return code, mask, {
        "ro_len": ro_len,
        "rw_len": rw_len,
        "heap_pages": heap_pages,
        "stack_size": stack_size,
        "jt_entries": jt_len,
        "code_len": blob_code_len,
    }


def iter_instructions(code, mask):
    """Yield (start_offset, length, opcode) tuples."""
    n = len(code)
    # Convert mask to a list of instruction-start positions.
    starts = []
    for byte_idx, byte in enumerate(mask):
        for bit_idx in range(8):
            pos = byte_idx * 8 + bit_idx
            if pos >= n:
                break
            if (byte >> bit_idx) & 1:
                starts.append(pos)
    starts.append(n)
    for i in range(len(starts) - 1):
        start, end = starts[i], starts[i + 1]
        yield start, end - start, code[start]


def main():
    if len(sys.argv) < 2:
        print("Usage: analyze-jam.py <jam-file>", file=sys.stderr)
        sys.exit(1)
    path = sys.argv[1]
    code, mask, meta = parse_jam(path)

    freq = Counter()
    size = Counter()
    pair_freq = Counter()  # consecutive opcode pairs
    prev_op = None
    total_instrs = 0
    for _start, length, op in iter_instructions(code, mask):
        name = OPCODES.get(op, f"Opcode_{op}")
        freq[name] += 1
        size[name] += length
        if prev_op is not None:
            pair_freq[(prev_op, name)] += 1
        prev_op = name
        total_instrs += 1

    total_bytes = sum(size.values())
    print(f"File: {path}")
    print(f"Code: {meta['code_len']:,} bytes  Jump-table: {meta['jt_entries']:,} entries")
    print(f"Total instructions: {total_instrs:,}")
    print(f"Avg encoded size: {total_bytes / max(1, total_instrs):.2f} bytes/instr")
    print()

    print(f"{'Opcode':<24} {'Count':>10} {'%':>6} {'Bytes':>12} {'%':>6} {'AvgSz':>6}")
    print("-" * 72)
    for name, cnt in freq.most_common(40):
        bcnt = size[name]
        avg = bcnt / cnt if cnt else 0
        pct_c = 100.0 * cnt / total_instrs
        pct_b = 100.0 * bcnt / total_bytes
        print(f"{name:<24} {cnt:>10,} {pct_c:>6.2f} {bcnt:>12,} {pct_b:>6.2f} {avg:>6.2f}")

    print()
    print("=== Top consecutive instruction pairs (size opportunities) ===")
    for (a, b), cnt in pair_freq.most_common(25):
        pct = 100.0 * cnt / max(1, total_instrs - 1)
        print(f"  {a:<22} → {b:<22} {cnt:>10,}  ({pct:>5.2f}%)")


if __name__ == "__main__":
    main()
