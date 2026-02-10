# 05 - Performance Inefficiencies

**Category**: Performance  
**Impact**: Generated code size, execution speed, memory usage  
**Status**: Known V1 tradeoffs, optimization opportunities identified

---

## Summary

Performance was deprioritized for V1 in favor of correctness. The LLVM-based compiler makes intentional correctness-over-performance tradeoffs, primarily using a conservative stack-slot allocation strategy.

**Performance Profile**:

| Aspect | Current State | Priority |
|--------|--------------|----------|
| **Compilation time** | Moderate (LLVM passes) | Medium |
| **Generated code size** | Baseline | Low |
| **Execution speed** | Limited by stack slots | High for V2 |
| **Memory usage** | Larger stack frames | Medium |

---

## LLVM Optimizations (Working)

### ✅ mem2reg Pass

**Status**: Active via LLVM

**Benefit**: Promotes alloca-based locals to SSA registers, eliminating some redundant stack operations.

**Example**:
```llvm
; Before mem2reg
%local = alloca i32
store i32 42, %local
%val = load i32, %local

; After mem2reg
%val = i32 42  ; Direct SSA value
```

---

### ✅ LLVM InstCombine

**Status**: Active via LLVM

**Benefit**: Combines redundant instructions, simplifies expressions.

---

## Performance Issues

### Issue 1: Stack-Slot Allocation (V1 Tradeoff) 🔴

**Severity**: High  
**Location**: `llvm_backend/lowering.rs`

**Current Strategy**: Every SSA value gets a dedicated memory slot

```rust
fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
    let offset = self.next_slot_offset;
    self.value_slots.insert(key, offset);
    self.next_slot_offset += 8;  // All slots 8 bytes
    offset
}
```

**Problems**:
1. **All values to memory**: No values kept in registers
2. **8-byte slots for all**: i32 values waste 4 bytes
3. **No slot reuse**: Slots never freed
4. **Memory traffic**: Every operation loads/stores

**Impact**:
- Larger stack frames (could be 2-5x with registers)
- Slower execution (memory vs register operations)
- More cache pressure

**Rationale**: "Correctness-first, no register allocation" - intentional V1 simplification

**Recommendation**: Implement register allocation in V2 (linear scan or graph coloring).

---

### Issue 2: Redundant Load/Store Pattern 🔴

**Severity**: Medium-High  
**Location**: `llvm_backend/lowering.rs`

**Pattern**: Every operation follows load-compute-store:

```rust
// Load from slot to temp
emit(Instruction::LoadIndU64 { dst: TEMP1, base: SP, offset: slot1 });
emit(Instruction::LoadIndU64 { dst: TEMP2, base: SP, offset: slot2 });

// Compute
emit(Instruction::Add32 { dst: TEMP_RESULT, src1: TEMP1, src2: TEMP2 }));

// Store back to slot
emit(Instruction::StoreIndU64 { base: SP, src: TEMP_RESULT, offset: dst_slot });
```

**Problem**: Values used immediately in next instruction still get stored/reloaded.

**Example**:
```wasm
local.get 0
local.get 1
i32.add
local.get 2
i32.add
```

**Current** (inefficient):
```asm
LoadIndU64 r2, sp, slot0  ; load local 0
StoreIndU64 sp, r2, temp_slot
LoadIndU64 r3, sp, slot1  ; load local 1
Add32 r4, r2, r3
StoreIndU64 sp, r4, temp1
LoadIndU64 r5, sp, temp1  ; reload just stored!
LoadIndU64 r6, sp, slot2
Add32 r7, r5, r6
```

**Optimal** (with registers):
```asm
LoadIndU64 r2, sp, slot0
LoadIndU64 r3, sp, slot1
Add32 r2, r2, r3     ; keep in r2
LoadIndU64 r3, sp, slot2
Add32 r2, r2, r3     ; result in r2
```

**Recommendation**: Track which values are in registers and avoid redundant stores.

---

### Issue 3: BrTable Linear Search 🔴

**Severity**: Medium  
**Status**: O(n) comparisons for n targets

**Code**:
```rust
// Linear search through targets
for (i, &depth) in target_depths.iter().enumerate() {
    let next_label = emitter.alloc_label();
    emitter.emit_branch_ne_imm_to_label(index_reg, i as i32, next_label);
    // ... branch to target ...
}
```

**Problem**: 100-entry table does 100 comparisons.

**Better Approaches**:
1. **Binary search**: O(log n)
2. **Jump table**: O(1) if PVM had better indirect jump support
3. **Range check**: For dense tables

**Mitigation**: BrTable is rare in typical code. Impact limited.

**Recommendation**: Implement binary search for large tables in V2.

---

### Issue 4: Frame Size Calculation 🟡

**Severity**: Medium  
**Location**: `llvm_backend/lowering.rs`

**Issue**: Pre-scan required to count values before allocation

**Code**:
```rust
struct PvmEmitter<'ctx> {
    next_slot_offset: i32,  // Bump allocator
    frame_size: i32,        // Set after pre-scan
}
```

**Problems**:
- Two-pass lowering (count then allocate)
- No slot reuse within function
- Could overflow for large functions

**Recommendation**: Consider streaming allocation or slot reuse.

---

### Issue 5: Unnecessary Fallthrough 🟢

**Severity**: Low  
**Impact**: Minimal code bloat (~1 byte per basic block)

**Current**:
```rust
fn define_label(&mut self, label: usize) {
    if !last_instruction_is_terminator() {
        self.emit(Instruction::Fallthrough);
    }
}
```

**Mitigation**: Required for PVM basic block semantics. Acceptable overhead.

---

## Performance Testing

### Current State

**Missing**:
- No dedicated performance benchmarks
- No code size measurements
- No execution speed comparisons

**Needed Benchmarks**:

| Benchmark | Purpose | Priority |
|-----------|---------|----------|
| Fibonacci(100) | Function call overhead | High |
| Matrix multiply | Memory access patterns | Medium |
| Bubble sort | Loop and comparison | Medium |
| Recursion depth | Call stack efficiency | Medium |
| anan-as compile | Real-world compilation | High |

---

## Optimization Opportunities (Ranked)

### High Impact, Low Effort

1. **Type-aware slot sizes**
   - i32 → 4-byte slots
   - i64 → 8-byte slots
   - ~20% stack space reduction

2. **Remove redundant stores**
   - Track live values in registers
   - ~10-30% memory operation reduction

### High Impact, High Effort

3. **Implement register allocation**
   - Linear scan or graph coloring
   - 2-5x execution speedup potential
   - Major V2 goal

4. **Optimize br_table**
   - Binary search implementation
   - Significant for switch-heavy code

### Low Impact

5. **Remove unnecessary Fallthrough**
   - Minimal code size savings
   - Low priority

---

## Recommendations

### Immediate

1. **Add performance benchmarks**
   - Establish baseline measurements
   - Track regression/improvement

2. **Profile anan-as compilation**
   - Real-world performance data
   - Identify hotspots

### Short Term

3. **Implement type-aware slots**
   - Variable slot sizes (4/8 bytes)
   - Reduce stack frame size

4. **Explore LLVM optimization passes**
   - Enable more aggressive optimizations
   - Profile-guided optimization

### Long Term (V2)

5. **True register allocation**
   - Replace stack-slot approach
   - Graph coloring or linear scan

6. **Custom peephole optimizations**
   - Pattern matching in backend
   - Strength reduction

---

## Summary

| Issue | Severity | V1 Status | V2 Priority |
|-------|----------|-----------|-------------|
| Stack-slot allocation | High | Intentional tradeoff | 1 |
| Redundant load/store | Medium | Acceptable | 2 |
| Linear br_table | Medium | Rare operation | 3 |
| Frame size calc | Medium | Working | 4 |
| Unnecessary Fallthrough | Low | Minimal impact | 5 |

**Verdict**: Performance tradeoffs are **intentional and documented** for V1. The architecture supports future optimizations through LLVM's pass system. The primary opportunity is implementing register allocation in V2.

---

*Review conducted 2026-02-10*
