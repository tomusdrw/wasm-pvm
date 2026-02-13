# 01 - Architectural Design Flaws

**Category**: Architecture & Design
**Date**: 2026-02-13 (Updated for V2 Architecture)
**Status**: Significantly Improved

---

## Summary

The architecture has undergone a massive rewrite since the previous review. The compiler now uses **LLVM IR** as an intermediate representation, leveraging `inkwell` for the frontend and a custom backend for PVM lowering. This resolves the critical "No IR" flaw and enables standard optimizations.

However, the new backend implementation introduces new inefficiencies, primarily due to a lack of register allocation.

---

## Flaw 1: Inefficient Slot-Based Backend (No Register Allocator)

**Severity**: 🟡 Medium (Performance)
**Location**: `crates/wasm-pvm/src/llvm_backend/lowering.rs`
**Impact**: Excessive code size and memory traffic

### Problem Description

The PVM backend (`lowering.rs`) currently uses a "stack slot" approach for all values.
1.  Every LLVM SSA value is assigned a dedicated slot in the PVM stack frame.
2.  Every instruction loads operands from slots into temporary registers (`r2`, `r3`), computes, and stores the result back to a slot.

```rust
// Current lowering of c = a + b
LoadIndU64 r2, SP, offset_a  // Load a
LoadIndU64 r3, SP, offset_b  // Load b
Add64 r4, r2, r3             // Compute
StoreIndU64 SP, r4, offset_c // Store c
```

This generates 4 instructions for a simple add, whereas a register-allocated approach could do it in 1 (if `a` and `b` are already in registers).

### Why This Was Done

This approach guarantees correctness (no register spills needed during computation) and simplifies the backend significantly. It avoids the complexity of graph coloring or linear scan allocation.

### Recommendation

Implement a proper register allocator (e.g., Linear Scan) for the PVM backend to map LLVM values to PVM registers (`r0`-`r12`) where possible, spilling only when necessary.

---

## Flaw 2: Manual `memory.copy` / `memory.fill` Loops (Performance Only)

**Severity**: 🟡 Medium (Performance)
**Location**: `crates/wasm-pvm/src/llvm_backend/lowering.rs` (`emit_pvm_memory_copy`, `emit_pvm_memory_fill`)
**Impact**: Poor performance for large memory operations

### Problem Description

PVM instructions `MemoryCopy` and `MemoryFill` do not exist (despite being intrinsics in the frontend). The backend lowers them to explicit byte-by-byte loops in PVM bytecode.

**Correctness**: ✅ **FIXED** - The `memory.copy` lowering in `lowering.rs` now implements memmove-style logic with proper overlap detection. When `dest > src`, it copies backward (from end to start) to avoid overwriting source before reading. This ensures correct behavior for overlapping regions.

**Performance Issue**: Byte-by-byte copying is extremely inefficient for large blocks. The current lowering uses `MemoryCopy` and `MemoryFill` intrinsics that are expanded to byte-by-byte loops.

### Recommendation

1.  ✅ **Correctness**: ~~Implement overlap detection and a backward copy loop~~ - **DONE**
2.  **Optimize Performance**: Use word-sized (64-bit) loads/stores for the bulk of the copy/fill, handling unaligned heads/tails separately. This would significantly improve performance for large memory operations while maintaining correctness.

---

## Flaw 3: Redundant Stack Frame Reservation

**Severity**: 🟢 Low
**Location**: `crates/wasm-pvm/src/llvm_backend/lowering.rs`

### Problem Description

The compiler allocates stack slots for *all* LLVM values, even those that are short-lived or dead. It also reserves space for all function parameters in the frame, even if they could stay in registers.

### Recommendation

Liveness analysis in the backend would allow reusing slots and reducing frame size.

---

## Resolved Flaws (from V1 Review)

| Old Flaw | Status | Note |
|----------|--------|------|
| **No IR** | ✅ Fixed | Now uses LLVM IR |
| **Monolithic CodeGen** | ✅ Fixed | Split into `llvm_frontend` and `llvm_backend` |
| **Manual Control Flow** | ✅ Fixed | Frontend uses LLVM CFG; Backend handles labels properly |
| **Hardcoded Layout** | ✅ Fixed | `memory_layout.rs` provides constants |
| **Fragile Spilling** | ✅ Fixed | Slot-based approach is robust (if slow) |
| **No Validation** | ✅ Fixed | `wasmparser::validate` is called |
