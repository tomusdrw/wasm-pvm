# 05 - Performance Inefficiencies

**Category**: Performance  
**Impact**: Generated code size, execution speed, memory usage

---

## Summary

While performance is not the primary goal for V1, several inefficiencies in the compiler and generated code should be noted for future optimization work.

---

## Compiler Performance Issues

### Issue 1: Single-Pass Translation Without Caching

**Severity**: Low  
**Impact**: Compilation time for large modules

#### Description

The compiler processes each WASM operator exactly once and emits PVM code immediately. There's no intermediate representation to enable:
- Constant folding
- Dead code elimination
- Common subexpression elimination

#### Example of Missed Optimization

```wasm
;; WASM input
i32.const 5
i32.const 3
i32.add
i32.const 10
i32.mul
```

**Current output**: 
- LoadImm r2, 5
- LoadImm r3, 3
- Add32 r4, r2, r3
- LoadImm r2, 10
- Mul32 r3, r4, r2

**Optimized output**:
- LoadImm r2, 80  ; (5+3)*10 computed at compile time

#### Recommendation

Add a constant folding pass after IR generation but before code generation.

---

### Issue 2: Redundant Register Moves

**Severity**: Medium  
**Impact**: Code size, execution speed

#### Description

The compiler emits unnecessary register-to-register moves.

#### Example

```rust
// From codegen.rs (I32Add)
let src2 = emitter.spill_pop();  // Pops to r3
let src1 = emitter.spill_pop();  // Pops to r2
let dst = emitter.spill_push();  // Pushes to r2
emitter.emit(Instruction::Add32 { dst, src1, src2 });  // Add32 r2, r2, r3
```

If the stack was [a, b] with a in r2 and b in r3:
- After pop: b in r3
- After pop: a in r2
- Push returns r2
- Add32 r2, r2, r3 produces result in r2

This is actually correct for the stack model. But consider:

```wasm
;; WASM
local.get 0
local.get 1
i32.add
local.set 2
```

**Current**: r9 → r2, r10 → r3, add → r2, r2 → r11
**Optimized**: r9 + r10 → r11 directly (no intermediate stack)

#### Recommendation

Track value locations and emit operations directly to destination registers when possible.

---

### Issue 3: Inefficient Spilling Strategy

**Severity**: Medium  
**Impact**: Memory traffic, code size

#### Description

Values are spilled based on stack depth, not liveness.

```rust
// From stack.rs
pub const fn needs_spill(depth: usize) -> bool {
    depth >= STACK_REG_COUNT  // Spill when depth >= 5
}
```

#### Problem Scenario

```wasm
;; WASM - deep expression
i32.const 1
i32.const 2
i32.const 3
i32.const 4
i32.const 5
i32.const 6
drop
drop
drop
drop
drop
```

**Current behavior**:
- Push 1, 2, 3, 4, 5 (fit in r2-r6)
- Push 6 (spills 1 to memory)
- Drop 6 (loads 1 back from memory)
- etc.

**Optimal behavior**:
- Recognize that intermediate values are short-lived
- Keep hot values in registers
- Spill cold values

#### Recommendation

Implement liveness analysis and spill the value with the longest remaining lifetime.

---

### Issue 4: BrTable is Linear Search

**Severity**: Medium  
**Impact**: O(n) branch dispatch for n cases

#### Description

`br_table` uses a chain of comparisons:

```rust
// From codegen.rs:2114-2147
for (i, &depth) in target_depths.iter().enumerate() {
    let next_label = emitter.alloc_label();
    emitter.emit_branch_ne_imm_to_label(index_reg, i as i32, next_label);
    // ... branch to target ...
    emitter.emit(Instruction::Fallthrough);
    emitter.define_label(next_label);
}
```

For a table with 100 entries, this does up to 100 comparisons!

#### Better Approaches

1. **Binary search**: O(log n) comparisons
2. **Jump table**: O(1) with indirect jump (if PVM supports it)
3. **Range check + offset**: For dense tables

#### Recommendation

For now, document the limitation. For production, implement jump tables or binary search.

---

### Issue 5: Unnecessary Fallthrough Instructions

**Severity**: Low  
**Impact**: Code size

#### Description

`Fallthrough` instructions are emitted when not strictly necessary.

```rust
// From codegen.rs
fn define_label(&mut self, label: usize) {
    if self.instructions.last()
        .is_some_and(|last| !last.is_terminating()) {
        self.emit(Instruction::Fallthrough);
    }
    self.labels[label] = Some(self.current_offset());
}
```

#### Problem

Fallthrough takes 1 byte. For sequential code without branches, these add up.

#### Better Approach

Only emit Fallthrough at actual basic block boundaries required by PVM semantics.

---

### Issue 6: Memory Operations Use 64-bit When 32-bit Sufficient

**Severity**: Low  
**Impact**: Code size, potentially performance

#### Description

Some operations use 64-bit instructions when 32-bit would suffice.

```rust
// LocalGet for register-based local uses AddImm64
if let Some(reg) = local_reg(idx) {
    let dst = emitter.spill_push();
    emitter.emit(Instruction::AddImm64 { dst, src: reg, value: 0 });
}
```

`AddImm64` with 0 is effectively a move. On PVM, this might be:
- 4 bytes: opcode + reg encoding + 0 (varint)

A hypothetical `Move` instruction might be:
- 2 bytes: opcode + regs

#### Recommendation

Consider adding a `Move` pseudo-instruction or use `AddImm32` when values are known to be 32-bit.

---

## Generated Code Performance Issues

### Issue 1: No Constant Propagation

**Severity**: Medium  
**Impact**: Runtime computation of known values

#### Example

```wasm
;; AssemblyScript-like pattern
i32.const 100  ;; loop limit
local.set $limit
loop
  local.get $i
  local.get $limit  ;; Loaded from memory every iteration!
  i32.lt_u
  if
    ;; loop body
    local.get $i
    i32.const 1
    i32.add
    local.set $i
    br 0
  end
end
```

**Current**: `$limit` is loaded from local memory every iteration.
**Optimized**: Keep `$limit` in a register throughout the loop.

#### Recommendation

Implement a simple register allocation that keeps loop-invariant values in registers.

---

### Issue 2: Comparison Operations are Verbose

**Severity**: Low  
**Impact**: Code size for comparisons

#### Example

I32GeU (greater than or equal unsigned) is implemented as:

```rust
// From codegen.rs:1858-1876
let b = emitter.spill_pop();
let a = emitter.spill_pop();
emitter.emit(Instruction::AddImm32 { dst: a, src: a, value: 0 });
emitter.emit(Instruction::AddImm32 { dst: b, src: b, value: 0 });
let dst = emitter.spill_push();
emitter.emit(Instruction::SetLtU { dst, src1: b, src2: a });  // b < a ?
let one = emitter.spill_push();
emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
let _ = emitter.spill_pop();
emitter.emit(Instruction::Xor { dst, src1: dst, src2: one });  // not (b < a) = a >= b
```

That's 9 instructions for one comparison!

#### Better Approach

If PVM had a `SetGeU` instruction, this would be 1 instruction.

#### Recommendation

Document as known limitation. Could add synthetic instructions that expand to efficient sequences.

---

### Issue 3: Memory.fill Uses Byte-by-Byte Loop

**Severity**: Medium  
**Impact**: Slow for large fills

#### Description

```rust
// From codegen.rs:2429-2479
// Use a loop to fill memory byte by byte
// while (size > 0) { mem[dest] = value; dest++; size--; }
```

#### Problem

For a 1KB fill, this does 1024 iterations.

#### Better Approaches

1. **Word-sized writes**: Fill 8 bytes at a time when aligned
2. **Block copy**: Use PVM's memory block operations if available
3. **Unroll loops**: For small fixed sizes

#### Recommendation

Optimize for common cases:
- Small fills (< 32 bytes): Unrolled
- Large fills: Word-sized
- Aligned fills: Use larger operations

---

### Issue 4: Frame Size Always Includes Spill Space

**Severity**: Low  
**Impact**: Stack usage

#### Description

From `emit_call`:
```rust
let spilled_frame_bytes = (num_spilled_locals * 8) as i32;
let operand_stack_start = 40 + spilled_frame_bytes;
let frame_size = operand_stack_start + (stack_depth_before_args * 8) as i32;
```

Even if no spilled locals are live across the call, space is still reserved.

#### Recommendation

Track liveness of spilled locals and only save those that are actually live across the call.

---

## Resource Usage

### Memory Layout Inefficiency

**Current layout** (per function):
- Spilled locals: 512 bytes (64 locals * 8 bytes)

**If a function has only 5 locals**:
- 4 in registers (r9-r12)
- 1 spilled
- Still reserves 512 bytes

**Better**: Variable-size allocation based on actual spilled local count.

---

## Performance Testing

### Current State

- No performance benchmarks exist
- 62 integration tests verify correctness only
- No measurement of:
  - Compilation time
  - Generated code size
  - Execution speed
  - Memory usage

### Recommended Benchmarks

| Benchmark | Purpose |
|-----------|---------|
| Fibonacci(100) | Function call overhead |
| Matrix multiply | Memory access patterns |
| Bubble sort (large array) | Loop and comparison performance |
| Recursion depth test | Call stack efficiency |
| Compilation time | Measure compiler performance |

---

## Summary Table

| Issue | Severity | Effort to Fix | Priority |
|-------|----------|---------------|----------|
| No constant folding | Medium | Medium | 3 |
| Redundant moves | Medium | Medium | 4 |
| Inefficient spilling | Medium | High | 5 |
| Linear br_table | Medium | Medium | 6 |
| Unnecessary fallthrough | Low | Low | 8 |
| 64-bit for 32-bit ops | Low | Low | 9 |
| Verbose comparisons | Low | Low | 10 |
| Byte-by-byte fill | Medium | Medium | 7 |
| Frame size overhead | Low | Low | 11 |

---

## Recommendations

### Phase 1: Measure (Immediate)

1. Add benchmarks for common operations
2. Measure current performance baseline
3. Identify actual bottlenecks

### Phase 2: Low-Hanging Fruit (Short Term)

4. Remove unnecessary Fallthrough instructions
5. Optimize memory.fill for word-sized operations
6. Use AddImm32 instead of AddImm64 where safe

### Phase 3: IR-Based Optimizations (Long Term)

7. Implement constant folding
8. Add dead code elimination
9. Improve register allocation with liveness analysis
10. Optimize br_table with jump tables

---

*Next: [06-proposed-architecture.md](./06-proposed-architecture.md)*
