# WASM to PVM Recompiler - Project Plan

**Version**: 2.0 (Post Architecture Review)
**Date**: 2026-02-10
**Status**: Most critical bugs fixed, PVM-in-PVM debugging remaining

---

## Project Overview

Build a **WASM (WebAssembly) to PVM (Polka Virtual Machine)** recompiler in Rust. The recompiler takes WASM bytecode and produces equivalent PVM bytecode that can execute on the PolkaVM.

### Goals
1. **Correctness** - Produce semantically equivalent PVM code
2. **Completeness** - Support core WASM MVP features
3. **Testability** - Comprehensive test suite with reference interpreter
4. **Maintainability** - Clean architecture following Rust best practices

### V1 Milestone: PVM-in-PVM
Compile [anan-as](https://github.com/polkavm/anan-as) (PVM interpreter in AssemblyScript) to WASM -> PVM, and run a PVM interpreter inside a PVM interpreter.

### Non-Goals (V1)
- Performance optimization (focus on correctness first)
- WASM proposals beyond MVP (SIMD, threads, etc.)
- Floating point support (PVM has no FP - reject WASM with floats)
- Intermediate Representation (deferred to V2)
- Proper register allocator (deferred to V2)

---

## V1 Completion Checklist

### Phase A: Critical Bug Fixes - DONE

- [x] **BUG-1**: `memory.copy` overlapping regions - VERIFIED FIXED (backward copy path)
- [x] **BUG-2**: Division overflow checks - FIXED (all 8 div/rem ops, 8 regression tests)
- [x] **BUG-3**: Import return values - FIXED (dummy value push, 4 regression tests)

### Phase B: PVM-in-PVM Execution - IN PROGRESS

- [ ] Execute `add.jam.wat` through compiled anan-as
- [ ] Execute all 62 tests through PVM-in-PVM
- [ ] Results match direct execution exactly
- [ ] No panics or memory faults

**Blocker**: BUG-4 - Inner interpreter PANICs at PC 56 with exitCode 0 (memory fault < 0x10000)

### Phase C: Architecture Hardening - DONE

- [x] Add WASM validation phase before translation (`wasmparser::validate()`)
- [x] Extract memory layout constants into dedicated module (`memory_layout.rs`)
- [x] Add debug assertions for stack depth invariants
- [x] Fix clippy warnings (dead code removed, zero warnings)

### Phase D: Completion Criteria

- [x] 62 integration tests pass in direct mode
- [ ] PVM-in-PVM core tests pass (add, factorial, fibonacci, gcd)
- [ ] 0 critical or high severity open bugs
- [ ] Architecture review findings documented and acknowledged

---

## Current Status

### What's Working
- **Direct Execution**: 62/62 tests pass
- **anan-as Compilation**: 423KB JAM file compiles successfully
- **Infrastructure**: SPI format, CLI, test harness all functional
- **Code Quality**: WASM validation, memory layout module, debug assertions, zero clippy warnings

### What's Not Working
- **PVM-in-PVM**: Inner interpreter PANICs (BUG-4) - main remaining blocker
  - Inner program args page at 0xFEFF0000 is correct
  - PANICs at PC 56 with exitCode 0 (memory fault < 0x10000)
  - Root cause theory: memory corruption in AS runtime (see KNOWN_ISSUES.md)

---

## Completed Phases

<details>
<summary>Click to view completed phase history</summary>

### Phase 1: Foundation
- [x] Rust project with Cargo workspace
- [x] `Opcode` and `Instruction` enums
- [x] Instruction encoding and bitmask generation
- [x] CLI structure and test infrastructure

### Phase 2: Simple Functions
- [x] WASM parsing, function types and bodies
- [x] Arithmetic translation (i32/i64)
- [x] Operand stack to register mapping (r2-r6)
- [x] SPI entrypoint convention

### Phase 3: Control Flow
- [x] block, loop, br, br_if, return, if/else/end
- [x] Block result values

### Phase 4: AssemblyScript & Test Suite
- [x] AS examples (add, factorial, fibonacci, gcd)
- [x] 62 integration tests, 50+ Rust tests
- [x] GitHub Actions CI

### Phase 5: Memory Operations
- [x] i32/i64 load/store with all sub-word variants
- [x] memory.fill, memory.copy (with memmove fix)

### Phase 6-9: Functions & Calls
- [x] call with return value handling
- [x] call_indirect via dispatch table
- [x] Recursion with proper call stack
- [x] Local variable spilling

### Phase 11: Game of Life
- [x] Fixed I64Load, spilled operand stack, local.tee bugs
- [x] Game of Life works for any number of steps

### Phase 12: Data Sections & Imports
- [x] Data section initialization at WASM_MEMORY_BASE
- [x] Import function stubbing (abort -> TRAP, others -> no-op)
- [x] anan-as compiles to 423KB JAM file

### Phase 13: Stack Overflow Detection
- [x] Stack limit checks in function prologues
- [x] Configurable stack size (default 64KB)

### Phase 14: Memory Model
- [x] memory.size/memory.grow with compiler-managed global

### Phase 15: call_indirect Fixes
- [x] Signature validation
- [x] Stack overflow check no longer clobbers operand stack

### Phase 16: PVM-in-PVM Infrastructure
- [x] PVM runner wrapper
- [x] Test harness (scripts/test-pvm-in-pvm.ts)
- [x] SPI format standardized throughout toolchain

### Phase 19a: Critical Bug Fixes (2026-02-09)
- [x] Division overflow checks for all 8 div/rem ops
- [x] Import return value handling
- [x] memory.copy verification

### Phase 19b: Code Quality (2026-02-09)
- [x] Dead code removal, clippy fixes
- [x] Debug assertions for stack invariants
- [x] Memory layout module extraction
- [x] WASM validation added
- [x] local.tee test coverage (8 tests)
- [x] Deep stack spill tests

</details>

---

## Remaining Work

### Phase 19c: PVM-in-PVM Debugging (NEXT)

**Priority**: CRITICAL - Main V1 blocker
**Goal**: Make PVM-in-PVM actually work

**Steps**:
1. Create PVM-in-PVM specific tracer with register dumps
2. Run add.jam.wat directly and through PVM-in-PVM, diff traces
3. Find first divergence point
4. Fix root cause
5. Run full 62-test suite through PVM-in-PVM

### Phase 20: Testing Improvements

**Priority**: MEDIUM
**Goal**: Comprehensive test coverage

1. Unit tests for `StackMachine` (spill logic, various depths)
2. Regression tests for all fixed bugs
3. Edge case tests (max locals, deep call stack)
4. Property-based tests (proptest) and fuzzing

### Phase 21: Host Calls / ecalli Support

**Priority**: LOW (planned)
**Goal**: Support generic external function calls via PVM `ecalli`

---

## Technical Debt Acknowledgment (V2 Scope)

These issues are acknowledged but deferred:

1. **No IR**: Direct translation works for V1 but prevents optimizations
2. **Monolithic codegen.rs**: 2,400-line file stays for V1
3. **Ad-hoc register allocation**: Hardcoded register usage stays
4. **No optimizations**: Constant folding, DCE deferred
5. **Passive data segments**: Not needed for current use cases

See [review/](./review/) for the full architecture review and V2 recommendations.

---

## Success Criteria

### V1 MVP (Minimum)
- [x] 3 critical bugs fixed
- [ ] PVM-in-PVM passes at least add/factorial/fibonacci/gcd tests
- [x] 62 direct execution tests pass
- [x] Documentation reflects current state

### V1 Full (Target)
- [ ] All 62 tests pass in PVM-in-PVM mode
- [x] Code quality issues from review addressed
- [ ] Test coverage > 60%
- [x] Architecture review findings documented

---

## Resources

- [Gray Paper](./gp-0.7.2.md) - PVM specification
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries & instruction reference
- [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) - Known bugs and limitations
- [V1-COMPLETION-PLAN.md](./V1-COMPLETION-PLAN.md) - Detailed V1 completion steps
- [AGENTS.md](./AGENTS.md) - AI agent guidelines
- [review/](./review/) - Architecture review findings and proposals
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/)
