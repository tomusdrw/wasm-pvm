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
- [x] memory.size - returns constant 256 pages
- [x] memory.grow - returns -1 (not supported)

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
- [x] local.tee
- [x] drop
- [x] select
- [x] unreachable (maps to TRAP)

#### Memory Operations (Phase 5)
- [x] i64.load
- [x] i64.store
- [x] i32/i64 load8_u, load8_s, load16_u, load16_s, load32_u, load32_s
- [x] i32/i64 store8, store16, store32

#### Phase 4: AssemblyScript Examples
- [x] Set up AssemblyScript project in `examples-as/`
- [x] Create `add.ts`, `factorial.ts`, `fibonacci.ts`, `gcd.ts`
- [x] Verify AS output compiles through wasm-pvm
- [x] Document AssemblyScript → JAM workflow

#### Phase 4b: Test Suite & CI
- [x] Created `scripts/test-all.ts` - 56 tests across WAT and AS examples
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

**Test Suite**: 58 integration tests passing (as of 2025-01-18)
- [x] `call-indirect.jam.wat` - indirect function calls via table

AssemblyScript examples (`examples-as/assembly/*.ts`):
- [x] `life.ts` - Game of Life (compiles, runs with 0 steps; multi-step needs debugging)

---

## Remaining Work for V1 MVP

### Phase 11: Game of Life Debugging (PHASE 1 - IMMEDIATE PRIORITY)
**Status**: In Progress (easier targeted bug fix)
**Impact**: Validates operand stack spilling and complex function calls work correctly
**Timeline**: 1-2 weeks (debugging existing functionality)

**Symptom**: `life.jam` runs correctly with 0 steps but faults with exit code `0x60000` (memory access at invalid address) when running with 1+ steps. The `step_once` function has deep stack usage (8 neighbors loaded and summed).

**Debugging approach**:
1. **Verify AssemblyScript correctness first**:
   - Run `life.wasm` directly in a standard WASM runtime (e.g., `wasmtime`, Node.js)
   - Test with 1, 5, 10 steps to confirm the algorithm works
   - If WASM fails, fix the AssemblyScript code

2. **If WASM works, debug the compiler**:
   - Add verbose logging to track stack spill/restore operations
   - Compare PVM execution trace with expected WASM semantics
   - Check address calculations in `step_once` neighbor lookup
   - Verify spill offset calculations are correct for deep stack (depth 6-8)

3. **Specific areas to investigate**:
   - The fault address `0x60000` = 2 × `0x30000` suggests possible address doubling
   - Check if spilled values are being loaded/stored correctly during function calls
   - Verify the spill area offset (`-0x100` from sp) doesn't conflict with call frames

**Related tasks to complete along the way**:
- May require fixing operand stack spilling bugs (currently marked as implemented but may have edge cases)
- May need to address memory layout conflicts between spill area and call frames
- Validate that complex function calls with deep stack usage work correctly

### Phase 12: Data Section Initialization (PHASE 2 - V1 BLOCKER)
**Status**: Not implemented (major new feature)
**Impact**: Blocks anan-as compilation - anan-as has multiple `(data ...)` sections
**Timeline**: 2-3 weeks (implementing new subsystem)
**Prerequisites**: Phase 11 completed (robust operand stack spilling)

**Required work**:
1. Parse WASM `DataSection` in `translate/mod.rs`
2. Initialize data in SPI `rw_data` section at correct offsets
3. Support both active and passive data segments
4. Update memory layout to account for initialized data
5. Handle offset expressions in active data segments
6. Ensure data doesn't conflict with existing memory regions (globals, heap, etc.)

### Phase 13: Stack Overflow Detection (PHASE 3)
**Status**: Not implemented
**Impact**: Deep recursion in anan-as interpreter may corrupt memory
**Timeline**: 1 week
**Prerequisites**: Phases 11-12 completed

**Required work**:
1. Add stack depth checking in call emission
2. Implement configurable stack size limits
3. Emit TRAP on stack overflow

### Phase 14: Enhanced Memory Model (PHASE 4)
**Status**: Partial (hardcoded memory.size=256, memory.grow=-1)
**Impact**: anan-as may expect dynamic memory
**Timeline**: 1-2 weeks
**Prerequisites**: Phase 12 completed

**Required work**:
1. Track actual memory size from WASM module
2. Support memory.grow up to PVM limits
3. Base address translation for WASM memory operations

### Phase 15: Runtime Safety (PHASE 5 - LOW PRIORITY)
**Status**: Partially implemented
**Timeline**: 1 week
**Prerequisites**: All previous phases completed

#### 🟢 LOW: call_indirect Signature Validation
**Status**: Not implemented (trusts caller)
**Impact**: Type safety violation possible

**Required work**:
1. Validate function signatures at runtime
2. Add type checking before dispatch table lookup
3. Emit TRAP on signature mismatch

#### 🟢 LOW: Operand Stack Validation
**Status**: ✅ Implemented (2025-01-18) - spills to memory when depth > 5
**Impact**: Complex expressions now compile and mostly work

---

## V1 Milestone: anan-as in PVM

**Goal**: Compile anan-as (AssemblyScript PVM interpreter) to WASM → PVM, run PVM-in-PVM.

### V1 Verification Checklist

#### Phase 1: Game of Life Validation (COMPLETE BEFORE PHASE 2)
- [ ] Fix Game of Life multi-step execution (Phase 11)
- [ ] Verify operand stack spilling works correctly for deep expressions
- [ ] Validate complex function call handling with spilled locals
- [ ] Test with various step counts (1, 5, 10, 20) to ensure stability

#### Phase 2: Core V1 Features (COMPLETE BEFORE PHASE 3)
- [ ] Implement data section initialization (Phase 12)
- [ ] Compile anan-as (AssemblyScript PVM interpreter) to WASM
- [ ] Translate WASM to PVM using wasm-pvm
- [ ] Run the compiled PVM interpreter inside a PVM interpreter
- [ ] Verify correctness with test vectors

#### Phase 3: Robustness & Safety (COMPLETE BEFORE PHASE 4)
- [ ] Add stack overflow detection (Phase 13)
- [ ] Test deep recursion scenarios

#### Phase 4: Memory Enhancement (COMPLETE BEFORE PHASE 5)
- [ ] Implement proper WASM memory model (Phase 14)
- [ ] Support dynamic memory growth where needed

#### Phase 5: Polish & Safety (FINAL PHASE)
- [ ] Add call_indirect signature validation
- [ ] Final integration testing with anan-as
- [ ] Performance benchmarking and optimization

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
- `scripts/test-all.ts` - Automated test suite (58 tests)
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
| **Game of Life bug blocks Phase 2** | High | **Phase 11 in progress** - fix operand stack spilling |
| Data section complexity | High | **Phase 12 planned** - systematic implementation |
| PVM instruction set insufficient | Medium | Check all WASM ops map to PVM early |
| Register pressure too high | Medium | **Phase 11 validating** spilling implementation |
| Control flow edge cases | Medium | Comprehensive test suite |
| Memory model mismatch | Medium | Define clear address translation |
| Recursion stack overflow | Medium | Phase 13: add stack overflow detection |
| Performance issues | Low | Not a priority for v1 |

---

## Open Questions to Resolve

1. ~~**PVM Calling Convention**~~: ✅ Resolved - See SPI convention above
2. **Host Calls**: How to handle WASM imports? Map to PVM ecalli?
3. ~~**Memory Growth**~~: ✅ Returns -1 (not supported)
4. ~~**Floating Point**~~: ✅ Resolved - PVM has no FP, reject WASM with floats
5. **Stack Size**: Configurable in SPI format (stackSize field, up to 16MB)

---

## Success Criteria

### Minimum Viable Product ✅
- All example WAT files compile and execute correctly
- AssemblyScript examples compile and execute correctly
- CLI tool works: `wasm-pvm compile input.wasm -o output.jam`
- Basic error handling and messages
- 58 integration tests passing

### V1 Release (Target: anan-as in PVM)
**Current Phase**: 11 (Game of Life debugging)

**Completed Features**:
- [x] WASM MVP compliance (except floats)
- [x] Comprehensive test suite (58 tests)
- [x] Documentation
- [x] Recursion support (Phase 8) ✅
- [x] Indirect calls (Phase 9) ✅

**Phase 11**: Game of Life debugging (In Progress)
- [ ] Fix operand stack spilling for complex expressions
- [ ] Validate function call handling with spilled locals

**Phase 12**: Data section initialization (Next)
- [ ] Parse WASM data sections
- [ ] Initialize SPI RW data segments
- [ ] Enable anan-as compilation

**Phase 13-15**: Remaining V1 features (After Phase 12)
- [ ] Stack overflow detection
- [ ] Enhanced memory model
- [ ] Runtime safety improvements
- [ ] Successfully compile and run anan-as PVM interpreter

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
