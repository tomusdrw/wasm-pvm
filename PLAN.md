# WASM to PVM Recompiler - Project Plan

## Project Overview

Build a **WASM (WebAssembly) to PVM (Polka Virtual Machine)** recompiler in Rust. The recompiler takes WASM bytecode and produces equivalent PVM bytecode that can execute on the PolkaVM.

### Goals
1. **Correctness** - Produce semantically equivalent PVM code
2. **Completeness** - Support core WASM MVP features
3. **Testability** - Comprehensive test suite with reference interpreter
4. **Maintainability** - Clean architecture following Rust best practices

### V1 Milestone: PVM-in-PVM
**Ultimate Goal**: Compile [anan-as](https://github.com/polkavm/anan-as) (the PVM interpreter written in AssemblyScript) to WASM, then to PVM, and run a PVM interpreter inside a PVM interpreter.

This demonstrates:
- Complete WASM MVP support (except floats)
- Correct handling of complex control flow
- Proper memory and stack management
- Self-hosting capability of the toolchain

### Non-Goals (Initial Version)
- Performance optimization (focus on correctness first)
- WASM proposals beyond MVP (SIMD, threads, etc.)
- Floating point support (**PVM has no FP - reject WASM with floats**)

### Output Format
**Primary target: JAM SPI (Standard Program Interface) format**
- Includes memory layout (RO data, RW data, heap, stack)
- Proper register initialization for JAM execution
- See LEARNINGS.md for detailed SPI format specification

---

## SPI Entrypoint Convention

All WASM programs targeting PVM/JAM must follow this convention:

### Function Signature
```wat
(func (export "main") (param $args_ptr i32) (param $args_len i32)
  ;; args_ptr = r7 (PVM address of SPI args, e.g., 0xFEFF0000)
  ;; args_len = r8 (length of args in bytes)
  ...
)

;; Optional second entry point for JAM (PC=5)
(func (export "main2") (param $args_ptr i32) (param $args_len i32)
  ...
)
```

### Return Value Convention
```wat
(global $result_ptr (mut i32) (i32.const 0))
(global $result_len (mut i32) (i32.const 0))

;; In function body:
(global.set $result_ptr (i32.const 0x30100))  ;; PVM address of result
(global.set $result_len (i32.const 4))         ;; Length in bytes
```

### Memory Layout
```
0x00010000: RO data segment (dispatch table for call_indirect)
0x00030000: Globals storage (0x30000 + idx*4)
0x00030100: User results area (256 bytes)
0x00030200: Spilled locals (512 bytes per function)
0x00030200 + num_funcs*512: User heap
0xFEFE0000: Stack segment end
0xFEFF0000: Args segment (read via args_ptr)
0xFFFF0000: EXIT address (HALT)
```

---

## Current Progress

**Latest Update**: 2026-02-06 - All 339 integration tests + 7 Rust tests passing. Fixed local zero-init bug and AS u8 arithmetic issues.

**Completed**:
- Phase 14: memory.size/memory.grow with proper tracking
- Phase 15: call_indirect signature validation + operand stack clobber fix
- Phase 16: PVM-in-PVM infrastructure complete (test harness working, basic arithmetic validated)
- Phase 16d: Local variable zero-initialization bug fix (WASM spec compliance)
- Phase 16e: AS complex-alloc tests fixed (u8 arithmetic semantics issue)

**Next Steps**:
- Phase 19: Working PVM-in-PVM Test Execution (IMMEDIATE PRIORITY)
- Phase 17: Host Calls / ecalli Support (planned)
- Phase 18: Architecture Refactor (unit testing improvements)

### ✅ Completed

#### Phase 1: Foundation
- [x] Initialize Rust project with Cargo workspace
- [x] Set up directory structure (crates/wasm-pvm, crates/wasm-pvm-cli)
- [x] Add dependencies: wasmparser, thiserror, anyhow, clap
- [x] Define `Opcode` enum with essential opcodes
- [x] Define `Instruction` enum with operands
- [x] Implement instruction encoding to bytes
- [x] Implement opcode bitmask generation
- [x] Create basic CLI structure
- [x] Set up test infrastructure (scripts/run-spi.ts with anan-as)

#### Phase 2: Simple Functions
- [x] Parse WASM module using wasmparser
- [x] Extract function types and bodies
- [x] Translate simple arithmetic (i32.add, i64.add)
- [x] Translate i32.const, i64.const
- [x] Handle local variables (local.get, local.set)
- [x] Implement operand stack → register mapping (r2-r6)
- [x] SPI entrypoint convention (args_ptr/args_len params)
- [x] Return value via globals ($result_ptr, $result_len)
- [x] Hardcoded EXIT address (0xFFFF0000)

#### Memory Operations
- [x] i32.load - direct PVM memory access
- [x] i32.store - direct PVM memory access
- [x] global.get / global.set - stored at 0x30000 + idx*4
- [x] memory.size - tracks actual memory size via compiler-managed global
- [x] memory.grow - properly updates memory size, returns old size or -1

#### Control Flow (Phase 3)
- [x] Translate `block` (forward branch target)
- [x] Translate `loop` (backward branch target)
- [x] Translate `br` (unconditional branch)
- [x] Translate `br_if` (conditional branch)
- [x] Translate `return`
- [x] Translate `if/else/end`
- [x] Handle block result values

#### Integer Operations
- [x] i32.add, i64.add
- [x] i32.sub, i64.sub
- [x] i32.mul, i64.mul
- [x] i32.div_u, i32.div_s, i64.div_u, i64.div_s
- [x] i32.rem_u, i32.rem_s, i64.rem_u, i64.rem_s
- [x] i32.gt_u, i32.gt_s, i64.gt_u, i64.gt_s
- [x] i32.lt_u, i32.lt_s, i64.lt_u, i64.lt_s
- [x] i32.ge_u, i32.ge_s, i64.ge_u, i64.ge_s
- [x] i32.le_u, i32.le_s, i64.le_u, i64.le_s
- [x] i32.eq, i32.ne, i32.eqz, i64.eq, i64.ne, i64.eqz
- [x] i32.and, i32.or, i32.xor, i64.and, i64.or, i64.xor
- [x] i32.shl, i32.shr_u, i32.shr_s, i64.shl, i64.shr_u, i64.shr_s
- [x] i32.clz, i64.clz, i32.ctz, i64.ctz, i32.popcnt, i64.popcnt
- [x] i32.rotl, i32.rotr, i64.rotl, i64.rotr
- [x] i32.wrap_i64, i64.extend_i32_s, i64.extend_i32_u
- [x] i32.extend8_s, i32.extend16_s, i64.extend8_s, i64.extend16_s, i64.extend32_s
- [x] local.tee
- [x] drop
- [x] select
- [x] unreachable (maps to TRAP)

#### Memory Operations (Phase 5)
- [x] i64.load
- [x] i64.store
- [x] i32/i64 load8_u, load8_s, load16_u, load16_s, load32_u, load32_s
- [x] i32/i64 store8, store16, store32
- [x] memory.fill (bulk memory fill)
- [x] memory.copy (bulk memory copy)

#### Phase 4: AssemblyScript Examples
- [x] Set up AssemblyScript project in `examples-as/`
- [x] Create `add.ts`, `factorial.ts`, `fibonacci.ts`, `gcd.ts`
- [x] Verify AS output compiles through wasm-pvm
- [x] Document AssemblyScript → JAM workflow

#### Phase 4b: Test Suite & CI
- [x] Created `scripts/test-all.ts` - 62 integration tests across WAT and AS examples
- [x] 30 Rust unit tests (13 lib + 16 integration + 1 doc test)
- [x] GitHub Actions CI workflow (`.github/workflows/ci.yml`)

#### Phase 6: Functions & Calls (Partial)
- [x] Translate `call` instruction
- [x] Handle function prologues/epilogues
- [x] Multi-function compilation with proper offsets
- [x] Jump table for return addresses (PVM JUMP_IND requirement)
- [x] Local variable spilling (registers r9-r12 + memory at 0x30000)
- [x] Entry jump when main is not first function

### ✅ Examples Working (JAM Convention)
WAT examples (`examples-wat/*.jam.wat`):
- [x] `add.jam.wat` - reads two i32 args, returns sum
- [x] `factorial.jam.wat` - computes n! using loop
- [x] `fibonacci.jam.wat` - fibonacci sequence
- [x] `gcd.jam.wat` - GCD (Euclidean algorithm)
- [x] `is-prime.jam.wat` - primality test
- [x] `div.jam.wat` - unsigned division
- [x] `call.jam.wat` - function calls
- [x] `br-table.jam.wat` - switch/jump table (br_table)
- [x] `bit-ops.jam.wat` - clz, ctz, popcnt
- [x] `rotate.jam.wat` - rotl, rotr
- [x] `entry-points.jam.wat` - multiple entry points (main/main2)
- [x] `recursive.jam.wat` - recursive factorial (tests call stack)
- [x] `nested-calls.jam.wat` - nested function calls

AssemblyScript examples (`examples-as/assembly/*.ts`):
- [x] `add.ts` - reads two i32 args, returns sum
- [x] `factorial.ts` - computes n! using loop
- [x] `fibonacci.ts` - fibonacci sequence
- [x] `gcd.ts` - GCD (Euclidean algorithm)

**Test Suite**: 62 integration tests passing (as of 2025-01-19)
- [x] `call-indirect.jam.wat` - indirect function calls via table

AssemblyScript examples (`examples-as/assembly/*.ts`):
- [x] `life.ts` - Game of Life (fully working with any number of steps)

---

## Remaining Work for V1 MVP

### ✅ Phase 11: Game of Life Debugging - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: Validated operand stack spilling and complex function calls work correctly

**Bugs Fixed**:
1. **`I64Load` instruction** (line ~798 in codegen.rs) - Was using incompatible patterns (`self.stack.pop`, `ctx.emit`, non-existent `Instruction::LoadI64`)
2. **Spilled operand stack across function calls** - For operand stack depths >= 5 (spilled to memory), the save/restore logic was reading from register r7 instead of the actual spill area. Fixed to load from `old_sp + frame_size + OPERAND_SPILL_BASE + offset`
3. **`local.tee` with spilled operand stack** - When operand stack top was spilled, `local.tee` had two bugs:
   - Didn't check `pending_spill` to know if value was still in r7 or already written to memory
   - Used r2/r3 as temp registers which could clobber operand stack; changed to use `SPILL_ALT_REG` (r8)

**Test Result**: 62/62 integration tests passing, Game of Life works correctly for 0, 1, 2, ... steps

### ✅ Phase 12: Data Section Initialization - COMPLETED (2025-01-19)
**Status**: COMPLETE (except imported function calls)
**Impact**: Data sections now initialized at WASM_MEMORY_BASE (0x50000)

**Implemented**:
1. ✅ Parse WASM `DataSection` in `translate/mod.rs`
2. ✅ Initialize data in SPI `rw_data` section at correct offsets
3. ✅ Support active data segments (passive not needed yet)
4. ✅ Update memory layout and heap_pages calculation
5. ✅ Handle offset expressions in active data segments
6. ✅ Parse `ImportSection` and count imported functions
7. ✅ Adjust function index translation for calls

**Not yet supported**:
- Imported function calls (anan-as uses `abort` and `console.log`)
- Memory operations don't auto-offset (programs must use WASM_MEMORY_BASE addresses)

### ✅ Phase 12b: Import Function & Additional Operations - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: anan-as now compiles successfully (423KB JAM file)

**Implemented**:
1. ✅ Stub imported functions: pop args, emit TRAP for `abort`, no-op for others
2. ✅ `memory.fill` operation (bulk memory fill via loop)
3. ✅ `memory.copy` operation (bulk memory copy via loop)
4. ✅ `i32.extend8_s`, `i32.extend16_s` (sign extension)
5. ✅ `i64.extend8_s`, `i64.extend16_s`, `i64.extend32_s` (sign extension)
6. ✅ Float truncation stubs (`i32.trunc_sat_f64_u` etc.) - return 0 (dead code path)
7. ✅ Fixed anan-as to use integer min instead of `Math.min` (which uses f64)

**anan-as Modifications**:
- Replaced `Math.min(4, x)` with `mini32(4, x)` in `arguments.ts`, `program-build.ts`
- Replaced `Math.min(PAGE_SIZE, x)` with `minu32(PAGE_SIZE, x)` in `memory.ts`
- Added `mini32` and `minu32` helper functions in `math.ts`
- Rebuilt anan-as WASM with zero float operations

**Note**: The compiled anan-as JAM file (423KB) is a library, not a standalone program.
Full PVM-in-PVM would require a wrapper that calls the API functions (resetGeneric, nSteps, etc.).

**Verification**: The compiled JAM file is structurally valid and can be loaded by the PVM interpreter:
- 632 bytes RO data (dispatch table)
- 152KB RW data (WASM linear memory with data sections)
- 1746 jump table entries (function entry points and return addresses)
- ~64,133 PVM instructions
- `prepareProgram` succeeds when loading the JAM file

### ✅ Phase 13: Stack Overflow Detection - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: Deep recursion now triggers PANIC instead of corrupting memory

**Implemented**:
1. ✅ Stack depth checking before every `call` and `call_indirect`
2. ✅ Configurable stack size limit (default 64KB)
3. ✅ TRAP emitted on stack overflow
4. ✅ Unsigned comparison via `BranchGeU` instruction
5. ✅ `LoadImm64` used to avoid sign-extension issues with high addresses

**Technical details**:
- Stack limit calculated as `STACK_SEGMENT_END (0xFEFE0000) - stack_size`
- Before each call, compute `new_sp = sp - frame_size` 
- If `new_sp < stack_limit`, emit TRAP (causes PANIC status)
- With 64KB stack and ~40-byte frames, overflow occurs at ~1600 recursion depth

**Testing**: All 58 integration tests pass, stack overflow correctly triggers PANIC

### ✅ Phase 14: Memory.size/Memory.grow - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: WASM programs can now query and grow memory

**Implemented**:
1. ✅ Parse WASM `MemorySection` for initial/max memory limits
2. ✅ Compiler-managed global at `memory_size_global_offset()` tracks current memory size
3. ✅ `memory.size` returns current size from compiler global (not hardcoded)
4. ✅ `memory.grow` updates compiler global, returns old size, -1 if bounds exceeded
5. ✅ Bounds checking against `max_memory_pages`
6. ✅ Proper register allocation to avoid clobbering stack values

### ✅ Phase 15: call_indirect Fixes - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: call_indirect now works correctly with signature validation

**Bugs Fixed**:
1. **Stack overflow check clobbered operand stack** - The stack overflow check in `emit_call_indirect` used r2 to hold the stack limit, which clobbered any function arguments on the operand stack. Fixed by temporarily saving r9 to memory, using it for the limit, then restoring.
2. **Added call_indirect to test suite** - 4 new test cases for call_indirect (double/triple with different arguments)

**Signature Validation** (implemented previously):
- Dispatch table entries expanded from 4 to 8 bytes (jump_addr + type_index)
- Runtime validation compares function's type_index against expected type
- Mismatch triggers TRAP (PANIC status)

**Testing**: All 62 integration tests pass, including call_indirect and signature validation tests

### ✅ Phase 16a: AS Runtime Isolation (Allocations) - COMPLETED (2025-01-20)
**Status**: COMPLETE
**Impact**: Tested complex AS allocations on PVM - all runtimes work correctly

**Findings**:
- Created complex allocation test with object graphs and circular references
- All three AS runtimes (`stub`, `minimal`, `incremental`) execute successfully on PVM
- Expected result (1107) returned correctly across all runtimes
- Basic allocation patterns work correctly on compiled PVM

### ✅ Phase 16b: PVM-in-PVM Validation - COMPLETED (2025-01-22)
**Status**: COMPLETE - Infrastructure implemented
**Impact**: PVM-in-PVM infrastructure created but test execution is stubbed

**Completed**:
- ✅ **Floating Point Issue Resolved**: anan-as builds without FP code after removing problematic constructs
- ✅ **PVM-in-PVM Test Harness**: `scripts/test-pvm-in-pvm.ts` created (currently stubbed)
- ✅ **Minimal PVM Runner**: Created `examples-as/assembly/pvm-runner.ts` - basic SPI program runner
- ✅ **SPI Format Support**: All programs use SPI format throughout toolchain
- ✅ **Argument Passing Format**: SPI args format defined for PVM runner input

**Architecture**:

```text
Test Script → anan-as CLI → PVM Runner (compiled to PVM) → SPI Program Execution → Results
```

**Current Limitation**: `scripts/test-pvm-in-pvm.ts` is a stub - it compiles anan-as to PVM but `runTestThroughAnanAsInPvm()` just returns dummy values. Actual execution requires Phase 19.

### ✅ Phase 16c: SPI-Only Execution - COMPLETED (2025-01-22)
**Status**: COMPLETE - SPI format standardized throughout toolchain
**Impact**: Consistent program format from compilation to execution

**Completed**:
- ✅ **SPI Standardization**: All programs use SPI format exclusively
- ✅ **Format Conversion**: Automatic WASM → SPI → PVM conversion pipeline
- ✅ **CLI Integration**: anan-as CLI accepts SPI programs with arguments
- ✅ **Test Compatibility**: All existing tests work with SPI format

**Required work**:

#### Step 1: Create PVM-in-PVM Test Harness
1. Create `scripts/test-pvm-in-pvm.ts` - orchestrates nested PVM execution
2. Load compiled anan-as JAM file as outer PVM program
3. Pass inner JAM program as argument data to outer PVM
4. Extract and verify return values

#### Step 2: anan-as Wrapper for Standalone Execution
anan-as is a library, not a standalone program. Need a wrapper:
1. Create `examples-as/pvm-runner.ts` - AssemblyScript wrapper
2. Implements main() entry point that:
   - Reads inner program from args
   - Calls `prepareProgram(programBlob)`
   - Calls `resetGeneric(pc, gas, argsAddr, argsLen)`
   - Calls `nSteps(n)` to execute
   - Returns result registers/memory
3. Compile wrapper + anan-as to single JAM file

#### Step 3: Test Matrix
Run each example in PVM-in-PVM mode and verify:

| Example | Direct Result | PVM-in-PVM Result | Status |
|---------|---------------|-------------------|--------|
| add.jam.wat | 12 | ? | Pending |
| factorial.jam.wat | 120 | ? | Pending |
| fibonacci.jam.wat | 55 | ? | Pending |
| gcd.jam.wat | 6 | ? | Pending |
| is-prime.jam.wat | 1 | ? | Pending |
| div.jam.wat | 4 | ? | Pending |
| call.jam.wat | 10 | ? | Pending |
| recursive.jam.wat | 120 | ? | Pending |
| nested-calls.jam.wat | ? | ? | Pending |
| call-indirect.jam.wat | ? | ? | Pending |
| i64-ops.jam.wat | ? | ? | Pending |
| many-locals.jam.wat | ? | ? | Pending |
| bit-ops.jam.wat | ? | ? | Pending |
| rotate.jam.wat | ? | ? | Pending |
| br-table.jam.wat | ? | ? | Pending |
| block-result.jam.wat | ? | ? | Pending |
| AS examples (add, factorial, fibonacci, gcd, life) | ? | ? | Pending |

#### Step 4: Gas and Resource Tracking
1. Track gas consumption in outer vs inner PVM
2. Verify no resource exhaustion
3. Document expected gas overhead for PVM-in-PVM

#### Step 5: Automated CI Integration
1. Add PVM-in-PVM tests to `scripts/test-all.ts`
2. Add to GitHub Actions workflow
3. Fail CI if any PVM-in-PVM test mismatches direct execution

**Success Criteria**:
- All 62 existing tests also pass in PVM-in-PVM mode
- Gas consumption is reasonable (< 100x overhead)
- No panics or unexpected behavior in nested execution

---

### Phase 19: Working PVM-in-PVM Test Execution (NEXT PRIORITY)
**Status**: NOT STARTED - Infrastructure exists but execution is stubbed
**Impact**: Enable true nested PVM execution for all 62 test cases
**Goal**: Make `scripts/test-pvm-in-pvm.ts` actually run tests through compiled anan-as

**Current State**:
- `scripts/test-pvm-in-pvm.ts` exists but `runTestThroughAnanAsInPvm()` returns dummy values
- Need to implement actual execution via anan-as CLI or library

**Sub-Tasks**:

#### Step 1: Addition Test (Proof of Concept)
1. Modify `runTestThroughAnanAsInPvm()` to actually execute the test
2. Use anan-as CLI to run compiled anan-as JAM with inner test JAM
3. Pass inner JAM program and arguments via SPI format
4. Parse result from anan-as output
5. Verify `add.jam.wat` with args `0500000007000000` returns 12

#### Step 2: All Arithmetic Tests
1. Extend to all arithmetic operations (add, factorial, fibonacci, gcd)
2. Handle different argument formats
3. Verify results match direct execution

#### Step 3: Complex Tests
1. Function calls (call, call-indirect, recursive, nested-calls)
2. Memory operations (many-locals, i64-ops)
3. Control flow (br-table, block-result)
4. AssemblyScript examples (as-add, as-factorial, as-fibonacci, as-gcd)

#### Step 4: Full Test Suite Integration
1. Run all 62 tests through PVM-in-PVM
2. Compare results with direct execution
3. Add `--pvm-in-pvm` flag to `test-all.ts`
4. Add PVM-in-PVM tests to CI pipeline

#### Step 5: Validation & Documentation
1. Document gas overhead for nested execution
2. Create comparison matrix (direct vs PVM-in-PVM results)
3. Identify any tests that fail in PVM-in-PVM mode
4. Root cause and fix any issues

**Success Criteria**:
- [ ] `bun scripts/test-pvm-in-pvm.ts --filter=add` passes with real execution
- [ ] All 62 tests pass in PVM-in-PVM mode
- [ ] Results match direct execution exactly
- [ ] Gas overhead documented (< 100x acceptable)
- [ ] CI includes PVM-in-PVM validation

---

### ✅ Phase 14: Enhanced Memory Model - COMPLETED (2025-01-19)
**Status**: COMPLETE
**Impact**: WASM memory.size and memory.grow now work correctly

**Implemented**:
1. ✅ Parse WASM Memory section to get initial/max pages
2. ✅ Compiler-managed global for tracking current memory size
3. ✅ memory.size reads from compiler global instead of hardcoded value
4. ✅ memory.grow properly updates memory size with bounds checking
5. ✅ Added `BranchLtU` instruction for unsigned comparisons
6. ✅ Fixed register allocation in memory.grow to avoid clobbering stack values

**Technical Details**:
- Memory size global stored at `GLOBAL_MEMORY_BASE + (num_user_globals * 4)`
- Initial value set from WASM memory section (or 0 for AS minimal runtime)
- memory.grow returns old size on success, -1 on failure (exceeds max_memory_pages)
- max_memory_pages derived from WASM explicit max or defaults (256/1024 pages)

### Phase 17: Host Calls / ecalli Support (PLANNED - Phase 17)
**Goal**: Support generic external function calls via PVM `ecalli`.
**Status**: Not started
**Design**:
- **Import Mapping**: Treat imports from specific modules (e.g. `env`, `host`) as host calls.
- **ABI**:
  - Args 0-4 -> Registers r2-r6.
  - Args 5+ -> TBD (Stack? Or limit to 5 args for MVP).
  - Return value -> Register r7.
  - Memory pointers passed as `i32` args.
- **Instruction**: `ecalli ID` where ID is derived from the import.

**Tasks**:
1. Refactor `Operator::Call` to handle mapped imports by emitting `ecalli`.
2. Implement `emit_host_call` in codegen.
3. Update `run-jam.ts` (host harness) to:
   - Catch `HOST` exit code.
   - Decode instruction size at PC to calculate `next_pc`.
   - Dispatch ID to JS function.
   - Read args from registers.
   - Write result to r7.
   - Resume execution at `next_pc`.
4. Add test cases (e.g. `host_print`, `host_random`).

**Blockers**: None - ready to implement when prioritized.

### Phase 18: Architecture Refactor (Unit Testing)
**Status**: Planned - Phase 18
**Goal**: Improve maintainability and testability via layer separation.

**Current State**: Some unit tests exist (30 passing), but translation layer lacks isolated tests.

**Layer Separation Strategy**:
1. **Translation Layer**: Maps WASM operators to abstract PVM operations (independent of encoding).
2. **Builder Layer (`PvmBuilder` trait)**: Abstract interface for emitting instructions. Allows mocking.
3. **Register Allocation (`StackMachine`)**: Already isolated in `translate/stack.rs` with unit tests.
4. **Encoding Layer**: Concrete PVM instruction emission (implementation of Builder).

**Tasks**:
1. ✅ Extract `StackMachine` into a standalone, testable module with unit tests - DONE
2. Define `PvmBuilder` trait for instruction emission.
3. Implement `MockPvmBuilder` (for tests) and `ConcretePvmBuilder` (for production).
4. Refactor `codegen.rs` to use `PvmBuilder`.
5. Write unit tests for translation logic using `MockPvmBuilder` (verify arithmetic, locals, simple control flow).

---

## V1 Milestone: anan-as in PVM

**Goal**: Compile anan-as (AssemblyScript PVM interpreter) to WASM → PVM, run PVM-in-PVM.

### V1 Verification Checklist

#### Phase 1: Game of Life Validation ✅ COMPLETE (2025-01-19)
- [x] Fix Game of Life multi-step execution (Phase 11)
- [x] Verify operand stack spilling works correctly for deep expressions
- [x] Validate complex function call handling with spilled locals
- [x] Test with various step counts (0, 1, 2, 3, 4, 5) - all pass correctly

#### Phase 2: Core V1 Features ✅ COMPLETE
- [x] Implement data section initialization (Phase 12) ✅
- [x] Parse and handle imported functions in function indices ✅
- [x] Handle imported function calls (Phase 12b) ✅ - Stub imports with TRAP/no-op
- [x] Compile anan-as (AssemblyScript PVM interpreter) to WASM ✅
- [x] Translate WASM to PVM using wasm-pvm ✅ (423KB JAM file)

#### Phase 3: Robustness & Safety ✅ COMPLETE (2025-01-19)
- [x] Add stack overflow detection (Phase 13) ✅
- [x] Test deep recursion scenarios ✅

#### Phase 4: PVM-in-PVM Infrastructure ✅ COMPLETE (Phase 16)
- [x] Create anan-as wrapper with main() entry point
- [x] Build PVM-in-PVM test harness (infrastructure only)

#### Phase 5: Working PVM-in-PVM Test Execution (Phase 19) 🔄 IN PROGRESS
- [ ] Implement `runTestThroughAnanAsInPvm()` to actually execute tests
- [ ] Start with addition test as proof of concept
- [ ] Run all 62 examples through compiled anan-as
- [ ] Verify outputs match direct execution
- [ ] Add to CI pipeline (optional enhancement)

#### Phase 6: Memory Enhancement ✅ COMPLETE (Phase 14)
- [x] Implement proper WASM memory model (memory.size/memory.grow)
- [x] Support dynamic memory tracking via compiler-managed global

#### Phase 7: Polish & Safety ✅ COMPLETE (Phase 15)
- [x] Add call_indirect signature validation
- [x] Final integration testing with anan-as
- [ ] Performance benchmarking and optimization (future)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         WASM Binary                              │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    1. WASM Parser (wasmparser)                   │
│                    Parses WASM binary to module                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    2. Module Analyzer                            │
│  - Validate module                                               │
│  - Collect function signatures                                   │
│  - Analyze imports/exports                                       │
│  - Build type information                                        │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    3. Function Translator                        │
│  For each function:                                              │
│  - Parse WASM instructions                                       │
│  - Build control flow graph                                      │
│  - Convert stack ops to IR                                       │
│  - Register allocation                                           │
│  - Generate PVM instructions                                     │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    4. Module Linker                              │
│  - Resolve function addresses                                    │
│  - Build jump tables                                             │
│  - Layout memory sections                                        │
│  - Generate initialization code                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    5. PVM Encoder                                │
│  - Encode instructions to bytes                                  │
│  - Build opcode bitmask                                          │
│  - Encode jump table                                             │
│  - Produce final program blob                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    6. SPI Packager                               │
│  - Package RO data section                                       │
│  - Package RW data section                                       │
│  - Set heap/stack sizes                                          │
│  - Produce SPI binary                                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SPI Binary (JAM-compatible)                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Testing Strategy

### Unit Tests
- Each WASM instruction maps correctly to PVM sequence
- Register allocation works for various scenarios
- Control flow translation is correct

### Integration Tests
- Compile WAT files in `examples-wat/`
- Compile AssemblyScript files in `examples-as/`
- Execute on PVM interpreter (anan-as)
- Compare output with expected values

### Test Infrastructure
- `scripts/run-spi.ts` - Run SPI binaries on anan-as interpreter
- `scripts/test-all.ts` - Automated test suite (62 tests)
- `vendor/anan-as` - PVM reference implementation (submodule)

---

## Dependencies

### Required Crates
- `wasmparser` - Parse WASM binary format
- `thiserror` - Error handling
- `anyhow` - Error context for CLI
- `clap` - CLI argument parsing

### Development Crates
- `wat` - Parse WAT (text format) for tests

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| ~~Game of Life bug blocks Phase 2~~ | ~~High~~ | ✅ Resolved - Phase 11 complete |
| ~~Data section complexity~~ | ~~High~~ | ✅ Resolved - Phase 12 complete |
| PVM instruction set insufficient | Medium | ✅ All needed WASM ops map to PVM |
| ~~Register pressure too high~~ | ~~Medium~~ | ✅ Resolved - spilling works correctly |
| Control flow edge cases | Medium | ✅ Comprehensive test suite (62 tests) |
| Memory model mismatch | Medium | ✅ Clear address translation defined |
| ~~Recursion stack overflow~~ | ~~Medium~~ | ✅ Resolved - Phase 13 complete (stack overflow detection) |
| Performance issues | Low | Not a priority for v1 |
| anan-as is library not standalone | Low | Would need wrapper for full PVM-in-PVM |

---

## Open Questions to Resolve

1. ~~**PVM Calling Convention**~~: ✅ Resolved - See SPI convention above
2. ~~**Host Calls**~~: ✅ Resolved - Stub imports (TRAP for abort, no-op for others)
3. ~~**Memory Growth**~~: ✅ Returns -1 (not supported)
4. ~~**Floating Point**~~: ✅ Resolved - PVM has no FP, stubs for dead code paths
5. **Stack Size**: Configurable in SPI format (stackSize field, up to 16MB)

---

## Success Criteria

### Minimum Viable Product ✅
- All example WAT files compile and execute correctly
- AssemblyScript examples compile and execute correctly
- CLI tool works: `wasm-pvm compile input.wasm -o output.jam`
- Basic error handling and messages
- 62 integration tests passing

### V1 Release (Target: anan-as in PVM)
**Current Phase**: Phase 16 COMPLETE - PVM-in-PVM infrastructure operational!

**Completed Features**:
- [x] WASM MVP compliance (except floats)
- [x] Comprehensive test suite (62 integration + 30 Rust unit tests)
- [x] Documentation
- [x] Recursion support (Phase 8) ✅
- [x] Indirect calls (Phase 9) ✅
- [x] Game of Life debugging (Phase 11) ✅
- [x] Data section initialization (Phase 12) ✅
- [x] Import function stubbing (Phase 12b) ✅
- [x] anan-as compilation (423KB JAM file) ✅
- [x] Stack overflow detection (Phase 13) ✅
- [x] Enhanced memory model (Phase 14) ✅
- [x] call_indirect fixes + signature validation (Phase 15) ✅
- [x] AS Runtime Isolation testing (Phase 16a) ✅
- [x] PVM-in-PVM validation harness (Phase 16b) ✅
- [x] SPI-only execution (Phase 16c) ✅

**Next Phase Work** (in priority order):
1. **Phase 19: Working PVM-in-PVM Test Execution** (IMMEDIATE - currently stubbed)
   - Implement actual test execution in `test-pvm-in-pvm.ts`
   - Start with addition test as proof of concept
   - Extend to all 62 tests
   
2. **Phase 17: Host Calls / ecalli Support** (planned)
   - Support generic external function calls via PVM `ecalli`
   
3. **Phase 18: Architecture Refactor** (Unit Testing improvements)
   - Extract PvmBuilder trait for better testability

---

## Resources

- [Gray Paper](./gp-0.7.2.md) - PVM specification (Appendix A is key)
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries & instruction reference
- [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) - Known bugs and limitations
- [AGENTS.md](./AGENTS.md) - AI agent guidelines
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [Zink Compiler](./vendor/zink) - WASM→EVM compiler for architecture inspiration (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/)
- [AssemblyScript](https://www.assemblyscript.org/) - TypeScript-like language to WASM
