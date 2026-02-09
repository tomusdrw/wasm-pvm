# Known Issues

This document tracks known issues, bugs, and improvements for future work. Items here can be converted to GitHub issues when appropriate.

---

## Compiler Limitations

### ~~No stack overflow detection~~ (RESOLVED - Phase 13)


**Severity**: Medium
**Status**: ✅ Resolved (2025-01-19)

Deep recursion was causing silent memory corruption.
**Resolution**: Implemented stack limit checks in function prologues. Emits TRAP on overflow.
**Verification**: Code at `codegen.rs:390-424` (emit_call) and `codegen.rs:597-639` (emit_call_indirect) shows proper stack limit checks using unsigned comparison.

---

### ~~`memory.copy` incorrect for overlapping regions~~ (RESOLVED - 2026-02-06)

**Severity**: High
**Status**: ✅ Resolved (2026-02-06)

The implementation was updated to include a backward copy path when `dest > src`. When regions overlap with `dest > src`, the copy starts from the end and works backward (like `memmove`). See `codegen.rs:2551-2672`.

---

### ~~Division overflow checks missing~~ (RESOLVED - 2026-02-09)

**Severity**: Medium
**Status**: ✅ Resolved (2026-02-09)

Added WASM-spec-compliant division checks:
- **Div-by-zero check**: All 8 div/rem operations (i32/i64, signed/unsigned) now emit `BranchNeImm + Trap` before the division instruction.
- **INT_MIN / -1 overflow**: `i32.div_s` and `i64.div_s` now check for signed overflow. For i64, uses `LoadImm64 + Xor` approach since `i64::MIN` doesn't fit in a 32-bit immediate. Optimized with fast path that skips the check when divisor != -1.
- **Regression tests**: 8 tests in `tests/division_checks.rs` verify all check behaviors.

---

### ~~Import return values ignored~~ (RESOLVED - 2026-02-09)

**Severity**: Medium
**Status**: ✅ Resolved (2026-02-09)

Imported function stubs now push a dummy value (0) when the import signature has a return type, maintaining stack balance for callers.

**Regression tests**: 4 tests in `tests/import_returns.rs` verify import handling.

---

### Passive data segments (`memory.init`) not supported

**Severity**: Low
**Status**: 🔵 Known Limitation (Verified 2025-01-30)

Only active data segments (initialized at instantiation) are supported. Passive segments and `memory.init` instruction are not implemented.

**Workaround**: Use active segments or manual initialization code.

**Verification**: Source inspection at `translate/mod.rs:177` explicitly states: "Passive data segments are ignored for now (used with memory.init)".

---

### No floating point support
**Severity**: N/A  
**Status**: By design

PVM has no floating point instructions. WASM modules containing any float operations will be rejected at compile time (except for dead code paths where stubs are used).

**Error**: `Floating point operations not supported`

**Workaround**: Use fixed-point arithmetic or integer-only algorithms.

**Note**: Float truncation operations (`i32.trunc_sat_f64_u`, etc.) are stubbed to return 0. This allows compilation of modules with float operations in dead code paths (like anan-as), but calling these operations will produce incorrect results.

---

## Documentation

*No open documentation issues.*

---

## How to Add New Issues

When you discover a new issue:

1. Add it to this file with:
   - **Severity**: Critical / High / Medium / Low
   - **Status**: Open / In Progress / Planned (Phase X) / Known limitation
   - Description of the problem
   - Error message (if applicable)
   - Workaround (if any)
   - Fix needed

2. When ready to work on it, create a GitHub issue and link it here.

---

## Resolved Issues

### ~~Stack overflow detection~~ (Resolved 2025-01-19)
**Resolution**: Implemented stack overflow detection in function prologues:

- Calculate `new_sp = sp - frame_size` before each call
- Compare against `stack_limit = STACK_SEGMENT_END - stack_size` using unsigned comparison
- Emit TRAP on overflow (causes PANIC status)
- Default 64KB stack limit allows ~1600 recursion depth with 40-byte frames
- Verification: `codegen.rs:390-424` (emit_call), `codegen.rs:597-639` (emit_call_indirect)

---

### ~~`if/else/end` control flow not implemented~~ (Resolved 2026-01-17)
**Resolution**: Implemented in Phase 3. Uses `BRANCH_EQ_IMM` for condition, `JUMP` for else branch, proper label management.

---

### ~~Limited local variable count (4 max)~~ (Resolved 2026-01-17)
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x30200 + func_idx * 512 + (local_idx - 4) * 8`.

---

### ~~Function calls not implemented~~ (Resolved 2026-01-17)
**Resolution**: Implemented `call` instruction with:
- Jump table for return addresses (PVM requires JUMP_IND targets in jump table)
- Caller saves return address (jump table index) in r0
- Arguments passed via callee's local registers (r9+)
- Return value in r7
- Proper function prologue/epilogue

---

### ~~Spilled locals memory fault~~ (Resolved 2026-01-17)
**Resolution**: Moved spilled locals from 0x30000 to 0x30200 (within heap area). Heap pages are now automatically calculated based on the number of functions to ensure enough space for spilled locals.

---

### ~~Missing i64 operations~~ (Resolved 2026-01-17)
**Resolution**: Implemented all i64 operations:
- i64.div_u, i64.div_s, i64.rem_u, i64.rem_s
- i64.ge_u, i64.ge_s, i64.le_u, i64.le_s
- i64.and, i64.or, i64.xor
- i64.shl, i64.shr_u, i64.shr_s
- i64.load, i64.store

---

### ~~No recursion support~~ (Resolved 2026-01-18)
**Resolution**: Implemented proper call stack with frame management:
- Save/restore operand stack values across calls
- Save/restore locals (r9-r12) to call stack
- Dynamic frame size based on operand stack depth
- Spilled locals (for functions with >4 locals) still use fixed memory, but register locals are saved

---

### ~~No `call_indirect` support~~ (Resolved 2026-01-18)
**Resolution**: Implemented indirect function calls via table:
- Parse WASM table and element sections
- Build function table in RO memory at 0x10000
- Dispatch table maps indices to jump table references
- `call_indirect` performs table lookup + indirect jump

---

### ~~`if/else/end` control flow not implemented~~ (Resolved 2026-01-17)
**Resolution**: Implemented in Phase 3. Uses `BRANCH_EQ_IMM` for condition, `JUMP` for else branch, proper label management.

---

### ~~Limited local variable count (4 max)~~ (Resolved 2026-01-17)
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x30200 + func_idx * 512 + (local_idx - 4) * 8`.

---

### ~~Function calls not implemented~~ (Resolved 2026-01-17)
**Resolution**: Implemented `call` instruction with:
- Jump table for return addresses (PVM requires JUMP_IND targets in jump table)
- Caller saves return address (jump table index) in r0
- Arguments passed via callee's local registers (r9+)
- Return value in r1
- Proper function prologue/epilogue

---

### ~~Spilled locals memory fault~~ (Resolved 2026-01-17)
**Resolution**: Moved spilled locals from 0x30000 to 0x30200 (within heap area). Heap pages are now automatically calculated based on the number of functions to ensure enough space for spilled locals.

---

### ~~Missing i64 operations~~ (Resolved 2026-01-17)
**Resolution**: Implemented all i64 operations:
- i64.div_u, i64.div_s, i64.rem_u, i64.rem_s
- i64.ge_u, i64.ge_s, i64.le_u, i64.le_s
- i64.and, i64.or, i64.xor
- i64.shl, i64.shr_u, i64.shr_s
- i64.load, i64.store

---

### ~~LICENSE file missing~~ (Resolved 2026-01-18)
**Resolution**: Added MIT LICENSE file.

---

### ~~Block result values not supported~~ (Resolved 2026-01-18)
**Resolution**: Implemented block result value handling. Blocks, loops, and if/else now properly track and propagate result values. The stack depth is restored at block end, and `br` instructions copy the result value to the correct stack position.

---

### ~~No `br_table` support~~ (Resolved 2026-01-18)
**Resolution**: Implemented `br_table` instruction using a series of compare-and-branch instructions. Each table entry is compared with the index, and if matched, branches to the corresponding target. Out-of-bounds indices fall through to the default target.

---

### ~~No data section initialization~~ (Resolved 2026-01-19)
**Resolution**: Implemented WASM data section parsing and initialization. Data segments are placed in the SPI `rw_data` section at WASM_MEMORY_BASE (0x50000). Active data segments with offset expressions are supported.

---

### ~~Operand stack limited to 5 slots~~ (Resolved 2026-01-19)
**Resolution**: Implemented operand stack spilling to memory when depth exceeds 5. Fixed bugs in spill/restore logic for function calls and `local.tee` operations. Game of Life now works correctly with any number of steps.

---

### ~~Game of Life multi-step execution fault~~ (Resolved 2026-01-19)
**Resolution**: Fixed three bugs:
1. `I64Load` instruction was using invalid patterns
2. Spilled operand stack across function calls was reading from r7 instead of spill area
3. `local.tee` with spilled operand stack didn't check `pending_spill` and used wrong temp registers

---

### ~~Import function calls not supported~~ (Resolved 2026-01-19)
**Resolution**: Implemented import function stubbing. Imported functions pop their arguments and:
- `abort` emits TRAP
- Other imports are no-ops (useful for console.log, etc.)

This allows compilation of anan-as which imports `abort` and `console.log`.

