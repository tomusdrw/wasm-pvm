# 02 - Correctness Issues and Bugs

**Category**: Correctness Defects  
**Impact**: Silent data corruption, crashes, security vulnerabilities

---

## Summary

While the compiler passes 62 integration tests, there are **known correctness bugs** and several **potential correctness issues** that could cause silent failures or crashes in edge cases.

---

## Confirmed Bugs (from KNOWN_ISSUES.md)

### Bug 1: memory.copy Fails on Overlapping Regions 🔴

**Status**: Open (Verified 2025-01-30)  
**Severity**: High  
**Location**: `codegen.rs:2481-2593`

#### Problem

The `memory.copy` implementation only performs forward copying:

```rust
// Forward copy loop
emitter.define_label(forward_loop);
emitter.emit_branch_eq_imm_to_label(size, 0, end);
emitter.emit(Instruction::LoadIndU8 { dst: temp, base: src, offset: 0 });
emitter.emit(Instruction::StoreIndU8 { base: dest, src: temp, offset: 0 });
emitter.emit(Instruction::AddImm32 { dst: dest, src: dest, value: 1 });  // dest++
emitter.emit(Instruction::AddImm32 { dst: src, src, value: 1 });         // src++
emitter.emit(Instruction::AddImm32 { dst: size, src: size, value: -1 }); // size--
emitter.emit_jump_to_label(forward_loop);
```

**WASM spec requires**: When `dest > src` and regions overlap, must copy backward (from end to start) to avoid overwriting source before reading.

**Current code analysis**: Looking more carefully at the code, there IS a backward copy path, but it's triggered incorrectly. The code checks `emit_branch_gtu(dest, src, backward_setup)` which should handle the backward case.

**However**, the bug report states this is still open. Let me verify by reading the code more carefully:

```rust
// Line 2499
emitter.emit_branch_gtu(dest, src, backward_setup);  // If dest > src, goto backward_setup
```

So there IS backward copy logic. The bug may be in the condition or the backward copy implementation itself.

Looking at the backward copy code:
```rust
// Lines 2544-2593
emitter.emit(Instruction::Fallthrough);
emitter.define_label(backward_setup);
// Add WASM_MEMORY_BASE to both
emitter.emit(Instruction::AddImm32 { ... });
// dest += size, src += size (go to end)
emitter.emit(Instruction::Add32 { dst: dest, src1: dest, src2: size });
emitter.emit(Instruction::Add32 { dst: src, src1: src, src2: size });
// backward_loop with pre-decrement
```

This looks correct in theory. The bug might be:
1. The condition for choosing forward vs backward is wrong
2. The WASM memory base is added in the wrong place (added in forward path at lines 2502-2511, but backward path also adds it at 2547-2556 - double addition?)

Wait, looking at line 2549-2556:
```rust
// === BACKWARD COPY (dest > src) ===
// backward_setup:
emitter.emit(Instruction::Fallthrough);
emitter.define_label(backward_setup);
emitter.emit(Instruction::AddImm32 { dst: dest, src: dest, value: ctx.wasm_memory_base });
emitter.emit(Instruction::AddImm32 { dst: src, src, value: ctx.wasm_memory_base });
```

And in the forward path (lines 2502-2511):
```rust
// === FORWARD COPY (dest <= src) ===
emitter.emit(Instruction::AddImm32 { dst: dest, src: dest, value: ctx.wasm_memory_base });
emitter.emit(Instruction::AddImm32 { dst: src, src, value: ctx.wasm_memory_base });
```

This looks correct - each path adds the base once.

**Potential Issue**: The backward copy does `dest += size` and `src += size` AFTER adding the base. If `dest` or `src` + `size` overflows, this could be a problem. But that's unlikely with proper bounds checking.

**Conclusion**: The bug report says this is open. Either the implementation has a subtle bug or the condition for choosing backward vs forward is incorrect. The code exists but may have logic errors.

#### Impact

- `memmove`-like operations corrupt data when `dest > src`
- Affects programs that use `memory.copy` with overlapping regions

#### Fix Required

Verify and fix the condition logic. The backward copy path exists but may not be triggered correctly or may have implementation bugs.

---

### Bug 2: Division Overflow Not Checked 🔴

**Status**: Open (Verified 2025-01-30)  
**Severity**: Medium  
**Location**: `codegen.rs:1462-1481` (I32DivS), `codegen.rs:1716-1725` (I64DivS)

#### Problem

Signed division has two edge cases that WASM requires to trap:
1. Division by zero
2. `INT_MIN / -1` (overflow)

Current implementation:
```rust
Operator::I32DivS => {
    let src2 = emitter.spill_pop();
    let src1 = emitter.spill_pop();
    let dst = emitter.spill_push();
    emitter.emit(Instruction::DivS32 { dst, src1: src2, src2: src1 });  // No checks!
}
```

#### WASM Spec Requirement

Both cases must cause a trap (unreachable/panic behavior).

#### Impact

- Division by zero returns garbage or crashes
- `INT_MIN / -1` produces wrong result

#### Fix Required

Add explicit checks before division:

```rust
Operator::I32DivS => {
    let divisor = emitter.spill_pop();
    let dividend = emitter.spill_pop();
    
    // Check divisor == 0
    let continue_label = emitter.alloc_label();
    emitter.emit(Instruction::BranchNeImm { 
        reg: divisor, 
        value: 0, 
        offset: /* to continue_label */ 
    });
    emitter.emit(Instruction::Trap);  // Division by zero
    
    // Check INT_MIN / -1
    emitter.define_label(continue_label);
    let not_overflow_label = emitter.alloc_label();
    emitter.emit(Instruction::LoadImm { reg: temp, value: i32::MIN });
    emitter.emit(Instruction::BranchNeImm { reg: dividend, value: i32::MIN, offset: /* skip */ });
    emitter.emit(Instruction::LoadImm { reg: temp, value: -1 });
    emitter.emit(Instruction::BranchNeImm { reg: divisor, value: -1, offset: /* skip */ });
    emitter.emit(Instruction::Trap);  // Overflow
    
    // Actual division
    emitter.define_label(not_overflow_label);
    let dst = emitter.spill_push();
    emitter.emit(Instruction::DivS32 { dst, src1: divisor, src2: dividend });
}
```

---

### Bug 3: Import Return Values Ignored 🔴

**Status**: Open (Verified 2025-01-30)  
**Severity**: Medium  
**Location**: `codegen.rs:2210-2241`

#### Problem

Imported functions are stubbed but don't push return values:

```rust
// Check if this is a call to an imported function
if (*function_index as usize) < ctx.num_imported_funcs {
    let import_name = ctx.imported_func_names.get(*function_index as usize)
        .map_or("unknown", String::as_str);
    
    // Pop arguments (they're on the stack)
    for _ in 0..num_args {
        emitter.spill_pop();
    }
    
    // Handle specific imports:
    if import_name == "abort" {
        emitter.emit(Instruction::Trap);
    }
    // For has_return, we'd need to push a dummy value, but abort/console.log don't return
}
```

#### Problem

If the import signature has a return value (`has_return = true`), the stub:
1. Pops arguments from stack ✓
2. Does NOT push a return value ✗

This causes stack imbalance and the caller reads garbage.

#### Impact

- Callers of imports that return values get wrong results
- Stack underflow may occur

#### Fix Required

Push dummy value (0) if import has return type:

```rust
if has_return {
    let dst = emitter.spill_push();
    emitter.emit(Instruction::LoadImm { reg: dst, value: 0 });
}
```

---

## Potential Correctness Issues (Code Review Findings)

### Issue 1: Stack Overflow Check Clobbers Registers 🔴

**Severity**: High  
**Location**: `codegen.rs:390-430`

#### Observation

The stack overflow check uses r7 (ARGS_PTR_REG) to hold the limit:

```rust
// NOTE: This clobbers r7 (ARGS_PTR_REG) with the limit value.
// We must:
// 1. Flush any pending spill (r7 may hold a deferred spill value at depth >= 5)
self.flush_pending_spill();
self.emit(Instruction::LoadImm64 { reg: ARGS_PTR_REG, value: u64::from(limit as u32) });
```

#### Risk

The comment acknowledges that r7 might hold a deferred spill value. If `flush_pending_spill()` doesn't properly save this value, data corruption occurs.

Looking at `flush_pending_spill()`:

```rust
fn flush_pending_spill(&mut self) {
    if let Some(spill_depth) = self.pending_spill.take() {
        let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(spill_depth);
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: StackMachine::reg_at_depth(spill_depth),  // Could be r7!
            offset,
        });
    }
}
```

**Potential Bug**: If `pending_spill` is for depth >= 5 (which uses r7 as SPILL_TEMP_REG), and then we load the limit into r7, the spill value is lost.

**Status**: The code does flush pending spills first, so this is handled. But this is fragile.

---

### Issue 2: I32 Comparison Normalization May Overflow 🔴

**Severity**: Medium  
**Location**: `codegen.rs:1756-1768`

#### Observation

The i32 comparison operations normalize operands using `AddImm32` with value 0:

```rust
Operator::I32GtU => {
    let b = emitter.spill_pop();
    let a = emitter.spill_pop();
    // Normalize both operands to sign-extended 32-bit
    emitter.emit(Instruction::AddImm32 { dst: a, src: a, value: 0 });
    emitter.emit(Instruction::AddImm32 { dst: b, src: b, value: 0 });
    let dst = emitter.spill_push();
    emitter.emit(Instruction::SetLtU { dst, src1: a, src2: b });
}
```

#### Risk

The comment says "sign-extends the result to i64" which is intentional for comparing upper bits. However:

1. This assumes `AddImm32` sign-extends, which it does
2. But what if the input register has garbage in upper 32 bits that affects subsequent operations?

This normalization is done IN-PLACE in register `a` and `b`. If these registers are still needed later with their original 64-bit values, they've been corrupted.

**However**, for comparisons, `a` and `b` are popped (consumed), so this is fine for comparisons.

**But what about I32GtS at line 1780?**
```rust
Operator::I32GtS => {
    let b = emitter.spill_pop();
    let a = emitter.spill_pop();
    emitter.emit(Instruction::AddImm32 { dst: a, src: a, value: 0 });
    emitter.emit(Instruction::AddImm32 { dst: b, src: b, value: 0 });
    // ...
}
```

Same pattern. Since `a` and `b` are popped, they're consumed. The normalization happens in the popped registers before they're overwritten by `spill_push()` later. This seems correct but is fragile.

---

### Issue 3: BrTable Out-of-Bounds Not Handled Correctly 🟡

**Severity**: Low-Medium  
**Location**: `codegen.rs:2114-2147`

#### Observation

`br_table` compares index against each case and falls through to default:

```rust
for (i, &depth) in target_depths.iter().enumerate() {
    if let Some((target, target_depth, has_result)) = emitter.get_branch_info(depth) {
        let next_label = emitter.alloc_label();
        emitter.emit_branch_ne_imm_to_label(index_reg, i as i32, next_label);
        // ... branch to target ...
        emitter.emit(Instruction::Fallthrough);
        emitter.define_label(next_label);
    }
}

// Default case
if let Some((target, target_depth, has_result)) = emitter.get_branch_info(default_depth) {
    // ... branch to target (no check, just falls through)
    emitter.emit_jump_to_label(target);
}
```

#### Risk

What if `index_reg` is greater than all case indices? The code falls through to the default case, which is correct.

What if `index_reg` is negative? The comparison `index_reg != i` (unsigned comparison?) vs signed matters. If `index_reg` is -1 (0xFFFFFFFF), comparing against small positive `i` values:

- If comparison is unsigned: 0xFFFFFFFF != 0, 0xFFFFFFFF != 1, etc. → falls through to default ✓
- If comparison is signed: -1 != 0, -1 != 1, etc. → falls through to default ✓

This seems okay. The `BranchNeImm` is likely doing a signed comparison, which handles negative values correctly.

---

### Issue 4: Call Indirect Signature Check Uses Wrong Type Index 🟡

**Severity**: Medium  
**Location**: `codegen.rs:857-878`

#### Observation

In `emit_call_indirect`, the signature validation loads the type index from the dispatch table:

```rust
// Load type index from dispatch table (at offset 4)
self.emit(Instruction::LoadIndU32 { dst: ARGS_PTR_REG, base: SAVED_TABLE_IDX_REG, offset: 4 });

// Validate type signature: compare with expected type index
let sig_ok_label = self.alloc_label();
self.emit(Instruction::BranchEqImm { reg: ARGS_PTR_REG, value: expected_type_index as i32, offset: 0 });
```

#### Risk

This assumes the dispatch table entry at offset 4 contains the type index. Looking at how the dispatch table is built in `mod.rs:455-479`:

```rust
for &func_idx in &function_table {
    if func_idx == u32::MAX {
        ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // jump address
        ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // type index
    } else if (func_idx as usize) < num_imported_funcs {
        ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // jump address
        ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // type index
    } else {
        let local_func_idx = func_idx as usize - num_imported_funcs as usize;
        let jump_ref = 2 * (func_entry_jump_table_base + local_func_idx + 1) as u32;
        ro_data.extend_from_slice(&jump_ref.to_le_bytes());
        let type_idx = *function_type_indices.get(local_func_idx).unwrap_or(&u32::MAX);
        ro_data.extend_from_slice(&type_idx.to_le_bytes());
    }
}
```

The type index IS stored at offset 4 (after the 4-byte jump address). This looks correct.

However, for invalid entries (u32::MAX), the type index is also u32::MAX. If someone calls an invalid index, the signature check will compare against the expected type, fail, and trap. This is correct behavior.

---

### Issue 5: Local Tee with Spilled Values May Corrupt Stack 🔴

**Severity**: High  
**Location**: `codegen.rs:1304-1360`

#### Observation

The `local.tee` implementation is complex when the operand stack top is spilled:

```rust
Operator::LocalTee { local_index } => {
    let idx = *local_index as usize;
    let stack_depth = emitter.stack.depth();

    // Get the source register for the top of stack
    let src = if stack_depth > 0 && StackMachine::needs_spill(stack_depth - 1) {
        // Check if there's a pending spill for this depth
        if emitter.pending_spill == Some(stack_depth - 1) {
            // Value is still in r7, not yet spilled
            StackMachine::reg_at_depth(stack_depth - 1) // r7
        } else {
            // Value was already spilled to memory, load it
            let spill_offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(stack_depth - 1);
            emitter.emit(Instruction::LoadIndU64 { dst: SPILL_ALT_REG, ... });
            SPILL_ALT_REG
        }
    } else {
        emitter.stack.peek(0)
    };

    if let Some(reg) = local_reg(idx) {
        emitter.emit(Instruction::AddImm64 { dst: reg, src, value: 0 });
    } else {
        // Spilled local handling
        let temp = if src == SPILL_ALT_REG { 7 } else { SPILL_ALT_REG };
        // ...
    }
}
```

#### Risk

This code has many branches and special cases:
1. Pending spill vs already spilled
2. Register local vs spilled local
3. Using r7 as temp when src is r8

The complexity suggests high bug risk. The previous fix for Game of Life (mentioned in PLAN.md) addressed bugs here, but more may exist.

**Key Question**: What if `src == 7` and we need a temp, but `local_reg(idx)` returns `Some(9)`? The code uses `AddImm64` with src=7, dst=9. That's fine.

But what if `src == SPILL_ALT_REG (8)` and `local_reg(idx)` returns `None` (spilled local)? Then:
```rust
let temp = if src == SPILL_ALT_REG { 7 } else { SPILL_ALT_REG };  // temp = 7
emitter.emit(Instruction::LoadImm { reg: temp, value: offset });  // r7 = offset
emitter.emit(Instruction::StoreIndU64 { base: temp, src, offset: 0 });  // Store r8 to [r7]
```

This seems correct. But the complexity is concerning.

---

### Issue 6: Stack Pointer Calculation May Underflow 🔴

**Severity**: Medium  
**Location**: `codegen.rs:390-446`

#### Observation

In `emit_call`, the frame size calculation:

```rust
let spilled_frame_bytes = (num_spilled_locals * 8) as i32;
let operand_stack_start = 40 + spilled_frame_bytes;
let frame_size = operand_stack_start + (stack_depth_before_args * 8) as i32;
```

Then later:
```rust
self.emit(Instruction::AddImm64 { dst: STACK_PTR_REG, src: STACK_PTR_REG, value: -frame_size });
```

#### Risk

If `frame_size` is larger than the current stack pointer, the subtraction underflows. The stack overflow check happens BEFORE this subtraction:

```rust
let limit = stack_limit(stack_size);
// ... check if new_sp < stack_limit ...
```

But this check uses unsigned comparison. If SP is near 0 and frame_size is large, the subtraction wraps around in unsigned arithmetic.

However, the stack pointer is initialized to a high value (near 0xFEFE0000) and grows downward. Underflow would require a frame larger than the entire address space, which is unlikely.

But what if there's a bug and SP gets corrupted? Then subsequent calls could cause issues.

---

## Summary of Correctness Issues

| Issue | Severity | Status | Location |
|-------|----------|--------|----------|
| memory.copy overlap | 🔴 High | Open (needs verification) | codegen.rs:2481+ |
| Division overflow | 🔴 Medium | Open | codegen.rs:1462+, 1716+ |
| Import return values | 🔴 Medium | Open | codegen.rs:2210+ |
| Stack overflow clobber | 🔴 High | Mitigated but fragile | codegen.rs:390+ |
| I32 comparison normalization | 🟡 Medium | Probably correct but fragile | codegen.rs:1756+ |
| BrTable out-of-bounds | 🟡 Low | Likely correct | codegen.rs:2114+ |
| Call indirect signature | 🟡 Medium | Appears correct | codegen.rs:857+ |
| LocalTee complexity | 🔴 High | Bug-prone | codegen.rs:1304+ |
| Stack pointer underflow | 🔴 Medium | Edge case | codegen.rs:390+ |

---

## Recommendations

### Immediate Actions

1. **Fix division overflow checks** - Add explicit checks for div by zero and INT_MIN/-1
2. **Fix import return values** - Push dummy value when import has return type
3. **Verify memory.copy** - Test overlapping copy scenarios thoroughly
4. **Add assertions** - Insert debug assertions for stack depth, register validity

### Testing

See [07-testing-strategy.md](./07-testing-strategy.md) for comprehensive testing recommendations.

---

*Next: [03-missing-features.md](./03-missing-features.md)*
