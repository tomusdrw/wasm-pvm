# 02 - Correctness Issues and Bugs

**Category**: Correctness Defects  
**Impact**: Potential data corruption, crashes, spec violations  
**Status**: LLVM backend needs edge case verification

---

## Summary

The LLVM-based compiler passes 360+ integration tests, demonstrating solid correctness for common cases. However, **edge cases require verification** to ensure full WASM spec compliance.

**Critical Areas for Verification**:

| Issue | Status | Priority |
|-------|--------|----------|
| Division overflow checks | 🔴 Needs verification | High |
| Memory.copy overlapping regions | 🔴 Needs verification | High |
| Import return values | 🟡 Needs verification | Medium |
| BrTable edge cases | 🟢 Likely handled by LLVM | Low |

---

## Critical Issues (Needs Verification)

### Issue 1: Division Overflow Checking 🔴

**Status**: **NEEDS VERIFICATION**  
**WASM Spec**: Requires trap for:
1. Division by zero
2. `INT_MIN / -1` overflow

**LLVM Behavior**:
- LLVM's `sdiv`/`udiv` instructions produce `poison` values on overflow
- Does not automatically trap per WASM semantics
- Frontend must emit explicit checks

**Evidence Needed**:
```rust
// llvm_frontend/function_builder.rs
// Does the frontend emit checks before division?

// Should generate:
// 1. Check divisor == 0 → trap if true
// 2. Check dividend == INT_MIN && divisor == -1 → trap if true  
// 3. Emit division instruction
```

**Impact**: 
- Non-compliant with WASM spec
- Silent wrong answers instead of traps
- Could cause security issues

**Test Case**:
```wat
(module
  (func (export "div_by_zero") (result i32)
    i32.const 42
    i32.const 0
    i32.div_s)  ;; Should trap, not return
  
  (func (export "int_min_overflow") (result i32)
    i32.const 0x80000000  ;; INT_MIN
    i32.const -1
    i32.div_s))  ;; Should trap per WASM spec
```

**Recommendation**: 
- Verify current behavior
- Add explicit overflow checks if missing
- Add targeted tests

---

### Issue 2: Memory.copy Overlapping Regions 🔴

**Status**: **NEEDS VERIFICATION**  
**WASM Spec**: `memory.copy` requires `memmove` semantics (handle overlapping regions)

**Implementation**:
- Handled via PVM intrinsics in LLVM frontend
- Lowered in backend to forward/backward copy paths

**Evidence Needed**:
```rust
// llvm_backend/lowering.rs
// Does the backend generate both forward and backward copy paths?
// Does it correctly detect overlap and choose appropriate path?
```

**Test Case**:
```wat
(module
  (memory 1)
  
  ;; Initialize memory: [A, B, C, D, E, F, G, H, I, J]
  (data (i32.const 0) "ABCDEFGHIJ")
  
  (func (export "test_overlap") (result i32)
    ;; Copy 5 bytes from addr 0 to addr 3 (overlap: dest > src)
    ;; Before: ABCDEFGHIJ
    ;; After:  ABCABCFGHJ
    i32.const 3    ;; dest
    i32.const 0    ;; src  
    i32.const 5    ;; len
    memory.copy
    
    ;; Check result at addr 3 should be 'A', not 'D'
    i32.const 3
    i32.load8_u))  ;; Should return 65 ('A')
```

**Recommendation**:
- Create explicit overlap test
- Verify backward copy path works correctly

---

### Issue 3: Import Return Values 🟡

**Status**: **NEEDS VERIFICATION**  
**Severity**: Medium

**Description**: Imported functions are stubbed. When an import has a return value, the stub must push a dummy value to maintain stack balance.

**Evidence Needed**:
```rust
// llvm_backend/lowering.rs
// How are imported functions handled in call_indirect lowering?
// Does the stub push a return value when has_return is true?
```

**Test Case**:
```wat
(module
  (import "env" "get_value" (func $import (result i32)))
  
  (func (export "test_import_return") (result i32)
    call $import
    ;; Should have a value on stack (even if dummy 0)
    i32.const 5
    i32.add))  ;; Should work without stack underflow
```

**Recommendation**:
- Verify import handling in LLVM backend
- Ensure dummy values pushed for return types

---

## Potential Issues

### Issue 4: LLVM Integer Poison Values 🔴

**Severity**: High (if division unchecked)  
**Location**: `llvm_frontend/function_builder.rs`

**Concern**: LLVM produces `poison` values for undefined operations. The frontend must add explicit checks to ensure WASM-compliant trapping behavior.

**Affected Operations**:
- `i32.div_s` / `i64.div_s` (overflow cases)
- `i32.div_u` / `i64.div_u` (div-by-zero)
- `i32.rem_s` / `i64.rem_s` (similar issues)

**Mitigation**: Add branch-on-condition before each division to check for edge cases.

---

### Issue 5: PVM Intrinsic Lowering Correctness 🟡

**Severity**: Medium  
**Location**: `llvm_backend/lowering.rs`

**Concern**: The LLVM frontend declares PVM intrinsics (e.g., `__pvm_memory_copy`). The backend must:
1. Recognize each intrinsic call
2. Lower to correct PVM instruction sequence
3. Handle all parameter combinations

**Risk**: If backend misses an intrinsic, it could:
- Generate call to non-existent function
- Silently drop the operation
- Produce incorrect code

**Intrinsics to Verify**:
- `__pvm_load_i32/i64` and variants
- `__pvm_store_i32/i64` and variants
- `__pvm_memory_size` / `__pvm_memory_grow`
- `__pvm_memory_fill` / `__pvm_memory_copy`
- `__pvm_call_indirect`

---

### Issue 6: SSA Value Slot Allocation Overflow 🟡

**Severity**: Low-Medium  
**Location**: `llvm_backend/lowering.rs:188-193`

**Code**:
```rust
fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
    let offset = self.next_slot_offset;
    self.value_slots.insert(key, offset);
    self.next_slot_offset += 8;  // Fixed 8-byte increment
    offset
}
```

**Concern**:
1. No maximum slot limit - could overflow frame size
2. Fixed 8-byte slots waste space for i32 values
3. Large functions could exhaust stack space

**Risk**: Stack overflow in functions with many values.

**Recommendation**: Add bounds checking and consider variable slot sizes.

---

## Verification Strategy

### Priority 1: Critical Edge Cases

Create explicit test cases for:

```rust
// Test 1: Division by zero
#[test]
fn test_div_by_zero_traps() {
    let wasm = wat!(r#"
        (module
          (func (export "test") (result i32)
            i32.const 42
            i32.const 0
            i32.div_s))
    "#);
    
    let result = execute(&wasm);
    assert!(matches!(result, ExecutionResult::Trap));
}

// Test 2: INT_MIN / -1 overflow
#[test]
fn test_int_min_div_minus_one_traps() {
    let wasm = wat!(r#"
        (module
          (func (export "test") (result i32)
            i32.const 0x80000000
            i32.const -1
            i32.div_s))
    "#);
    
    let result = execute(&wasm);
    assert!(matches!(result, ExecutionResult::Trap));
}

// Test 3: Memory copy overlap
#[test]
fn test_memory_copy_overlap() {
    let wasm = wat!(r#"
        (module
          (memory 1)
          (data (i32.const 0) "ABCDEFGHIJ")
          (func (export "test") (result i32)
            i32.const 3
            i32.const 0
            i32.const 5
            memory.copy
            i32.const 3
            i32.load8_u))
    "#);
    
    let result = execute(&wasm);
    assert_eq!(result, ExecutionResult::Value(65)); // 'A'
}
```

### Priority 2: Fuzzing

Use `wasm-smith` to generate random WASM modules and verify:
1. Compilation succeeds
2. Execution matches reference (wasmtime)
3. No crashes or hangs

### Priority 3: Differential Testing

Compare execution results against `wasmtime` for:
- All 360 integration tests
- Additional edge case tests
- Fuzz-generated tests

---

## Summary

| Issue | Status | Priority | Action |
|-------|--------|----------|--------|
| Division overflow | 🔴 Verify | High | Add checks, test |
| Memory.copy overlap | 🔴 Verify | High | Test overlap cases |
| Import returns | 🟡 Verify | Medium | Check stub handling |
| LLVM poison values | 🔴 Risk | High | Ensure traps added |
| Intrinsic lowering | 🟡 Verify | Medium | Verify all lowered |
| Slot allocation | 🟡 Monitor | Low | Add bounds check |

**Immediate Actions**:
1. Verify division overflow handling in LLVM frontend
2. Create explicit overlap test for memory.copy
3. Run differential tests against wasmtime
4. Add fuzzing to catch edge cases

---

*Review conducted 2026-02-10*
