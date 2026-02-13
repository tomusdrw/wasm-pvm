# WASM→PVM Compiler Architecture Review (V2)

**Compiler Expert Assessment**  
**Date**: 2026-02-13  
**Scope**: Comprehensive architectural review of the rewritten WASM-to-PVM recompiler  
**Status**: **Architecture Fixed, Correctness Issues Remain**

---

## Executive Summary

Since the previous review (2026-02-09), the codebase has undergone a **complete architectural rewrite**. The compiler now properly uses **LLVM IR** as an intermediate representation, addressing the fundamental "No IR" flaw. The codebase is now structured, modular, and extensible.

However, a critical correctness issue regarding **`memory.copy` overlap** remains unaddressed. Division safety checks and stack depth limits are intentionally delegated to PVM semantics/gas.

### Key Findings at a Glance

| Category | Status | Summary |
|----------|--------|---------|
| **Architecture** | 🟢 **Excellent** | Proper frontend (LLVM) / backend split. Clean orchestration. |
| **Correctness** | 🟡 **Medium Risk** | `memory.copy` corrupts overlapping data. Div/Stack checks delegated. |
| **Security** | 🟢 **Good** | Validation added. Resource exhaustion handled by Gas. |
| **Code Quality** | 🟢 **Good** | Modular, readable, typed. |
| **Performance** | 🟡 **Suboptimal** | Slot-based backend generates excessive memory traffic. |

---

## Major Changes Since V1

1.  **LLVM Integration**: Uses `inkwell` to translate WASM to LLVM IR.
2.  **Validation**: Now validates WASM input using `wasmparser`.
3.  **Stack Safety**: Prologue checks for frame overflow; recursion limited by gas.
4.  **Import Handling**: Imports with return values are now handled correctly (dummy values).

---

## Critical Action Items

1.  **Fix `memory.copy`**: Implement backward copying for overlapping regions where `dst > src`.
2.  **Optimize Backend**: Implement a register allocator to replace the current slot-based approach.

**Note**: Division checks and explicit recursion limits are NOT required (delegated to PVM).

---

## Detailed Reports

1.  [01-architectural-flaws.md](./findings/01-architectural-flaws.md)
2.  [02-correctness-issues.md](./findings/02-correctness-issues.md)
3.  [03-missing-features.md](./findings/03-missing-features.md)
4.  [04-code-quality.md](./findings/04-code-quality.md)
5.  [05-performance.md](./findings/05-performance.md)
