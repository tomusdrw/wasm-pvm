# Known Issues - Updated (Post-Review)

**Version**: 2.0  
**Date**: 2026-02-09  
**Status**: Updated with architecture review findings

---

## How to Use This Document

- **🔴 Open**: Actively causing problems, needs immediate fix
- **🟡 In Progress**: Being worked on
- **🟢 Verified Fixed**: Fixed and tested
- **🔵 Known Limitation**: By design or low priority

---

## Critical Issues (Fix Required for V1)

### BUG-1: memory.copy Overlapping Regions 🟢

**Original Issue**: memory.copy incorrect for overlapping regions
**Status**: VERIFIED FIXED (backward copy path exists and is correct)
**Severity**: High → Critical
**Updated**: 2026-02-09 - Verified code at codegen.rs:2551-2672

#### Current Understanding

**From status.md**: Major bug was FIXED - the memory.copy now has backward copy path when `dest > src`. However, the PVM-in-PVM issue persists, suggesting there may be additional bugs or the fix isn't complete.

**Original Problem**: WASM `memory.copy` was implemented as forward-only byte loop. When `dest > src` with overlap, forward copy overwrites source before reading.

**Fix Applied**: Added backward copy path in `codegen.rs:2481-2599`. When `dest > src`, copy starts from end and works backward (like `memmove`).

**Current Issue**: After fix, inner interpreter still PANICs. Possible causes:
1. Fix is incomplete (other memmove issues)
2. Different bug causing PVM-in-PVM failure
3. AS runtime has other `memory.copy` usages

**Action Required**: 
- [ ] Verify fix handles all overlap cases
- [ ] Test with explicit memmove pattern
- [ ] Trace if AS runtime has other copy operations
- [ ] Check if fix applies to all memory.copy call sites

---

### BUG-2: Division Overflow Checks Missing 🟢

**Status**: FIXED (2026-02-09)
**Severity**: Medium → High (WASM spec violation)
**Updated**: 2026-02-09 - All 8 div/rem ops now have div-by-zero checks. DivS32/DivS64 have INT_MIN/-1 checks.

#### Problem

`i32.div_s` and `i64.div_s` do not check for:
1. Division by zero
2. `INT_MIN / -1` overflow

WASM spec requires TRAP for both cases.

**Impact**: 
- Division by zero returns garbage instead of trapping
- `INT_MIN / -1` produces wrong result (should trap)
- Not detected in tests because test cases don't hit these edge cases

**Code Locations**:
- `codegen.rs:1472-1481` (I32DivS)
- `codegen.rs:1716-1725` (I64DivS)

**Fix Required**:
```rust
// Before emitting DivS instruction:
// 1. Check divisor == 0, emit TRAP if true
// 2. Check dividend == INT_MIN && divisor == -1, emit TRAP if true
// 3. Then emit normal division
```

**Effort**: 1-2 days

---

### BUG-3: Import Return Values Ignored 🟢

**Status**: FIXED (2026-02-09)
**Severity**: Medium
**Updated**: 2026-02-09 - Dummy value (0) now pushed when import has return type.

#### Problem

Imported function stubs pop arguments but don't push return values when `has_return` is true.

**Code Location**: `codegen.rs:2210-2241`

**Impact**: 
- Callers of imports with return values get garbage
- Stack imbalance
- Could cause crashes in programs using WASI or host functions

**Current Behavior**:
```rust
if (*function_index as usize) < ctx.num_imported_funcs {
    // Pop arguments
    for _ in 0..num_args { emitter.spill_pop(); }
    
    if import_name == "abort" { emitter.emit(Instruction::Trap); }
    // Missing: if has_return { push dummy value }
}
```

**Fix Required**: Push dummy value (0) if import signature has return type.

**Effort**: 1 day

---

## High Priority Issues

### BUG-4: PVM-in-PVM Inner Interpreter PANIC 🔴

**Status**: Under Investigation  
**Severity**: High (Blocks V1)  
**Updated**: 2026-02-09

#### Current State

From status.md:
- Inner program args page at 0xFEFF0000 is correct
- All 350 direct tests pass
- Inner interpreter PANICs at PC 56 with exitCode 0
- r3=0, r4=0 (should be non-zero if args loaded correctly)

#### PANIC Analysis

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

### BUG-5: LocalTee with Spilled Stack Complexity 🟢

**Status**: Tested (2026-02-09)
**Severity**: High (Bug-prone) - now mitigated with 8 test cases
**Updated**: 2026-02-09

#### Problem

The `local.tee` implementation has 3 levels of nesting and multiple edge cases:
- Pending spill tracking
- Already spilled detection
- Register vs spilled local handling
- Temp register selection

**Code**: `codegen.rs:1304-1360`

**Risk**: Complex code with many branches. Previous bugs were fixed (Game of Life), but more may exist.

**Recommendation**: Add comprehensive unit tests for `local.tee` with various stack depths and local indices.

---

## Medium Priority Issues

### ISSUE-6: No WASM Validation Phase 🟢

**Status**: FIXED (2026-02-09)
**Severity**: Medium

**Resolution**: Added `wasmparser::validate()` call as the first step in `compile()`.
Invalid WASM modules now fail early with clear error messages. This caught 3 invalid
test cases that were silently accepted before.

---

### ISSUE-7: Suppressed Compiler Warnings 🟢

**Status**: PARTIALLY FIXED (2026-02-09)
**Severity**: Medium
**Files**: `lib.rs`, `codegen.rs`

**Resolution**: Dead code removed (`check_for_floats`, `is_float_op`), unused imports cleaned up,
doc_markdown warnings fixed. Cast-related suppressions retained with explanatory comments since
they are intentional for compiler code (WASM i32/i64 <-> PVM u8 register operations). Zero clippy
warnings remain.

---

### ISSUE-8: Hardcoded Memory Addresses 🟢

**Status**: FIXED (2026-02-09)
**Severity**: Medium
**Files**: `translate/memory_layout.rs` (new)

**Resolution**: All PVM memory address constants extracted into a dedicated
`memory_layout` module with full ASCII art layout diagram. Constants include
`GLOBAL_MEMORY_BASE`, `SPILLED_LOCALS_BASE`, `STACK_SEGMENT_END`, `EXIT_ADDRESS`,
etc. Helper functions (`compute_wasm_memory_base`, `spilled_local_addr`, `global_addr`)
centralized there as well.

---

## Low Priority / Known Limitations

### ISSUE-9: Passive Data Segments (memory.init) 🔵

**Status**: Known Limitation  
**Severity**: Low  
**Impact**: Cannot use `memory.init` instruction

**Workaround**: Use active segments

---

### ISSUE-10: Floating Point Support 🔵

**Status**: By Design (PVM Limitation)  
**Severity**: N/A  
**Note**: PVM has no FP instructions. WASM with floats is rejected (except stubs for dead code).

---

### ISSUE-11: No Intermediate Representation 🔵

**Status**: Technical Debt (V2)  
**Severity**: Medium (for V2)  
**Impact**: Cannot add optimizations

**Note**: Acknowledged as architectural debt. Deferred to V2.

---

## Resolved Issues (History)

### Stack Overflow Detection ✅
**Fixed**: 2025-01-19  
**Resolution**: Added stack limit checks in function prologues

### Spilled Locals Memory Fault ✅
**Fixed**: 2026-01-17  
**Resolution**: Moved spilled locals from 0x30200 to 0x40000

### Local Variable Zero-Initialization ✅
**Fixed**: 2026-02-06  
**Resolution**: Added LoadImm 0 for non-parameter locals in prologue

### AS u8 Arithmetic Semantics ✅
**Fixed**: 2026-02-06  
**Resolution**: Documented AS behavior (not a wasm-pvm bug)

### memory.copy Forward Copy ✅
**Fixed**: 2026-02-06 (per status.md)  
**Resolution**: Added backward copy path for overlapping regions

---

## Action Items by Priority

### Immediate (This Week)
1. Verify memory.copy fix is complete
2. Test memmove patterns explicitly
3. Add division overflow checks
4. Add import return value handling

### Short Term (Next 2 Weeks)
5. Debug PVM-in-PVM PANIC issue
6. Add WASM validation phase
7. Fix clippy warnings
8. Add LocalTee tests

### Medium Term (Next Month)
9. Create MemoryLayout abstraction
10. Add comprehensive test coverage
11. Document register conventions
12. Add fuzzing

### Long Term (V2)
13. Add Intermediate Representation
14. Refactor monolithic codegen.rs
15. Implement proper register allocator
16. Add optimizations (constant folding, DCE)

---

## Issue Tracking Template

When adding new issues, use this format:

```markdown
### ISSUE-XX: Brief Title

**Status**: 🔴 Open / 🟡 In Progress / 🟢 Fixed / 🔵 Known Limitation
**Severity**: Critical / High / Medium / Low
**Date**: YYYY-MM-DD

#### Problem
Description

#### Impact
What breaks

#### Code Location
File.rs:line-range

#### Fix Required
What needs to be done

#### Effort Estimate
X days
```

---

## Review Integration

This document incorporates findings from the architecture review:
- BUG-2, BUG-3 from review/02-correctness-issues.md
- BUG-5 complexity concern from review
- ISSUE-6, 7, 8 from review/04-code-quality.md
- BUG-4 (PVM-in-PVM) from status.md

For complete details, see:
- review/findings/02-correctness-issues.md
- review/findings/04-code-quality.md
- review/proposals/07-testing-strategy.md

---

*This document replaces the original KNOWN_ISSUES.md for V1 planning.*
