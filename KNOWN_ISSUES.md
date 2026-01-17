# Known Issues

This document tracks known issues, bugs, and improvements for future work. Items here can be converted to GitHub issues when appropriate.

---

## Compiler Limitations

### No recursion support
**Severity**: Medium  
**Status**: Known limitation (Phase 6+)

Recursive function calls will not work correctly. Spilled locals use fixed memory addresses per function (at `0x30000 + func_idx * 512`), not a proper call stack. Each function gets 512 bytes for spilled locals.

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

### Block result values not supported
**Severity**: Low  
**Status**: Planned (Phase 5)

WASM blocks that produce values (e.g., `(block (result i32) ...)`) are not handled correctly. The result value is not properly propagated.

**Workaround**: Use explicit locals instead of block results:
```wat
;; Instead of:
(block (result i32)
  (i32.const 42)
)

;; Use:
(local $tmp i32)
(block
  (local.set $tmp (i32.const 42))
)
(local.get $tmp)
```

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

### Spilled locals memory not in heap
**Severity**: Medium  
**Status**: Known limitation

Spilled locals are stored at 0x30000, which may be outside the allocated heap range in SPI format. Functions with more than 4 locals (including parameters) will cause FAULT when accessing spilled locals.

**Workaround**: Keep functions to 4 or fewer locals (including parameters).

**Fix needed**: Move spilled locals base address to within the heap area (e.g., 0x20200) or increase heap pages in SPI output.

---

### No `br_table` support
**Severity**: Low  
**Status**: Planned

WASM `br_table` instruction (switch/jump table) is not implemented.

**Error**: `Unsupported instruction: BrTable`

**Workaround**: Use nested `if/else` or `br_if` chains.

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

### LICENSE file missing
**Status**: Open

No LICENSE file in repository. README mentions MIT but file doesn't exist.

**Fix**: Add LICENSE file with appropriate license text.

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
**Resolution**: Implemented local spilling to memory. Locals 0-3 use registers r9-r12, locals 4+ are spilled to memory at `0x30000 + func_idx * 512 + (local_idx - 4) * 8`.

---

### ~~Function calls not implemented~~ (Resolved 2025-01-17)
**Resolution**: Implemented `call` instruction with:
- Jump table for return addresses (PVM requires JUMP_IND targets in jump table)
- Caller saves return address (jump table index) in r0
- Arguments passed via callee's local registers (r9+)
- Return value in r1
- Proper function prologue/epilogue

