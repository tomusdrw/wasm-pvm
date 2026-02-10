# WASM to PVM Recompiler - Learnings & Knowledge Base (Updated)

**Version**: 2.0 (With Architecture Review Insights)  
**Date**: 2026-02-09  

---

## Document Structure

This document contains:
1. **Original Learnings** - Technical discoveries from implementation
2. **Architecture Insights** - New section from code review
3. **Debugging Journal** - History of bug fixes with lessons

---

# Part 1: Original Technical Learnings

<details>
<summary>Click to expand original LEARNINGS.md content</summary>

## PVM (Polka Virtual Machine) Overview

### From Gray Paper v0.7.2

**Core Characteristics:**
- Based on RISC-V rv64em variant (64-bit, embedded, with multiplication)
- 13 general-purpose 64-bit registers (ω ∈ ⟦ N_R ⟧^13, indexed 0-12)
- Gas-metered execution (N_G = N_{2^64})
- Memory organized in pages (PAGE_SIZE = 4KB, SEGMENT_SIZE = 64KB)
- Little-endian byte order

**Exit Conditions:**
- `∎` (halt) - Normal termination
- `☇` (panic) - Error/trap
- `∞` (out-of-gas) - Gas exhausted
- `F × address` (page-fault) - Memory access violation
- `h̵ × id` (host-call) - External function call

## SPI Format

**Binary Format:**
```
┌─────────────────────────────────────────┐
│ roLength           (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ rwLength           (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ heapPages          (u16 - 2 bytes)      │
├─────────────────────────────────────────┤
│ stackSize          (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ roData             (roLength bytes)     │
├─────────────────────────────────────────┤
│ rwData             (rwLength bytes)     │
├─────────────────────────────────────────┤
│ codeLength         (u32 - 4 bytes)      │
├─────────────────────────────────────────┤
│ code               (PVM program blob)   │
└─────────────────────────────────────────┘
```

## PVM Instruction Encoding

### ThreeReg Instructions (CRITICAL!)

For ThreeReg instructions (ADD_32, SUB_32, etc.):
- Encoding: `[opcode, src1<<4 | src2, dst]`
- **Execution: reg[c] = op(reg[b], reg[a])** ← Note the swap!

This means for `SET_LT_U`, it computes `dst = (src2 < src1)`, NOT `dst = (src1 < src2)`.

**Fix for WASM translation**: Swap operand order for comparison and division ops.

</details>

---

# Part 2: Architecture Insights (NEW)

## Current Architecture Assessment

### What Works

**Direct Translation Approach**:
- WASM → PVM conversion is straightforward and works for most programs
- 62 integration tests pass
- anan-as compiles successfully (423KB JAM file)
- Simple control flow (if/else, loops, blocks) works correctly

**Register Convention**:
- r0: Return address (jump table index)
- r1: Stack pointer
- r2-r6: Operand stack (5 slots)
- r7: Return value / SPI args pointer
- r8: SPI args length / saved table idx
- r9-r12: Local variables (first 4)

### What Doesn't Work (Architectural Debt)

#### 1. No Intermediate Representation

**Problem**: WASM is translated directly to PVM machine code.

```
Current: WASM → PVM bytes
Proper:  WASM → IR → Optimizations → PVM bytes
```

**Consequences**:
- No place to insert constant folding
- No dataflow analysis possible
- Debugging requires tracing PVM execution
- Cannot verify correctness before code generation

**Example**: To optimize `(i32.const 5) (i32.const 3) i32.add` → `(i32.const 8)`, you must modify the 600-line `translate_op()` function. With an IR, you'd add a single constant folding pass.

#### 2. Monolithic Code Generation

**Problem**: `codegen.rs` is 2,400 lines handling everything.

| Responsibility | Lines | Should Be |
|----------------|-------|-----------|
| Operator dispatch | 600+ | ~100 |
| Control flow | 200+ | Separate module |
| Function calls | 600+ | Separate module |
| Memory operations | 300+ | Separate module |
| Register allocation | Mixed | Separate module |

**Better Structure** (see review/proposals/06-proposed-architecture.md):
```
translate/
  mod.rs           - Orchestration
  ir.rs            - Intermediate representation
  instruction.rs   - Instruction selection
  register.rs        - Register allocation
  control_flow.rs    - CFG and branches
  calling.rs         - Calling convention
  memory.rs          - Memory model
```

#### 3. Ad-Hoc Register Allocation

**Problem**: Register usage is hardcoded with magic numbers.

```rust
// From codegen.rs
const ARGS_PTR_REG: u8 = 7;
const ARGS_LEN_REG: u8 = 8;
const FIRST_LOCAL_REG: u8 = 9;
const MAX_LOCAL_REGS: usize = 4;
```

**Issues**:
- Cannot change register assignments without modifying code throughout
- No liveness analysis - spills based on stack depth, not actual usage
- Fixed limit of 4 locals in registers
- No interference tracking

**Better Approach**: Graph coloring or linear scan allocator with proper interference analysis.

#### 4. Manual Spilling Complexity

**Problem**: Operand stack spilling is manually managed.

```rust
fn spill_pop(&mut self) -> u8 {
    self.flush_pending_spill();
    let depth = self.stack.depth();
    if depth > 0 && StackMachine::needs_spill(depth - 1) {
        // Complex logic to determine if value is in register or memory
        // Check pending_spill, load from spill area, handle conflicts
    }
    self.stack.pop()
}
```

**Issues**:
- 3 nested conditionals
- Side effects (flush_pending_spill)
- Hard to test all branches
- Previous bugs in this code (Game of Life fixes)

**Better Approach**: Liveness analysis + proper spilling algorithm.

#### 5. Manual Control Flow Management

**Problem**: No Control Flow Graph (CFG).

**Current**: Manual label allocation, fixup resolution, stack depth tracking.

```rust
let else_label = self.alloc_label();
let end_label = self.alloc_label();
self.fixups.push((fixup_idx, else_label));
// ... later ...
self.resolve_fixups()?;
```

**Better**: Build CFG, use standard algorithms for dominance, loops, SSA.

---

# Part 3: Debugging Journal (Lessons from Bug Fixes)

## Entry 1: Game of Life Multi-Step (2026-01-19)

**Symptom**: Game of Life worked for 1 step but failed on subsequent steps.

**Root Causes** (3 bugs found):

1. **I64Load instruction**: Invalid patterns, non-existent Instruction::LoadI64
2. **Spilled operand stack**: Reading from r7 instead of actual spill area  
   *Fix*: Load from `old_sp + frame_size + OPERAND_SPILL_BASE + offset`
3. **local.tee with spill**: Didn't check `pending_spill`, used wrong temp registers

**Lesson**: Stack spilling is fragile. Changes in one place affect others. Need comprehensive tests for all stack depths.

---

## Entry 2: PVM-in-PVM Memory Corruption (2026-02-06)

**Symptom**: Inner interpreter PANICs at PC 56, exitCode 0 (memory fault < 0x10000).

**Root Cause (Primary)**: `memory.copy` forward-only copy corrupts data when `dest > src` with overlap.

**How It Manifests**:
1. AS `Array.unshift()` uses `memory.copy` to shift elements right
2. Forward copy overwrites before reading → corrupts source
3. This corrupts Arena page allocator
4. All Uint8Array views alias same memory
5. Inner program loads garbage instead of args

**Fix**: Add backward copy path when `dest > src` (like `memmove`).

**Current Status**: Fix applied but PVM-in-PVM still PANICs. Investigation ongoing.

**Lesson**: 
- Memory operations are critical - any bug causes cascading failures
- AS runtime internals (unshift, copy) affect generated code
- Need to test with AS runtime patterns, not just WAT

---

## Entry 3: Local Variable Zero-Init (2026-02-06)

**Symptom**: Loop counters start with garbage values, loops don't execute.

**Root Cause**: WASM spec requires all locals zero-initialized, but only parameters were being set up.

**Fix**: Added `LoadImm { reg, value: 0 }` in `emit_prologue()` for non-parameter locals.

**Lesson**: WASM spec compliance matters. Small omissions cause hard-to-debug issues.

---

## Entry 4: AS u8 Arithmetic (2026-02-06)

**Symptom**: AS tests fail with 256-byte difference when values >= 128.

**Root Cause**: AS applies `& 0xFF` mask to u8 arithmetic results, even when storing to u32.

```typescript
// AS generates:
arr[0] + arr[1]  // Computes (value & 0xFF)

// Must cast to avoid mask:
<u32>arr[0] + <u32>arr[1]  // Correct
```

**Lesson**: Not all bugs are in wasm-pvm. AS semantics differ from intuition.

---

# Part 4: Design Decisions Revisited

## Decision: No Intermediate Representation

**Original**: Direct WASM → PVM for simplicity  
**Current**: Works but limits extensibility  
**Revised Recommendation**: Add IR for V2, keep direct translation for V1

**Rationale**: The current approach works for V1 scope. The technical debt is manageable. IR would enable optimizations but isn't required for correctness.

---

## Decision: Hardcoded Register Allocation

**Original**: Simple, predictable  
**Current**: Works but fragile  
**Revised Recommendation**: Keep for V1, proper allocator for V2

**Rationale**: Register allocation bugs are hard to debug. Current approach works for the 62 test cases. A proper allocator (graph coloring) is a V2 feature.

---

## Decision: Stack-based Translation

**Original**: Direct WASM stack to PVM registers  
**Current**: Works with careful spilling  
**Revised Recommendation**: Keep for V1, consider stack-to-SSA for V2

**Rationale**: The current spilling approach works. An IR would convert stack operations to SSA form, but that's V2 scope.

---

# Part 5: Recommendations for Future Development

## Immediate (V1 Completion)

1. **Fix critical bugs**: memory.copy, division overflow, import returns
2. **Make PVM-in-PVM work**: Debug remaining PANIC issue
3. **Document assumptions**: Memory layout, register usage, calling convention
4. **Add assertions**: Stack depth, register validity in debug builds

## Short Term (V1 Polish)

5. **Extract constants**: Create MemoryLayout abstraction
6. **Fix warnings**: Remove clippy suppressions
7. **Add tests**: StackMachine, spilling, control flow
8. **Document thoroughly**: Every register usage, every memory address

## Long Term (V2 Architecture)

9. **Add IR**: SSA-based intermediate representation
10. **Split codegen**: Separate modules per responsibility
11. **Proper allocator**: Graph coloring or linear scan
12. **Optimizations**: Constant folding, DCE, CSE
13. **CFG**: Control flow graph with dominance analysis

---

# Part 6: Key Insights from Architecture Review

## The Compiler Works, But...

**Works for**: 
- Simple programs (add, factorial, fibonacci)
- Direct execution (no nesting)
- Programs without complex memory patterns

**Struggles with**:
- Complex AS runtime (unshift, allocations)
- PVM-in-PVM nesting
- Edge cases (div overflow, memmove)

## Why Architecture Matters

The current architecture is a **direct translator**, not a real compiler:

| Aspect | Translator | Compiler |
|--------|-----------|----------|
| Structure | Single pass | Multiple phases |
| Optimization | None | Many passes |
| Debugging | Hard (trace PVM) | Easy (inspect IR) |
| Correctness | Test-only | Verify + test |
| Extensibility | Poor | Excellent |

**The trade-off**: Translator is simpler to write initially, but hits limits. For V1, this is acceptable. For V2 (optimizations, complex features), need proper compiler architecture.

## Testing Strategy Insights

**Current**: 62 integration tests, 30 unit tests  
**Gap**: No tests for:
- Stack spilling at various depths
- Overlapping memory.copy
- Division edge cases
- Complex AS patterns

**Recommendation**: See review/proposals/07-testing-strategy.md for comprehensive testing plan.

---

## References

- Original: LEARNINGS.md (technical details)
- Architecture Review: review/findings/ (critical analysis)
- Proposed Architecture: review/proposals/06-proposed-architecture.md
- Testing Strategy: review/proposals/07-testing-strategy.md
- Rebuilding Plan: review/proposals/08-rebuilding-plan.md

---

*This document integrates original technical learnings with architecture review insights.*
