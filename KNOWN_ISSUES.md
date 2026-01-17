# Known Issues

This document tracks known issues, bugs, and improvements for future work. Items here can be converted to GitHub issues when appropriate.

---

## Test Infrastructure

### `getMemory()` doesn't return bytes in test runner
**File**: `scripts/run-spi.ts`  
**Severity**: Low  
**Status**: Open

The `getMemory()` call from anan-as doesn't return the actual memory bytes in the test output. The "Return Value" section shows address and length but no actual bytes.

**Current behavior**:
```
=== Return Value ===
  Address: 0x20100
  Length: 4 bytes
```

**Expected behavior**:
```
=== Return Value ===
  Address: 0x20100
  Length: 4 bytes
  Bytes: 0c 00 00 00
  As U32: 12
```

**Workaround**: Results can be verified via register values (r7=address, r8=length, and checking r11 for factorial shows 120).

**Fix needed**: Investigate anan-as `getMemory()` API usage or implement memory read differently.

---

## Compiler

### `if/else/end` control flow not implemented
**Status**: Planned (Phase 5)

WASM `if/else/end` instructions are not yet translated. Programs using these will fail with "Unsupported" error.

**Workaround**: Use `block`/`loop`/`br_if` pattern instead.

---

### Block result values not supported
**Status**: Planned (Phase 5)

WASM blocks that produce values (e.g., `(block (result i32) ...)`) are not handled correctly.

**Workaround**: Use explicit locals instead of block results.

---

### Limited local variable count
**Status**: Known limitation

Only 4 local variables are supported (registers r9-r12). Programs with more locals will fail.

**File**: `crates/wasm-pvm/src/translate/codegen.rs` (`MAX_LOCAL_REGS = 4`)

**Fix needed**: Implement register spilling to stack memory.

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
   - **File**: Where the issue is (if applicable)
   - **Severity**: Critical / High / Medium / Low
   - **Status**: Open / In Progress / Planned (Phase X)
   - Description of the problem
   - Expected vs actual behavior
   - Workaround (if any)
   - Fix needed

2. When ready to work on it, create a GitHub issue and link it here.

---

## Resolved Issues

*Move resolved issues here with date and resolution.*

<!-- Example:
### ~~Issue title~~ (Resolved 2025-01-17)
**Resolution**: Brief description of how it was fixed.
-->
