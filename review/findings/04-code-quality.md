# 04 - Code Quality Issues

**Category**: Maintainability and Technical Debt  
**Impact**: Development velocity, bug introduction, onboarding difficulty

---

## Summary

Beyond architectural flaws, the codebase has numerous code quality issues that make it hard to understand, modify, and maintain.

---

## Code Smells

### Smell 1: Magic Numbers Everywhere 🔴

**Severity**: High  
**Frequency**: Pervasive

#### Description

Numerical constants are scattered throughout the code without explanation or abstraction.

#### Examples

```rust
// From codegen.rs
const GLOBAL_MEMORY_BASE: i32 = 0x30000;
const SPILLED_LOCALS_BASE: i32 = 0x40000;
const EXIT_ADDRESS: i32 = -65536;
const RO_DATA_BASE: i32 = 0x10000;
const PARAM_OVERFLOW_BASE: i32 = 0x3FF00;
const STACK_SEGMENT_END: i32 = 0xFEFE_0000;
const SPILLED_LOCALS_PER_FUNC: i32 = 512;
const ENTRY_HEADER_SIZE: usize = 10;
const MAX_LOCAL_REGS: usize = 4;
```

And in calculations:
```rust
// From codegen.rs
let frame_size = operand_stack_start + (stack_depth_before_args * 8) as i32;
let spill_offset = frame_size + OPERAND_SPILL_BASE + StackMachine::spill_offset(i);
let mem_offset = SPILLED_LOCALS_BASE + (func_idx as i32) * 512 + ((local_idx - 4) as i32) * 8;
```

#### Problems

1. **No explanation**: Why 0x30000? Why 512 bytes per function?
2. **Hard to change**: Need to find and update every occurrence
3. **Inconsistent**: Some addresses computed, some hardcoded
4. **No validation**: Could have overlaps or gaps

#### Better Approach

```rust
struct MemoryLayout {
    const GLOBALS_START: u32 = 0x30000;
    const GLOBALS_SIZE: u32 = 0x100;  // 256 bytes, 64 globals max
    
    const SPILLED_LOCALS_START: u32 = 0x40000;
    const SPILLED_LOCALS_SIZE_PER_FUNC: u32 = 512;  // 64 locals * 8 bytes
    
    fn validate(&self) -> Result<()> {
        // Check no overlaps, all regions valid
    }
}
```

---

### Smell 2: Commented-Out Code 🔴

**Severity**: Medium  
**Frequency**: Occasional

#### Description

Dead code is commented out rather than removed, creating clutter and confusion.

#### Example

```rust
// From codegen.rs (presumably, based on patterns)
// Old implementation:
// let dst = self.stack.pop();
// self.emit(Instruction::StoreIndU64 { ... });

// New implementation:
let dst = self.spill_pop();
```

#### Problems

1. **Clutter**: Makes files longer than necessary
2. **Confusion**: Readers wonder why code was commented
3. **Bit rot**: Commented code becomes outdated
4. **Version control abuse**: Git tracks history, no need to keep in source

#### Better Approach

Use version control (git) to track history. Delete unused code:

```bash
git log -p -- codegen.rs | grep -A 5 "old implementation"
```

---

### Smell 3: Excessive Comments Explaining Obvious Code 🟡

**Severity**: Low  
**Frequency**: Common

#### Description

Comments state the obvious, creating noise.

#### Examples

```rust
// From codegen.rs
// Copy the adjusted value to r9 (local 0)
emitter.emit(Instruction::AddImm64 { dst: FIRST_LOCAL_REG, src: ARGS_PTR_REG, value: 0 });

// Pop the return value
let ret_val = emitter.spill_pop();
```

#### Problems

1. **Noise**: Makes it hard to find important comments
2. **Maintenance burden**: Comments must be updated with code
3. **Redundancy**: Code should be self-documenting

#### Better Approach

Use descriptive names and only comment WHY, not WHAT:

```rust
// Good: Explains business reason
// r7 holds SPI args pointer. We must preserve it across the start function call
// because main() expects to receive it unchanged.

// Bad: States the obvious
// Copy r7 to r9
```

---

### Smell 4: Large Functions with Multiple Responsibilities 🔴

**Severity**: High  
**Frequency**: Pervasive

#### Description

Functions do too much, making them hard to understand and test.

#### Examples

| Function | Lines | Responsibilities |
|----------|-------|------------------|
| `translate_op()` | 600+ | 100+ WASM operators |
| `emit_call()` | ~300 | Stack setup, register save, argument passing, overflow check |
| `emit_call_indirect()` | ~400 | Same as emit_call + dispatch table + signature check |
| `compile()` in mod.rs | ~400 | Parsing, orchestration, fixup resolution |

#### Problems

1. **Hard to test**: Cannot test individual behaviors
2. **Hard to understand**: Too many code paths
3. **High cyclomatic complexity**: Too many branches
4. **Duplication**: Similar logic in emit_call and emit_call_indirect

#### Better Approach

Extract functions with single responsibilities:

```rust
// Instead of emit_call doing everything:
fn emit_call(...) {
    emit_stack_overflow_check(...)?;
    emit_frame_setup(...)?;
    emit_register_save(...)?;
    emit_argument_passing(...)?;
    emit_jump_to_target(...)?;
    emit_return_point(...)?;
    emit_register_restore(...)?;
    emit_frame_teardown(...)?;
}
```

---

### Smell 5: Complex Conditional Logic 🔴

**Severity**: High  
**Frequency**: Common

#### Description

Deep nesting and complex conditions make code hard to follow.

#### Example

```rust
// From codegen.rs:1304-1360 (LocalTee)
let src = if stack_depth > 0 && StackMachine::needs_spill(stack_depth - 1) {
    if emitter.pending_spill == Some(stack_depth - 1) {
        StackMachine::reg_at_depth(stack_depth - 1)
    } else {
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
    let temp = if src == SPILL_ALT_REG { 7 } else { SPILL_ALT_REG };
    emitter.emit(Instruction::LoadImm { reg: temp, value: offset });
    emitter.emit(Instruction::StoreIndU64 { base: temp, src, offset: 0 });
}
```

This has:
- 3 levels of nesting in the first `if`
- Pattern matching (`if let`)
- Another conditional inside the `else`

#### Problems

1. **Cognitive load**: Hard to track all conditions
2. **Testability**: Need many test cases for all branches
3. **Bug-prone**: Easy to miss edge cases

#### Better Approach

Use early returns and extract helper functions:

```rust
fn get_stack_top_value(&mut self) -> Result<Operand> {
    let depth = self.stack.depth();
    if depth == 0 {
        return Err(Error::StackUnderflow);
    }
    
    let top_idx = depth - 1;
    if !StackMachine::needs_spill(top_idx) {
        return Ok(Operand::Register(StackMachine::reg_at_depth(top_idx)));
    }
    
    if self.pending_spill == Some(top_idx) {
        return Ok(Operand::Register(StackMachine::reg_at_depth(top_idx)));
    }
    
    let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(top_idx);
    self.emit(Instruction::LoadIndU64 { dst: SPILL_ALT_REG, base: STACK_PTR_REG, offset });
    Ok(Operand::Register(SPILL_ALT_REG))
}
```

---

### Smell 6: Unused Code 🔴

**Severity**: Medium  
**Frequency**: Some instances

#### Description

Code that exists but is never used, creating confusion.

#### Examples

```rust
// From mod.rs:699-708
#[allow(dead_code)]
fn check_for_floats(body: &FunctionBody) -> Result<()> {
    // Exists but not called anywhere
}

// From mod.rs:710-767
#[allow(dead_code)]
fn is_float_op(op: &wasmparser::Operator) -> bool {
    // Exists but not called
}
```

#### Problems

1. **Confusion**: Is this supposed to be used?
2. **Maintenance burden**: Must update with changes
3. **Dead weight**: Increases compile time slightly

#### Better Approach

Delete unused code. If needed later, retrieve from git history.

---

### Smell 7: Suppressed Warnings 🔴

**Severity**: Medium  
**Frequency**: Pervasive

#### Description

Compiler warnings are suppressed rather than addressed.

#### Evidence

```rust
// From lib.rs
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
)]

// From codegen.rs
#[allow(dead_code)]
fn spill_finalize(&mut self) { }

// From mod.rs
#[allow(dead_code)]
fn check_for_floats(...) { }
```

#### Problems

1. **Clippy is right**: These warnings indicate real issues
2. **Hiding bugs**: `cast_possible_truncation` could hide integer overflow bugs
3. **Technical debt**: Warnings accumulate until someone disables them

#### Better Approach

Fix the underlying issues:

```rust
// Instead of suppressing cast warnings:
let value = some_u64 as u32;  // Risky!

// Use checked conversions:
let value = u32::try_from(some_u64).map_err(|_| Error::IntegerOverflow)?;

// Or explicit truncation with comment:
// Truncation is safe here because we know value fits in 32 bits from earlier check
let value = some_u64 as u32;
```

---

### Smell 8: Duplicate Code 🔴

**Severity**: High  
**Frequency**: Common

#### Description

Similar code repeated in multiple places.

#### Examples

1. **emit_call and emit_call_indirect** share ~80% of their code
2. **Division operations** (I32DivU, I32DivS, I64DivU, I64DivS) are nearly identical
3. **Comparison operations** share normalization logic
4. **Load/store operations** share address calculation

#### Example from Code

```rust
// I32DivU (codegen.rs:1462-1470)
let src2 = emitter.spill_pop();
let src1 = emitter.spill_pop();
let dst = emitter.spill_push();
emitter.emit(Instruction::DivU32 { dst, src1: src2, src2: src1 });

// I32DivS (codegen.rs:1472-1481)
let src2 = emitter.spill_pop();
let src1 = emitter.spill_pop();
let dst = emitter.spill_push();
emitter.emit(Instruction::DivS32 { dst, src1: src2, src2: src1 });

// Only difference: DivU32 vs DivS32!
```

#### Problems

1. **Maintenance**: Fix bugs in multiple places
2. **Inconsistency**: Changes may not propagate everywhere
3. **Bloat**: Makes files longer

#### Better Approach

Use generics or helper functions:

```rust
fn emit_binary_op<F>(&mut self, op: F) -> Result<()>
where F: FnOnce(u8, u8, u8) -> Instruction {
    let src2 = self.spill_pop();
    let src1 = self.spill_pop();
    let dst = self.spill_push();
    self.emit(op(dst, src1, src2));
    Ok(())
}

// Usage:
Operator::I32Add => emitter.emit_binary_op(Instruction::Add32)?,
Operator::I32DivU => emitter.emit_binary_op(Instruction::DivU32)?,
Operator::I32DivS => emitter.emit_binary_op(|d, s1, s2| {
    // With overflow check
    self.emit_overflow_check(s1, s2)?;
    Instruction::DivS32 { dst: d, src1: s1, src2: s2 }
})?,
```

---

### Smell 9: State Management via Side Effects 🔴

**Severity**: High  
**Frequency**: Pervasive

#### Description

Functions modify internal state as side effects rather than returning values.

#### Example

```rust
// From stack.rs
pub fn push(&mut self) -> u8 {
    let reg = Self::reg_for_depth(self.depth);
    self.depth += 1;  // Side effect!
    if self.depth > self.max_depth {
        self.max_depth = self.depth;  // Side effect!
    }
    reg
}

// From codegen.rs
fn spill_push(&mut self) -> u8 {
    self.flush_pending_spill();  // Side effect!
    self.last_spill_pop_reg = None;  // Side effect!
    let depth = self.stack.depth();
    let reg = self.stack.push();  // Side effect!
    if StackMachine::needs_spill(depth) {
        self.pending_spill = Some(depth);  // Side effect!
    }
    reg
}
```

Each call modifies multiple pieces of state. This is hard to reason about.

#### Problems

1. **Hidden effects**: Caller can't see all changes
2. **Testing difficulty**: Must verify all state changes
3. **Ordering dependencies**: Must call in correct order
4. **Non-reproducible**: State depends on history

#### Better Approach

Make state transitions explicit:

```rust
struct StackTransition {
    new_depth: usize,
    allocated_register: u8,
    needs_spill: bool,
}

impl StackMachine {
    fn push(&self) -> StackTransition {
        let reg = Self::reg_for_depth(self.depth);
        StackTransition {
            new_depth: self.depth + 1,
            allocated_register: reg,
            needs_spill: self.depth >= STACK_REG_COUNT,
        }
    }
    
    fn apply(&mut self, transition: StackTransition) {
        self.depth = transition.new_depth;
        self.max_depth = self.max_depth.max(self.depth);
    }
}
```

---

### Smell 10: Tight Coupling to PVM Details 🔴

**Severity**: Medium  
**Frequency**: Pervasive

#### Description

The compiler is tightly coupled to PVM-specific details, making it hard to:
1. Test without generating actual PVM code
2. Retarget to other VMs
3. Reason about semantics independently of encoding

#### Examples

1. **Register numbers hardcoded**: r0-r12 assumed
2. **Instruction encoding mixed in**: `Instruction::encode()` called during translation
3. **PVM-specific calling convention**: Jump table, r0/r1 usage
4. **PVM memory layout**: Specific addresses for globals, stack

#### Better Approach

Introduce abstraction layers:

```rust
// Abstract register (not tied to PVM)
enum VirtualRegister {
    StackSlot(usize),      // Abstract stack position
    Local(usize),          // Local variable
    Temporary(usize),      // Compiler temporary
}

// Abstract instruction (not tied to PVM)
enum IrInstruction {
    Add32 { dst: VirtualRegister, src1: VirtualRegister, src2: VirtualRegister },
    Load { dst: VirtualRegister, addr: VirtualRegister, offset: i32 },
    // ...
}

// Later phase: map to PVM
struct PvmTarget;
impl CodeGenerator for PvmTarget {
    fn emit_add32(&mut self, dst: PvmReg, src1: PvmReg, src2: PvmReg) {
        self.emit(Instruction::Add32 { dst, src1, src2 });
    }
}
```

---

## Metrics Summary

| Metric | Current | Target |
|--------|---------|--------|
| Lines per file (max) | 2,400 | < 500 |
| Lines per function (max) | 600+ | < 50 |
| Functions with >10 branches | Many | < 5 |
| Public functions without docs | Many | 0 |
| Suppressed clippy warnings | 6+ | 0 |
| Duplicate code blocks | 15+ | < 5 |

---

## Recommendations

### Immediate (Low Effort)

1. **Remove dead code** - Delete `#[allow(dead_code)]` functions
2. **Fix easy clippy warnings** - Address cast warnings with explicit conversions
3. **Delete commented-out code** - Use git for history
4. **Remove obvious comments** - Keep only explanatory comments

### Short Term (Medium Effort)

5. **Extract helper functions** - Reduce function sizes
6. **Abstract magic numbers** - Create MemoryLayout, RegisterSet abstractions
7. **Reduce duplication** - Create generic emit functions
8. **Simplify conditionals** - Use early returns

### Long Term (High Effort)

9. **Introduce IR layer** - Separate translation from code generation
10. **Add abstraction layers** - Decouple from PVM specifics
11. **Make state explicit** - Reduce side effects

---

*Next: [05-performance.md](./05-performance.md)*
