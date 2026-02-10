# Known Issues (Post-Review)

**Version**: 2.0
**Date**: 2026-02-10
**Status**: Updated with architecture review findings and recent fixes

---

## How to Use This Document

- **Open**: Actively causing problems, needs immediate fix
- **Verified Fixed**: Fixed and tested
- **Known Limitation**: By design or low priority

---

## Critical Issues

### BUG-1: memory.copy Overlapping Regions - Verified Fixed

**Status**: VERIFIED FIXED (backward copy path exists and is correct)
**Severity**: High -> Critical
**Updated**: 2026-02-09 - Verified code at codegen.rs:2551-2672

The `memory.copy` now has backward copy path when `dest > src`. When regions overlap with `dest > src`, the copy starts from the end and works backward (like `memmove`).

**Remaining concern**: After fix, PVM-in-PVM inner interpreter still PANICs. The memory.copy fix alone doesn't resolve the PVM-in-PVM issue (see BUG-4).

---

### BUG-2: Division Overflow Checks Missing - Fixed

**Status**: FIXED (2026-02-09)
**Severity**: Medium -> High (WASM spec violation)

All 8 div/rem ops now have div-by-zero checks (`BranchNeImm + Trap`). `DivS32`/`DivS64` have `INT_MIN/-1` overflow checks. For i64, uses `LoadImm64 + Xor` approach since `i64::MIN` doesn't fit in a 32-bit immediate. 8 regression tests in `tests/division_checks.rs`.

---

### BUG-3: Import Return Values Ignored - Fixed

**Status**: FIXED (2026-02-09)
**Severity**: Medium

Imported function stubs now push a dummy value (0) when the import signature has a return type, maintaining stack balance. 4 regression tests in `tests/import_returns.rs`.

---

## High Priority Issues

### BUG-4: PVM-in-PVM Inner Interpreter PANIC - Open

**Status**: Under Investigation
**Severity**: High (Blocks V1)
**Updated**: 2026-02-09

#### Current State

- Inner program args page at 0xFEFF0000 is correct
- All direct tests pass
- Inner interpreter PANICs at PC 56 with exitCode 0
- r3=0, r4=0 (should be non-zero if args loaded correctly)

ExitCode 0 means memory fault at address < 0x10000 (reserved memory).

**Possible Causes**:
1. Register corruption (r7 modified, should be 0xFEFF0000)
2. Memory.copy issue not fully resolved
3. Another memory corruption bug
4. Stack pointer corruption

**Next Steps**:
1. Add step-by-step tracing to see register changes
2. Verify all memory operations in inner interpreter
3. Check if inner program's Decoder uses unshift/memory.copy
4. Compare memory state between direct and PVM-in-PVM

---

### BUG-5: LocalTee with Spilled Stack Complexity - Tested

**Status**: Tested (2026-02-09) - mitigated with 8 test cases
**Severity**: High (Bug-prone)

The `local.tee` implementation has 3 levels of nesting and multiple edge cases. Code at `codegen.rs:1304-1360`. Previous bugs were fixed (Game of Life), and 8 test cases now cover the main paths.

---

## Medium Priority Issues

### ISSUE-6: No WASM Validation Phase - Fixed

**Status**: FIXED (2026-02-09)

Added `wasmparser::validate()` call as the first step in `compile()`. Invalid WASM modules now fail early with clear error messages.

---

### ISSUE-7: Suppressed Compiler Warnings - Fixed

**Status**: PARTIALLY FIXED (2026-02-09)

Dead code removed (`check_for_floats`, `is_float_op`), unused imports cleaned up, doc_markdown warnings fixed. Cast-related suppressions retained with explanatory comments. Zero clippy warnings remain.

---

### ISSUE-8: Hardcoded Memory Addresses - Fixed

**Status**: FIXED (2026-02-09)
**Files**: `translate/memory_layout.rs` (new)

All PVM memory address constants extracted into `memory_layout` module with ASCII art layout diagram. Constants include `GLOBAL_MEMORY_BASE`, `SPILLED_LOCALS_BASE`, `STACK_SEGMENT_END`, `EXIT_ADDRESS`, etc. Helper functions centralized there as well.

---

## Low Priority / Known Limitations

### ISSUE-9: Passive Data Segments (memory.init)

**Status**: Known Limitation
**Severity**: Low
**Workaround**: Use active segments

---

### ISSUE-10: Floating Point Support

**Status**: By Design (PVM Limitation)
**Note**: PVM has no FP instructions. WASM with floats is rejected (except stubs for dead code).

---

### ISSUE-11: No Intermediate Representation

**Status**: Technical Debt (V2)
**Severity**: Medium (for V2)
**Note**: Acknowledged as architectural debt. Deferred to V2.

---

## Resolved Issues (History)

| Issue | Fixed | Resolution |
|-------|-------|------------|
| Stack Overflow Detection | 2025-01-19 | Stack limit checks in function prologues |
| Spilled Locals Memory Fault | 2026-01-17 | Moved spilled locals from 0x30200 to 0x40000 |
| Local Variable Zero-Initialization | 2026-02-06 | Added LoadImm 0 for non-parameter locals |
| AS u8 Arithmetic Semantics | 2026-02-06 | Documented AS behavior (not a wasm-pvm bug) |
| memory.copy Forward Copy | 2026-02-06 | Added backward copy path for overlapping regions |
| Division Overflow | 2026-02-09 | Div-by-zero + INT_MIN/-1 checks for all 8 ops |
| Import Return Values | 2026-02-09 | Dummy value push for has_return imports |
| WASM Validation | 2026-02-09 | wasmparser::validate() added |
| Clippy Warnings | 2026-02-09 | Dead code removed, warnings fixed |
| Hardcoded Memory Addresses | 2026-02-09 | Extracted to memory_layout.rs |

---

## Action Items by Priority

### Immediate
1. Debug PVM-in-PVM PANIC issue (BUG-4)
2. Verify memory.copy fix handles all overlap cases

### Medium Term
3. Add comprehensive test coverage (stack spilling, edge cases)
4. Add fuzzing targets
5. Document register conventions thoroughly

### Long Term (V2)
6. Add Intermediate Representation
7. Refactor monolithic codegen.rs
8. Implement proper register allocator
9. Add optimizations (constant folding, DCE)

---

## Review Integration

This document incorporates findings from the architecture review (2026-02-09):
- BUG-2, BUG-3 from review/findings/02-correctness-issues.md
- BUG-5 complexity concern from review
- ISSUE-6, 7, 8 from review/findings/04-code-quality.md
- BUG-4 (PVM-in-PVM) from status.md

For complete details, see:
- review/findings/02-correctness-issues.md
- review/findings/04-code-quality.md
- review/proposals/07-testing-strategy.md
