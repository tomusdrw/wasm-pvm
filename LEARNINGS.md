# WASM to PVM Recompiler - Learnings & Knowledge Base

**Version**: 2.0 (With Architecture Review Insights)
**Date**: 2026-02-10

This document captures technical learnings, design decisions, and discoveries made during the development of the WASM to PVM recompiler.

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
- halt - Normal termination
- panic - Error/trap
- out-of-gas - Gas exhausted
- page-fault - Memory access violation
- host-call - External function call

---

## Architecture Insights (From Code Review - 2026-02-09)

### Current Architecture Assessment

**What Works**:
- WASM to PVM conversion is straightforward and works for most programs
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

### Architectural Debt (V2 Scope)

1. **No Intermediate Representation**: Direct WASM to PVM translation. Works for V1 but prevents optimizations and makes debugging harder.
2. **Monolithic Code Generation**: `codegen.rs` is 2,400 lines handling everything (operator dispatch, control flow, function calls, memory ops, register allocation).
3. **Ad-Hoc Register Allocation**: Hardcoded register usage with magic numbers, no liveness analysis.
4. **Manual Spilling**: 3 nested conditionals, side effects, hard to test all branches.
5. **Manual Control Flow**: No CFG, manual label allocation and fixup resolution.

### The Compiler Works, But...

The current architecture is a **direct translator**, not a full compiler:

| Aspect | Translator (Current) | Compiler (V2) |
|--------|-----------|----------|
| Structure | Single pass | Multiple phases |
| Optimization | None | Many passes |
| Debugging | Hard (trace PVM) | Easy (inspect IR) |
| Correctness | Test-only | Verify + test |
| Extensibility | Poor | Excellent |

**The trade-off**: Translator is simpler to write initially, but hits limits. For V1, this is acceptable. For V2 (optimizations, complex features), need proper compiler architecture.

---

## Stack Overflow Detection (Phase 13)

**Implementation**: Added stack overflow detection to prevent silent memory corruption during deep recursion.

**Mechanism**:
1. Calculate `new_sp = sp - frame_size` before each function call
2. Compare against `stack_limit = STACK_SEGMENT_END - stack_size` (default 64KB)
3. Use unsigned comparison (`BranchGeU`) to handle high addresses like `0xFEEE0000`
4. Emit `TRAP` instruction on overflow, causing PANIC status

**Code Locations**:
- `codegen.rs:390-424` - Stack overflow check in `emit_call()`
- `codegen.rs:597-639` - Stack overflow check in `emit_call_indirect()`

**Configuration**:
- Default stack size: 64KB (configurable in SPI format up to 16MB)
- With ~40-byte frames, overflow occurs at ~1600 recursion depth
- Stack grows downward from `0xFEFE0000`

---

## Division Overflow & Import Return Fixes (Phase 19a - 2026-02-09)

**Problem 1 - Division overflow**: WASM spec requires trap for division by zero and `INT_MIN / -1` (signed overflow). PVM hardware doesn't trap on these - it returns garbage values instead.

**Fix**: Added checks before all 8 div/rem operations:
- **Div-by-zero**: `BranchNeImm(divisor, 0, ok) + Trap` for all div/rem ops
- **Signed overflow**: `BranchNeImm(dividend, INT_MIN, ok) + BranchNeImm(divisor, -1, ok) + Trap` for i32.div_s
- **i64 signed overflow**: Since `i64::MIN` doesn't fit in a 32-bit immediate, uses `LoadImm64 + Xor` approach with fast path (skip if divisor != -1)

**Problem 2 - Import returns**: Imported function stubs popped arguments but didn't push return values, causing stack underflow.

**Fix**: Added `spill_push() + LoadImm(0)` when import signature has a return type.

**Key Insight**: PVM follows RISC-V semantics for division edge cases (returns specific values instead of trapping), so WASM-level checks are always required.

**Code Locations**: `codegen.rs` helper methods `emit_div_by_zero_check`, `emit_i32_signed_div_overflow_check`, `emit_i64_signed_div_overflow_check`

**Tests**: `tests/division_checks.rs` (8 tests), `tests/import_returns.rs` (4 tests)

---

## Local Variable Zero-Initialization Bug (Phase 16d - 2026-02-06)

**Problem**: WASM spec requires all local variables to be zero-initialized, but wasm-pvm was not initializing non-parameter locals.

**Symptom**: Loop counters starting with garbage values, causing loops to not execute correctly.

**Fix**: Added `LoadImm { reg, value: 0 }` instructions in `emit_prologue()` for all non-parameter locals that fit in registers.

**Code Location**: `crates/wasm-pvm/src/translate/codegen.rs` lines 924-937

---

## AssemblyScript u8 Arithmetic Semantics (Phase 16e - 2026-02-06)

**Problem**: AssemblyScript tests for complex allocations were failing with a consistent 256-byte difference.

**Root Cause**: AssemblyScript applies `& 0xFF` mask to the result of u8 arithmetic, even when assigning to a u32 variable.

```typescript
// BUG: 128 + 159 = 287 & 0xFF = 31
result = arr[0] + arr[1];

// FIX: cast to u32 first to avoid the mask
result = <u32>arr[0] + <u32>arr[1];  // = 287
```

**Impact**: Any u8 addition that exceeds 255 will wrap incorrectly. This is NOT a wasm-pvm bug - it's AS type semantics.

**Lesson**: When summing Uint8Array elements in AssemblyScript where the result may exceed 255, always cast to u32/i32 first.

---

## PVM-in-PVM Implementation (Phase 16b)

**Architecture**:
- **Outer PVM**: Compiled `pvm-runner.ts` (135KB PVM bytecode) - minimal interpreter for SPI programs
- **Inner Programs**: SPI format programs executed by the outer PVM
- **Execution Chain**: Test Script -> anan-as CLI -> PVM Runner -> SPI Program -> Results

**Current Capabilities**:
- Basic arithmetic operations (add, factorial, fibonacci, gcd) infrastructure ready
- Full PVM-in-PVM test pipeline exists
- All existing example programs can be tested through PVM-in-PVM
- **Blocker**: Inner interpreter PANICs (BUG-4) - see KNOWN_ISSUES.md

**Argument Passing**:
- anan-as generic programs don't accept CLI arguments
- Workaround: Embed arguments in SPI format within PVM runner memory
- Format: `[spi_program_length: u32][spi_program_data][input_args...]`

---

## PVM-in-PVM Investigation (2025-01-19)

### Problem Summary
When running anan-as (compiled to PVM) as an interpreter for an inner PVM program, execution fails with a FAULT at PC 1819, attempting to access memory address ~170MB (0x0A218A68).

### Root Cause Analysis

**Symptom:** A `LOAD_IND_U32` instruction tries to read from an extremely large address.

**Trace Analysis:**
1. At PC 160515, a `MUL_32` computes `r4 = 196716 * 196716 = 42,478,992` (32-bit wrapped)
2. The value 196716 (0x3006C) is the **address** of global 27 (`__heap_base`), not its **value** (54292)
3. This corrupted value propagates through array indexing: `base + index * 4`
4. Eventually leads to accessing address 0x0A218A68

**Key Finding:** Something stores the global ADDRESS (0x3006C) into WASM linear memory instead of loading the global VALUE (54292) and storing that.

### Suspected Causes

1. **AS Runtime Class Pointers**: AssemblyScript stores class metadata pointers in objects. If a global address is accidentally stored where a class pointer should be, method dispatch or field access will compute garbage addresses.
2. **Memory Initialization Issue**: The rw_data section initializes both globals (at 0x30000) and WASM data sections (at 0x50000+). If there's overlap or misalignment, global addresses could leak into WASM memory.
3. **64-bit vs 32-bit Confusion**: PVM uses 64-bit registers, WASM uses 32-bit addresses. Sign extension or truncation issues could produce unexpected values.

---

## Code Quality Improvements (Phase 19b - 2026-02-09)

### Dead Code Removal & Clippy Fixes
- Removed dead `check_for_floats()` and `is_float_op()` functions from `mod.rs`
- Removed unused `FunctionBody` import
- Fixed all `doc_markdown` clippy warnings
- Cast-related suppressions retained with comments (intentional for compiler code)

### Debug Assertions for Stack Invariants
- Added `MAX_STACK_DEPTH` (128) bound check in `push()`
- Added depth bound check in `set_depth()`
- Added register range assertions in `reg_at_depth()`, `reg_for_depth()`
- Added spill depth precondition in `spill_offset()`
- Added register/depth assertions in codegen `spill_push()`/`spill_pop()`

### Memory Layout Module
- Extracted all PVM memory address constants from `codegen.rs` into `translate/memory_layout.rs`
- Single source of truth for: `GLOBAL_MEMORY_BASE`, `SPILLED_LOCALS_BASE`, `STACK_SEGMENT_END`, `EXIT_ADDRESS`, `RO_DATA_BASE`, `PARAM_OVERFLOW_BASE`, `OPERAND_SPILL_BASE`, `DEFAULT_STACK_SIZE`
- Helper functions: `compute_wasm_memory_base()`, `memory_size_global_offset()`, `spilled_local_addr()`, `global_addr()`, `stack_limit()`
- Module-level ASCII art diagram of the PVM address space layout

---

## SPI (Standard Program Interface) Format

Source: `vendor/anan-as/assembly/spi.ts`

**Binary Format:**
```
+------------------------------------------+
| roLength           (u24 - 3 bytes)       |
+------------------------------------------+
| rwLength           (u24 - 3 bytes)       |
+------------------------------------------+
| heapPages          (u16 - 2 bytes)       |
+------------------------------------------+
| stackSize          (u24 - 3 bytes)       |
+------------------------------------------+
| roData             (roLength bytes)      |
+------------------------------------------+
| rwData             (rwLength bytes)      |
+------------------------------------------+
| codeLength         (u32 - 4 bytes)       |
+------------------------------------------+
| code               (PVM program blob)    |
+------------------------------------------+
```

**Memory Layout (32-bit address space):**
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

**Initial Register State (SPI):**
| Register | Value | Purpose |
|----------|-------|---------|
| r0 | 0xFFFF_0000 | EXIT address - jump here to HALT |
| r1 | STACK_SEGMENT_END (0xFEFE_0000) | Stack pointer |
| r2-r6 | 0 | Available for computation |
| r7 | ARGS_SEGMENT_START (0xFEFF_0000) | Arguments pointer (IN) / Result address (OUT) |
| r8 | args.length | Arguments length (IN) / Result length (OUT) |
| r9-r12 | 0 | Available for parameters/locals |

**Program Termination:**
- HALT: `LOAD_IMM r2, -65536; JUMP_IND r2, 0` -> jumps to 0xFFFF0000 -> status=HALT
- Note: Don't rely on r0 containing EXIT - hardcode 0xFFFF0000 (= -65536 as i32)
- PANIC: `TRAP` instruction -> status=PANIC

---

## PVM Instruction Encoding

### ThreeReg Instructions (CRITICAL!)

For ThreeReg instructions (ADD_32, SUB_32, etc.):
- Encoding: `[opcode, src1<<4 | src2, dst]`
- **Execution: reg[c] = op(reg[b], reg[a])** - Note the swap!

This means for `SET_LT_U`, it computes `dst = (src2 < src1)`, NOT `dst = (src1 < src2)`.

**Fix for WASM translation**: Swap operand order for comparison and division ops.

### TwoRegOneImm Encoding Details
**Critical:** High nibble (args.a) is typically the SOURCE, low nibble (args.b) is the DESTINATION.

```
Byte layout: [opcode] [src << 4 | dst] [imm...]

Example ADD_IMM_32: regs[dst] = regs[src] + imm
  Encoding: [131] [src << 4 | dst] [imm_bytes...]

Example LOAD_IND_U32: regs[dst] = memory[regs[base] + offset]
  Encoding: [128] [base << 4 | dst] [offset_bytes...]

Example STORE_IND_U32: memory[regs[base] + offset] = regs[src]
  Encoding: [122] [base << 4 | src] [offset_bytes...]
```

---

## WASM to PVM Mapping Strategy

### Arithmetic Operations
| WASM | PVM |
|------|-----|
| i32.add | ADD_32 rD, rA, rB |
| i64.add | ADD_64 rD, rA, rB |
| i32.const N | LOAD_IMM rD, N (or LOAD_IMM_64 for large) |

### Control Flow
| WASM | PVM Strategy |
|------|--------------|
| block | Label for forward branch |
| loop | Label for backward branch |
| br N | JUMP to Nth enclosing label |
| br_if N | BRANCH_NE_IMM + condition check |
| if/else/end | BRANCH + JUMP combination |

### Locals/Stack
**Challenge:** WASM has unlimited stack + locals, PVM has 13 registers.

**Strategy:**
1. Use r2-r6, r9-r12 for operand stack and locals (9 registers)
2. Spill to stack memory when needed (use r1 as stack pointer)
3. Track stack depth at each instruction

### Memory Operations
| WASM | PVM |
|------|-----|
| i32.load offset=N | LOAD_IND_U32 rD, rBase, N (with WASM_MEMORY_BASE offset) |
| i32.store offset=N | STORE_IND_U32 rBase, rVal, N (with WASM_MEMORY_BASE offset) |
| memory.size | Load from compiler-managed global |
| memory.grow(n) | Check limits, update global, return old size or -1 |
| memory.fill | Loop: store byte, increment dest, decrement count |
| memory.copy | Loop with backward path when dest > src (memmove semantics) |

**Address Translation:**
- WASM addresses start at 0
- PVM addresses < 0x10000 cause panic
- WASM linear memory base: 0x50000 (WASM_MEMORY_BASE)
- WASM data sections placed at 0x50000 + offset in rw_data

---

## Design Decisions

### Register Allocation
| Registers | Usage |
|-----------|-------|
| r0 | Return address (jump table index for calls) |
| r1 | Stack pointer (for call stack, grows down from 0xFEFE0000) |
| r2-r6 | Operand stack (5 slots) |
| r7 | Return value from function calls / SPI args_ptr (in main) |
| r8 | SPI args_len (in main) |
| r9-r12 | Local variables (first 4 locals) |

### Calling Convention
**Before call (caller-side):**
1. Calculate frame size: 40 bytes (r0 + r9-r12) + 8 bytes per operand stack slot below arguments
2. Decrement stack pointer by frame size
3. Save return address (r0) to [sp+0]
4. Save locals r9-r12 to [sp+8..40]
5. Save caller's operand stack values (those below the arguments) to [sp+40+]
6. Pop arguments from operand stack and copy to callee's local registers (r9+)
7. Load return address (jump table index) into r0
8. Jump to callee's entry point

**After return (caller-side):**
1. Restore return address (r0) from [sp+0]
2. Restore locals r9-r12 from [sp+8..40]
3. Restore operand stack values from [sp+40+]
4. Increment stack pointer by frame size
5. Copy return value from r7 to operand stack

---

## Jump Table Mechanism (CRITICAL for function calls)

PVM's `JUMP_IND` instruction does **NOT** jump directly to the address in the register. Instead, it uses a **jump table** lookup.

```
JUMP_IND rA, offset
  target_address = jumpTable[(rA + offset) / 2 - 1]
  jump to target_address
```

- Register value is NOT a PC, it's a **jump table reference**
- Value `2` refers to `jumpTable[0]`
- Value `4` refers to `jumpTable[1]`
- Value `2*(N+1)` refers to `jumpTable[N]`
- **Exception**: Value `0xFFFF0000` (EXIT address) is special-cased for program termination

---

## Local Variable Spilling

When a function has more than 4 local variables (including parameters), extras are spilled to memory.

```
Base address: 0x40000 (SPILLED_LOCALS_BASE, above user heap)
Per function: 512 bytes (SPILLED_LOCALS_PER_FUNC)
Spilled local address = 0x40000 + (func_idx * 512) + ((local_idx - 4) * 8)
```

| Local Index | Storage |
|-------------|---------|
| 0-3 | Registers r9-r12 |
| 4+ | Memory at spilled address |

---

## Indirect Calls (call_indirect)

WASM `call_indirect` allows calling functions through a table using a runtime index.

### Dispatch Table (RO Memory)
At 0x10000, dispatch table entries are 8 bytes:
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

## Design Decisions Revisited (Post-Review)

### No Intermediate Representation
**Original**: Direct WASM to PVM for simplicity
**Current**: Works but limits extensibility
**Revised Recommendation**: Add IR for V2, keep direct translation for V1

### Hardcoded Register Allocation
**Original**: Simple, predictable
**Current**: Works but fragile
**Revised Recommendation**: Keep for V1, proper allocator for V2

### Stack-based Translation
**Original**: Direct WASM stack to PVM registers
**Current**: Works with careful spilling
**Revised Recommendation**: Keep for V1, consider stack-to-SSA for V2

---

## Testing Strategy Insights

**Current**: 62 integration tests, ~50 Rust tests
**Gap**: No tests for stack spilling at various depths, complex AS patterns
**Recommendation**: See review/proposals/07-testing-strategy.md for comprehensive testing plan.

---

## References

- [Gray Paper v0.7.2](./gp-0.7.2.md) - JAM protocol specification
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/) - WASM specification
- Architecture Review: review/findings/ (critical analysis)
- Proposed Architecture: review/proposals/06-proposed-architecture.md
- Testing Strategy: review/proposals/07-testing-strategy.md
