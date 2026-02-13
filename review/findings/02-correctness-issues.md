# 02 - Correctness Issues and Bugs

**Category**: Correctness Defects
**Date**: 2026-02-13 (Updated for V2 Architecture)
**Status**: Critical Issues Remain

---

## Summary

The architecture rewrite fixed several correctness issues (stack overflow, import returns), but the `memory.copy` overlap bug remains, and division safety checks are still missing.

---

## Confirmed Bugs

### Bug 1: `memory.copy` Fails on Overlapping Regions ✅

**Status**: **Fixed** (Verified 2026-02-13)
**Severity**: High
**Location**: `crates/wasm-pvm/src/llvm_backend/lowering.rs:1934` (`emit_pvm_memory_copy`)

#### Problem
The backend previously lowered `memory.copy` to a simple forward loop, which corrupted data when `dst > src` (shifting right).

#### Fix Implemented
Implemented proper `memmove` semantics:
- Added a check for overlap: `if src < dst { backward_copy } else { forward_copy }`.
- **Forward Copy**: `dst++ = src++` (standard).
- **Backward Copy**: `dst += len-1; src += len-1; dst-- = src--`.
- Verified with new integration tests `tests/fixtures/wat/memory_copy_overlap.jam.wat`.

---

### Intended Behavior: Division/Remainder Checks Delegated to PVM 🔵

**Status**: **By Design**
**Severity**: Low (Accepting PVM Semantics)
**Location**: `crates/wasm-pvm/src/llvm_backend/lowering.rs`

#### Description
WASM requires `div_s` and `rem_s` to trap on division by zero and signed overflow (`INT_MIN / -1`). The compiler currently emits raw PVM instructions (`DivS32`, etc.) without explicit checks.

**Decision**: Explicit checks are omitted intentionally. We rely on PVM's native behavior and gas metering to handle these cases, or accept the divergence from strict WASM semantics (similar to the lack of Floating Point support). This reduces code size and complexity.

#### Impact
- Division by zero behavior depends on PVM (likely trap or defined result).
- Overflow behavior depends on PVM.

---

## Resolved Issues

### Fixed: Import Return Values ✅
**Previous**: Imports ignored return values.
**Current**: `lower_import_call` now pushes a dummy `0` value if the signature requires a return, ensuring stack consistency.

### Fixed: Stack Overflow Check ✅
**Previous**: Manual and fragile check.
**Current**: `emit_prologue` checks `SP - frame_size < stack_limit`. Note: Further stack depth limits or recursion limits are **delegated to PVM gas metering**. Infinite recursion will eventually run out of gas, so explicit recursion depth tracking is not implemented/needed.

### Fixed: Validation ✅
**Previous**: No validation.
**Current**: `WasmModule::parse` calls `wasmparser::validate` before processing.

---

## Potential Issues (New Architecture)

### Issue 1: `memory.fill` Byte-Loop Performance 🟡
**Location**: `lowering.rs:1885`
The `memory.fill` implementation is also a byte-by-byte loop. While correct (no overlap issues for fill), it is extremely slow for large fills.

### Issue 2: `phi` Cycle Handling 🟡
**Location**: `lowering.rs:1075` (`emit_phi_copies`)
The code handles phi cycles by using temp registers or stack slots. This looks correct but is complex and critical. If the "temp stack slots" (negative offsets from SP) overlap with anything else, it could be a bug. (Review indicates it uses space *below* the current frame, which should be safe as long as no other mechanism claims that space).
