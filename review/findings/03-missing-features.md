# 03 - Missing Features

**Category**: Feature Gaps
**Date**: 2026-02-13 (Updated for V2 Architecture)
**Status**: Improving

---

## Summary

The compiler implements the WASM MVP (Minimum Viable Product). Major missing features are intentional (Floating Point) or planned (Host Calls).

---

## Confirmed Missing Features

### 1. Floating Point Support ❌
**Status**: **Wontfix** (By Design)
PVM does not support floating point. The compiler intentionally rejects float operations.

### 2. Host Calls (`ecalli`) 🔵
**Status**: Planned
PVM `ecalli` instruction is not yet generated. Support is needed for WASI or generic host functions. Currently, imports trap.

### 3. Passive Data Segments (`memory.init`) 🟡
**Status**: Missing
Only active data segments are supported. `memory.init` and `data.drop` are not implemented in the frontend.

### 4. Bulk Memory Operations 🟡
**Status**: Partial
`memory.copy` and `memory.fill` are implemented (via loop lowerings), but `memory.init`, `table.init`, `table.copy` are missing.

---

## Feature Completeness Matrix

| Feature | Status | Notes |
|---------|--------|-------|
| **Core** | ✅ | i32/i64 arithmetic, control flow |
| **Validation** | ✅ | Uses `wasmparser::validate` |
| **Control Flow** | ✅ | Full LLVM CFG support |
| **Memory** | ✅ | Load/Store intrinsics |
| **Globals** | ✅ | Mapped to PVM memory |
| **Indirect Calls** | ✅ | Dispatch table + Type check |
| **Multi-Value** | ❌ | Not supported (Entry supports ptr/len) |
| **SIMD** | ❌ | Not supported |
| **Threads** | ❌ | Not supported |
| **Exception Handling**| ❌ | Not supported |

---

## Recommendations

1.  **Implement `ecalli`**: Map specific imports to `ecalli` instructions to enable host interaction.
2.  **Add `memory.init`**: Support passive data segments for bulk memory initialization.
