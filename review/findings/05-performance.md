# 05 - Performance Inefficiencies

**Category**: Performance
**Date**: 2026-02-13 (Updated for V2 Architecture)
**Status**: Functional but Unoptimized

---

## Summary

The current architecture prioritizes correctness and simplicity over generated code quality. The generated PVM code is functional but bloated due to the lack of register allocation.

---

## Inefficiencies

### 1. Excessive Stack Traffic (Slot-Based Lowering) 🔴
**Impact**: High
Every SSA value lives in a stack slot. Every instruction involves:
1.  Load operands from stack to temp regs.
2.  Execute.
3.  Store result to stack.

**Example**: `a = b + c` becomes:

```text
LoadInd r2, SP, offset_b
LoadInd r3, SP, offset_c
Add r4, r2, r3
StoreInd SP, r4, offset_a
```

(4 instructions + 3 memory accesses) vs (1 instruction) if allocated.

### 2. Byte-by-Byte Memory Ops 🟡
**Impact**: High (for large copies)
`memory.copy` and `memory.fill` are lowered to byte loops.
*   PVM has `LoadIndU64`/`StoreIndU64`.
*   We should use 64-bit copies for the bulk of data to increase throughput by 8x.

### 3. Redundant `Fallthrough` Instructions 🟢
**Impact**: Low
`PvmEmitter::define_label` emits `Fallthrough` if the previous instruction wasn't terminating. This is good for correctness but sometimes results in chains of fallthroughs or unnecessary instructions if jumps are close.

---

## Recommendations

1.  **Implement Register Allocator**: This is the single biggest performance win. Even a basic Linear Scan allocator would drastically reduce code size and memory traffic.
2.  **Optimize Memory Ops**: Implement word-sized copy/fill loops.
3.  **Peephole Optimization**: A simple pass over PVM code to remove redundant moves or jumps.
