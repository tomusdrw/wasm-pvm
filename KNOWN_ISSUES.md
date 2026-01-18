# Known Issues

This document tracks known issues, bugs, and improvements for future work. Items here can be converted to GitHub issues when appropriate.

---

## Compiler Limitations

### No data section initialization
**Severity**: High  
**Status**: Open (Blocks V1 milestone)

WASM data sections are ignored during compilation. Programs that rely on initialized memory (e.g., string literals, lookup tables) will have uninitialized memory.

**Impact**: Complex programs like anan-as that use `(data ...)` sections will not work correctly.

**Error**: No error - data is silently ignored.

**Fix needed**: 
1. Parse WASM `DataSection` in `translate/mod.rs`
2. Initialize data in SPI `rw_data` section at correct offsets
3. Support active data segments with offset expressions

---

### No stack overflow detection
**Severity**: Medium  
**Status**: Open

Deep recursion can corrupt memory without any error. The call stack grows downward from 0xFEFE0000 but there's no guard to prevent it from overwriting other memory regions.

**Impact**: Recursive programs with deep call stacks may corrupt globals or heap.

**Workaround**: Avoid deep recursion; use iterative algorithms.

**Fix needed**: Add stack depth checking in call emission, emit TRAP on overflow.

---

### Operand stack limited to 5 slots
**Severity**: Low  
**Status**: Known limitation

The operand stack uses registers r2-r6 (5 slots). Complex expressions requiring more than 5 intermediate values will fail.

**File**: `crates/wasm-pvm/src/translate/stack.rs`

**Error**: `Stack overflow: max depth exceeded`

**Workaround**: Break complex expressions into smaller parts using locals.

**Fix needed**: Implement operand stack spilling to memory when depth exceeds register count.

---

### No floating point support
**Severity**: N/A  
**Status**: By design

PVM has no floating point instructions. WASM modules containing any float operations will be rejected at compile time.

**Error**: `Floating point operations not supported`

**Workaround**: Use fixed-point arithmetic or integer-only algorithms.

---

## Test Infrastructure

### `getMemory()` doesn't return bytes in test runner
**File**: `scripts/run-spi.ts`  
**Severity**: Low  
**Status**: Open

The `getMemory()` call from anan-as doesn't return the actual memory bytes in the test output. Results are verified via globals instead.

**Current behavior**:
```
=== Return Value ===
  Address: 0x30100
  Length: 4 bytes
```

**Workaround**: Results are verified by reading globals $result_ptr and $result_len.

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

### ~~`if/else/end` control flow not implemented~~ (Resolved 2025-01-17)
**Resolution**: Implemented in Phase 3. Uses `BRANCH_EQ_IMM` for condition, `JUMP` for else branch, proper label management.

---

### ~~Limited local variable count (4 max)~~ (Resolved 2025-01-17)
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x30200 + func_idx * 512 + (local_idx - 4) * 8`.

---

### ~~Function calls not implemented~~ (Resolved 2025-01-17)
**Resolution**: Implemented `call` instruction with:
- Jump table for return addresses (PVM requires JUMP_IND targets in jump table)
- Caller saves return address (jump table index) in r0
- Arguments passed via callee's local registers (r9+)
- Return value in r1
- Proper function prologue/epilogue

---

### ~~Spilled locals memory fault~~ (Resolved 2025-01-17)
**Resolution**: Moved spilled locals from 0x30000 to 0x30200 (within heap area). Heap pages are now automatically calculated based on the number of functions to ensure enough space for spilled locals.

---

### ~~Missing i64 operations~~ (Resolved 2025-01-17)
**Resolution**: Implemented all i64 operations:
- i64.div_u, i64.div_s, i64.rem_u, i64.rem_s
- i64.ge_u, i64.ge_s, i64.le_u, i64.le_s
- i64.and, i64.or, i64.xor
- i64.shl, i64.shr_u, i64.shr_s
- i64.load, i64.store

---

### ~~No recursion support~~ (Resolved 2025-01-18)
**Resolution**: Implemented proper call stack with frame management:
- Save/restore operand stack values across calls
- Save/restore locals (r9-r12) to call stack
- Dynamic frame size based on operand stack depth
- Spilled locals (for functions with >4 locals) still use fixed memory, but register locals are saved

---

### ~~No `call_indirect` support~~ (Resolved 2025-01-18)
**Resolution**: Implemented indirect function calls via table:
- Parse WASM table and element sections
- Build function table in RO memory at 0x10000
- Dispatch table maps indices to jump table references
- `call_indirect` performs table lookup + indirect jump

---

### ~~`if/else/end` control flow not implemented~~ (Resolved 2025-01-17)
**Resolution**: Implemented in Phase 3. Uses `BRANCH_EQ_IMM` for condition, `JUMP` for else branch, proper label management.

---

### ~~Limited local variable count (4 max)~~ (Resolved 2025-01-17)
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x30200 + func_idx * 512 + (local_idx - 4) * 8`.

---

### ~~Function calls not implemented~~ (Resolved 2025-01-17)
**Resolution**: Implemented `call` instruction with:
- Jump table for return addresses (PVM requires JUMP_IND targets in jump table)
- Caller saves return address (jump table index) in r0
- Arguments passed via callee's local registers (r9+)
- Return value in r7
- Proper function prologue/epilogue

---

### ~~Spilled locals memory fault~~ (Resolved 2025-01-17)
**Resolution**: Moved spilled locals from 0x30000 to 0x30200 (within heap area). Heap pages are now automatically calculated based on the number of functions to ensure enough space for spilled locals.

---

### ~~Missing i64 operations~~ (Resolved 2025-01-17)
**Resolution**: Implemented all i64 operations:
- i64.div_u, i64.div_s, i64.rem_u, i64.rem_s
- i64.ge_u, i64.ge_s, i64.le_u, i64.le_s
- i64.and, i64.or, i64.xor
- i64.shl, i64.shr_u, i64.shr_s
- i64.load, i64.store

---

### ~~LICENSE file missing~~ (Resolved 2025-01-18)
**Resolution**: Added MIT LICENSE file.

---

### ~~Block result values not supported~~ (Resolved 2025-01-18)
**Resolution**: Implemented block result value handling. Blocks, loops, and if/else now properly track and propagate result values. The stack depth is restored at block end, and `br` instructions copy the result value to the correct stack position.

---

### ~~No `br_table` support~~ (Resolved 2025-01-18)
**Resolution**: Implemented `br_table` instruction using a series of compare-and-branch instructions. Each table entry is compared with the index, and if matched, branches to the corresponding target. Out-of-bounds indices fall through to the default target.

