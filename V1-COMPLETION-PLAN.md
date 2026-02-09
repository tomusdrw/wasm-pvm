# V1 Completion Plan

**Objective**: Get WASM->PVM compiler to V1 completion state
**Goal**: Working PVM-in-PVM with all critical bugs fixed
**Created**: 2026-02-09 (from architecture review)
**Updated**: 2026-02-10

---

## Current State

- 62 direct execution tests pass
- anan-as compiles to PVM (423KB JAM)
- Critical bugs BUG-2 (division overflow) and BUG-3 (import returns) are FIXED
- Code quality items (WASM validation, clippy, memory layout extraction) are DONE
- PVM-in-PVM inner interpreter still PANICs (BUG-4 - main remaining blocker)

### V1 Definition

**MUST HAVE** (Blocks V1):
1. ~~3 critical bugs fixed~~ -> BUG-2, BUG-3 fixed. BUG-1 (memory.copy) verified fixed.
2. PVM-in-PVM passes basic tests (add, factorial, fibonacci, gcd)
3. 62 direct tests still pass

**SHOULD HAVE** (V1 Quality):
4. Test coverage > 60%
5. Code quality issues addressed
6. Documentation complete

**WON'T FIX** (V2):
- No IR (direct translation stays)
- No proper register allocator
- No optimizations

---

## Remaining Work

### PHASE 1: PVM-in-PVM Debugging (PRIMARY BLOCKER)

The main remaining work is debugging why the PVM-in-PVM inner interpreter PANICs.

**Diagnostic Setup**:
- [ ] Create PVM-in-PVM specific tracer with register dumps
- [ ] Run add.jam.wat directly, capture initial state and first 50 instructions
- [ ] Run same through PVM-in-PVM, capture same data
- [ ] Diff the two traces to find first divergence point

**Root Cause Analysis**:
- [ ] Identify whether divergence is register value, memory content, or instruction flow
- [ ] If register values differ: trace back to where they diverged
- [ ] If memory differs: identify which store corrupted it
- [ ] If instruction flow differs: check jump table/branch logic

**Validation**:
- [ ] Run add.jam.wat -> should return 12
- [ ] Run factorial.jam.wat -> should return 120 (5!)
- [ ] Run fibonacci.jam.wat -> should return 55 (fib(10))
- [ ] Run gcd.jam.wat -> should return 6 (gcd(48,18))
- [ ] Test all 62 programs through PVM-in-PVM

### PHASE 2: Testing Improvements

- [ ] Add unit tests for `StackMachine` (push/pop at various depths, spill detection)
- [ ] Add tests for spilling logic (depths 0-10, function call interactions)
- [ ] Add edge case tests (maximum locals, deep call stack, empty functions)
- [ ] Run `cargo tarpaulin` and add tests for uncovered paths

### PHASE 3: Documentation & Completion

- [ ] Update documentation to reflect final state
- [ ] Create V2 planning section
- [ ] Tag v1.0

---

## Key Files

- `crates/wasm-pvm/src/translate/codegen.rs` - Main compiler (2,400 lines)
- `crates/wasm-pvm/src/translate/mod.rs` - Compilation orchestration
- `crates/wasm-pvm/src/translate/memory_layout.rs` - Memory address constants
- `scripts/test-all.ts` - Test runner
- `scripts/test-pvm-in-pvm.ts` - PVM-in-PVM harness

## Key Commands

```bash
# Run all tests
cargo test
bun scripts/test-all.ts

# Run specific test
bun scripts/test-all.ts --filter=add

# Run PVM-in-PVM
bun scripts/test-pvm-in-pvm.ts

# Trace execution
bun scripts/trace-steps.ts <jam-file> <args> <steps>
```

---

*This is a living document. Update it as progress is made.*
