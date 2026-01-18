# Known Issues

This document tracks known issues, bugs, and improvements for future work. Items here can be converted to GitHub issues when appropriate.

---

## Compiler Limitations

### No recursion support
**Severity**: Medium  
**Status**: Known limitation (Phase 8)

Recursive function calls will not work correctly. Spilled locals use fixed memory addresses per function (at `0x20200 + func_idx * 512`), not a proper call stack. Each function gets 512 bytes for spilled locals.

**Impact**: Programs with recursive functions will corrupt their own local variables.

**Workaround**: Convert recursive algorithms to iterative using explicit loops.

**Fix needed**: Implement proper call stack with frame pointer, push/pop spilled locals on call/return.

---

### No `call_indirect` support
**Severity**: Medium  
**Status**: Planned (Phase 6+)

WASM `call_indirect` instruction (indirect function calls via table) is not implemented. Programs using function pointers or vtables will fail.

**Error**: `Unsupported instruction: CallIndirect`

**Workaround**: Use direct `call` instructions where possible.

**Fix needed**: Build function table from WASM table section, translate `call_indirect` to table lookup + indirect jump.

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
  Address: 0x20100
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
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x20200 + func_idx * 512 + (local_idx - 4) * 8`.

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
**Resolution**: Moved spilled locals from 0x30000 to 0x20200 (within heap area). Heap pages are now automatically calculated based on the number of functions to ensure enough space for spilled locals.

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

