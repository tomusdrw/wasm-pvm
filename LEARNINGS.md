# WASM to PVM Recompiler - Technical Reference

**Date**: 2026-02-13

Technical learnings, PVM architecture details, and conventions used in the compiler.

---

## PVM (Polka Virtual Machine) Overview

### From Gray Paper v0.7.2

**Core Characteristics:**
- Based on RISC-V rv64em variant (64-bit, embedded, with multiplication)
- 13 general-purpose 64-bit registers (indexed 0-12)
- Gas-metered execution
- Memory organized in pages (PAGE_SIZE = 4KB, SEGMENT_SIZE = 64KB)
- Little-endian byte order

**Exit Conditions:**
- halt — Normal termination
- panic — Error/trap
- out-of-gas — Gas exhausted
- page-fault — Memory access violation
- host-call — External function call

---

## Register Convention

| Register | Usage |
|----------|-------|
| r0 | Return address (jump table offset) |
| r1 | Stack pointer |
| r2-r6 | Scratch registers |
| r7 | Return value from calls / SPI args pointer (in main) |
| r8 | SPI args length (in main) |
| r9-r12 | Local variables (first 4) / callee-saved across calls |

---

## Calling Convention

**Before call (caller-side):**
1. Calculate `new_sp = sp - frame_size`
2. Stack overflow check: `new_sp >= stack_limit` (unsigned comparison)
3. Save return address (r0) to `[sp+0]`
4. Save locals r9-r12 to `[sp+8..40]`
5. Save any additional state to stack
6. Place arguments in r9+ (first 4 args) and PARAM_OVERFLOW_BASE (5th+)
7. Load return address (jump table offset) into r0
8. Jump to callee entry point

**After return (caller-side):**
1. Restore return address (r0) from `[sp+0]`
2. Restore locals r9-r12 from `[sp+8..40]`
3. Restore additional state from stack
4. Increment stack pointer by frame size
5. Copy return value from r7

**Stack overflow detection:**
- Default stack size: 64KB (configurable in SPI format up to 16MB)
- Stack grows downward from `0xFEFE0000`
- With ~40-byte frames, overflow occurs at ~1600 recursion depth

---

## Jump Table Mechanism

PVM's `JUMP_IND` instruction uses a **jump table** lookup, NOT direct address jumping:

```text
JUMP_IND rA, offset
  target_address = jumpTable[(rA + offset) / 2 - 1]
  jump to target_address
```

- Value `2` refers to `jumpTable[0]`
- Value `4` refers to `jumpTable[1]`
- Value `2*(N+1)` refers to `jumpTable[N]`
- Value `0xFFFF0000` (EXIT address) is special-cased for program termination

---

## PVM Instruction Encoding

### ThreeReg Instructions

For ThreeReg instructions (ADD_32, SUB_32, etc.):
- Encoding: `[opcode, src2<<4 | src1, dst]`
- **Semantics: reg[dst] = reg[src1] op reg[src2]**

Note that `src2` is in the high nibble (rB) and `src1` is in the low nibble (rA).

### TwoRegOneImm Encoding

High nibble (args.a) is typically the SOURCE, low nibble (args.b) is the DESTINATION:

```text
Byte layout: [opcode] [src << 4 | dst] [imm...]

Example ADD_IMM_32: regs[dst] = regs[src] + imm
Example LOAD_IND_U32: regs[dst] = memory[regs[base] + offset]
Example STORE_IND_U32: memory[regs[base] + offset] = regs[src]
```

---

## SPI (Standard Program Interface) Format

**Binary Format:**
```
+------------------------------------------+
| roLength           (u24 - 3 bytes)       |
| rwLength           (u24 - 3 bytes)       |
| heapPages          (u16 - 2 bytes)       |
| stackSize          (u24 - 3 bytes)       |
| roData             (roLength bytes)      |
| rwData             (rwLength bytes)      |
| codeLength         (u32 - 4 bytes)       |
| code               (PVM program blob)    |
+------------------------------------------+
```

**Initial Register State:**
| Register | Value | Purpose |
|----------|-------|---------|
| r0 | 0xFFFF_0000 | EXIT address — jump here to HALT |
| r1 | 0xFEFE_0000 | Stack pointer (STACK_SEGMENT_END) |
| r2-r6 | 0 | Available for computation |
| r7 | 0xFEFF_0000 | Arguments pointer (IN) / Result address (OUT) |
| r8 | args.length | Arguments length (IN) / Result length (OUT) |
| r9-r12 | 0 | Available for parameters/locals |

**Program Termination:**
- HALT: `LOAD_IMM r2, -65536; JUMP_IND r2, 0` → jumps to 0xFFFF0000 → status=HALT
- Note: Don't rely on r0 containing EXIT — hardcode 0xFFFF0000 (= -65536 as i32)
- PANIC: `TRAP` instruction → status=PANIC

---

## Memory Layout

```
  Address          Region                    Access
 ----------------------------------------------------------
 0x0000_0000  +-------------------------+
              |   Reserved / Guard      |   None    (64 KB)
 0x0001_0000  +-------------------------+
              |   Read-Only Data (RO)   |   Read
 0x0002_0000+ +-------------------------+
              |   Read-Write Data (RW)  |   Write
              + - - - - - - - - - - - - +
              |   Heap (Zero-init)      |   Write
              +-------------------------+
              |                         |
              |    Unmapped / Guard     |   None
              |                         |
 stackStart   +-------------------------+
              |        Stack            |   Write   (grows down)
 0xFEFE_0000  +-------------------------+  (STACK_SEGMENT_END)
              |   Guard (64 KB)         |   None
 0xFEFF_0000  +-------------------------+  (ARGS_SEGMENT_START)
              |   Arguments (RO)        |   Read    (up to 16 MB)
              +-------------------------+
              |   Guard (64 KB)         |   None
 0xFFFF_FFFF  +-------------------------+
```

### Compiler Memory Regions

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data (dispatch table for `call_indirect`) |
| `0x30000` | Globals storage |
| `0x30100` | User heap (result storage) |
| `0x3FF00` | Parameter overflow area (5th+ args) |
| `0x40000` | Spilled locals (512 bytes per function) |
| `0x50000+` | WASM linear memory base (data sections placed here) |

Spilled local address: `0x40000 + (func_idx * 512) + ((local_idx - 4) * 8)`

---

## Indirect Calls (`call_indirect`)

### Dispatch Table (RO Memory)
At `0x10000`, dispatch table entries are 8 bytes:
```
[0..3]  jump address (u32)
[4..7]  type index (u32)
```

At runtime, `call_indirect` loads the type index from offset 4 and traps if it doesn't match the expected type.

### Jump Table Extension
```
jumpTable = [ret_addr_0, ret_addr_1, ..., func_offset_0, func_offset_1, ...]
              ^- for return from calls    ^- for indirect calls (func_entry_base)
```

---

## Division Edge Cases

PVM follows RISC-V semantics for division (returns specific values instead of trapping). WASM requires traps for:
- **Division by zero**: All div/rem operations
- **Signed overflow**: `INT_MIN / -1` for `i32.div_s`

The compiler currently relies on PVM hardware behavior for these edge cases.

---

## AssemblyScript Note

AssemblyScript applies `& 0xFF` mask to the result of u8 arithmetic, even when assigning to a u32 variable:

```typescript
// BUG: 128 + 159 = 287 & 0xFF = 31
result = arr[0] + arr[1];

// FIX: cast to u32 first
result = <u32>arr[0] + <u32>arr[1];  // = 287
```

When summing `Uint8Array` elements where the result may exceed 255, always cast to u32/i32 first.

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
- r0: Return address (jump table offset)
- r1: Stack pointer
- r2-r6: Operand stack (5 slots)
- r7: Return value / SPI args pointer
- r8: SPI args length / saved table idx
- r9-r12: Local variables (first 4)

### What Doesn't Work (Architectural Debt)

#### 1. No Intermediate Representation

**Problem**: WASM is translated directly to PVM machine code.

```text
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

**Better Structure** (see issue #30):

```text
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

**Current Status**: Fixed. The remaining PANIC was caused by a separate LLVM PHI validation issue (see Entry 3).

**Lesson**:
- Memory operations are critical - any bug causes cascading failures
- AS runtime internals (unshift, copy) affect generated code
- Need to test with AS runtime patterns, not just WAT

---

## Entry 3: LLVM PHI Node Validation Error (2026-02-16)

**Symptom**: Compilation fails with LLVM verification error when compiling complex WASM programs (e.g., anan-as compiler):

```text
PHINode should have one entry for each predecessor of its parent basic block!
  %fn_result = phi i64 [ undef, %fn_return ]
```

**Root Cause**: A workaround in `function_builder.rs` attempted to add an `undef` value to the function return PHI when it had no incoming values. However, it incorrectly used `merge_bb` (the block containing the PHI) as the "from" block, which is invalid LLVM IR.

**LLVM PHI Semantics**:
- A PHI node's incoming entries must reference actual predecessor basic blocks
- The "from" block in `phi.add_incoming(&[(&value, from_bb)])` must be a block that branches TO the PHI's block
- You cannot use the PHI's own block as a "from" block

**Fix**: Removed the broken workaround code. The WASM-to-LLVM translation already correctly adds values to the PHI for all code paths that need them, so the workaround was unnecessary.

**Code Location**: `crates/wasm-pvm/src/llvm_frontend/function_builder.rs`, lines 893-897 (removed)

**Lesson**:
- Workarounds for edge cases can introduce worse bugs than the original issue
- LLVM IR validation errors often indicate semantic misunderstandings, not just syntax issues
- The WASM-to-LLVM control flow translation was already correct; the "fix" was the bug

---

## Entry 4: Local Variable Zero-Init (2026-02-06)

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

1. **Extract constants**: Create MemoryLayout abstraction (see issue #31)
2. **Fix warnings**: Remove clippy suppressions
3. **Add tests**: StackMachine, spilling, control flow (see issue #32)
4. **Document thoroughly**: Every register usage, every memory address (see issue #34)

## Long Term (V2 Architecture)

5. **Add IR**: SSA-based intermediate representation
6. **Split codegen**: Separate modules per responsibility
7. **Proper allocator**: Graph coloring or linear scan
8. **Optimizations**: Constant folding, DCE, CSE
9. **CFG**: Control flow graph with dominance analysis

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

**Recommendation**: See tracking GitHub issues #30-#40 for comprehensive testing plan.

---

## References

- Original: LEARNINGS.md (technical details)
- Architecture Review: review/findings/ (critical analysis)
- Architecture & Planning: See tracking GitHub issues #30-#40
