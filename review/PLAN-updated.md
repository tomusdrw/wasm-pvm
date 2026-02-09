# Updated Project Plan - V1 Milestone (Post-Review)

**Version**: 2.0 (Incorporates Architecture Review Findings)  
**Date**: 2026-02-09  
**Status**: REVISED with critical fixes required before V1

---

## Executive Summary

Based on comprehensive architecture review, this updated plan incorporates **critical correctness fixes** required before V1 completion. The original plan is preserved with "[LEGACY]" markers, while new sections include review findings and revised priorities.

**CRITICAL DECISION**: V1 completion is blocked by 3 open bugs that could cause silent data corruption or crashes. These must be fixed before declaring V1 complete.

---

## Legacy Content (Original Plan)

<details>
<summary>Click to view original project overview</summary>

**[LEGACY] Original Project Overview**:
Build a **WASM (WebAssembly) to PVM (Polka Virtual Machine)** recompiler in Rust. The recompiler takes WASM bytecode and produces equivalent PVM bytecode that can execute on the PolkaVM.

**[LEGACY] Original Goals**:
1. Correctness - Produce semantically equivalent PVM code
2. Completeness - Support core WASM MVP features
3. Testability - Comprehensive test suite with reference interpreter
4. Maintainability - Clean architecture following Rust best practices

**[LEGACY] Original V1 Milestone**: PVM-in-PVM - Compile anan-as to WASM, then to PVM, and run a PVM interpreter inside a PVM interpreter.

**[LEGACY] Original Non-Goals**:
- Performance optimization (focus on correctness first)
- WASM proposals beyond MVP (SIMD, threads, etc.)
- Floating point support (PVM has no FP - reject WASM with floats)

</details>

---

## REVISED V1 Definition (Post-Review)

### What "V1 Complete" Actually Means

**OLD DEFINITION (Insufficient)**: Infrastructure exists, tests pass on direct execution  
**NEW DEFINITION (Correct)**: All critical bugs fixed, PVM-in-PVM works correctly, architecture ready for V2

### V1 Completion Checklist (REVISED)

#### Phase A: Critical Bug Fixes (REQUIRED for V1)

These 3 bugs must be fixed. They cause silent data corruption or crashes:

- [ ] **BUG-1**: `memory.copy` overlapping regions (memmove)
  - Status: 🔴 Open, High Severity
  - Impact: Data corruption when `dest > src` with overlap
  - Effort: 2-3 days
  - Files: `codegen.rs:2481-2599`

- [ ] **BUG-2**: Division overflow checks (INT_MIN / -1, div by zero)
  - Status: 🔴 Open, Medium Severity
  - Impact: Undefined behavior instead of WASM-specified trap
  - Effort: 1-2 days
  - Files: `codegen.rs:1462-1481` (I32DivS), `codegen.rs:1716-1725` (I64DivS)

- [ ] **BUG-3**: Import return values ignored
  - Status: 🔴 Open, Medium Severity
  - Impact: Stack imbalance when calling imports that return values
  - Effort: 1 day
  - Files: `codegen.rs:2210-2241`

#### Phase B: PVM-in-PVM Execution (REQUIRED for V1)

- [ ] Execute `add.jam.wat` through compiled anan-as
- [ ] Execute all 62 tests through PVM-in-PVM
- [ ] Results match direct execution exactly
- [ ] No panics or memory faults

#### Phase C: Architecture Hardening (REQUIRED for V1)

- [ ] Add WASM validation phase before translation
- [ ] Document all memory layout assumptions
- [ ] Add assertions for stack depth invariants
- [ ] Fix all clippy warnings (remove #![allow(...)] suppressions)

#### Phase D: Completion Criteria

- [ ] All 62 integration tests pass in both direct and PVM-in-PVM modes
- [ ] 0 critical or high severity open bugs
- [ ] Test coverage > 60% (up from ~30%)
- [ ] Architecture review findings documented and acknowledged

---

## Current Status Assessment

### What's Actually Working (Verified)

**✅ Direct Execution**: 62/62 tests pass on direct execution  
**✅ anan-as Compilation**: 423KB JAM file compiles successfully  
**✅ Infrastructure**: SPI format, CLI, test harness all functional  
**❌ PVM-in-PVM**: Inner interpreter PANICs (address < 0x10000)  
**❌ Critical Bugs**: 3 open bugs with data corruption potential

### The PVM-in-PVM Problem

**Current State**: Inner program args are correct (0xFEFF0000 contains expected values), but inner interpreter PANICs at PC 56 with exitCode 0.

**Root Cause Theory**: The `memory.copy` bug causes corruption during `Array.unshift()` operations in AssemblyScript runtime. This corrupts the Arena page allocator, causing all Uint8Array views to alias the same memory. When the inner interpreter tries to load args, it gets garbage.

**Why This Matters for V1**: The PVM-in-PVM milestone isn't just about running tests twice - it's about proving the compiled anan-as is actually correct. If it can't interpret simple programs, the compilation is wrong.

---

## Revised Phase Plan

### Phase 19 (REVISED): Critical Bug Fixes → THEN PVM-in-PVM

**Duration**: 2-3 weeks  
**Priority**: CRITICAL - Blocks V1

**Week 1: Fix memory.copy**
1. Verify overlapping copy issue with test case
2. Add backward copy path when `dest > src`
3. Test with memmove-like patterns
4. Verify Game of Life still works

**Week 2: Fix division overflow**
1. Add div by zero check before DivS instructions
2. Add INT_MIN / -1 check
3. Add test cases for edge cases
4. Verify performance impact is minimal

**Week 3: Fix import returns**
1. Push dummy value (0) for imports with return types
2. Add test with mock import
3. Verify no stack imbalance

**Then**: Re-test PVM-in-PVM with fixed memory.copy

### Phase 20: PVM-in-PVM Test Execution (REPRIORITIZED)

**Duration**: 1-2 weeks  
**Priority**: HIGH - Required for V1

**Goal**: Make PVM-in-PVM actually work

**Steps**:
1. Verify memmove fix resolves PANIC issue
2. If not, trace inner interpreter execution step-by-step
3. Compare register states between direct and PVM-in-PVM
4. Fix any remaining issues
5. Run full 62-test suite through PVM-in-PVM

### Phase 21: Architecture Cleanup (NEW)

**Duration**: 2 weeks  
**Priority**: MEDIUM - Enables V2

**Goal**: Address code quality issues from review

**Tasks**:
1. Remove dead code (`check_for_floats`, commented-out code)
2. Fix clippy warnings (remove #![allow(...)])
3. Extract constants into `MemoryLayout` abstraction
4. Add assertions for stack depth invariants
5. Document register conventions comprehensively

### Phase 22: Testing Improvements (NEW)

**Duration**: 2 weeks  
**Priority**: MEDIUM - Prevents regressions

**Goal**: Comprehensive test coverage

**Tasks**:
1. Add unit tests for `StackMachine`
2. Add tests for spilling logic
3. Add regression tests for all fixed bugs
4. Add property-based tests (proptest)
5. Add fuzzing targets
6. Add differential tests against wasmtime

---

## Technical Debt Acknowledgment

### What Won't Be Fixed in V1 (By Design)

These issues are acknowledged but deferred to V2:

1. **No Intermediate Representation**: Direct translation is technical debt, but works for now
2. **Monolithic codegen.rs**: 2,400-line file stays for V1, refactor in V2
3. **Ad-hoc register allocation**: Hardcoded register usage stays, proper allocator in V2
4. **No optimizations**: Constant folding, DCE deferred to V2
5. **Passive data segments**: Not needed for current use cases

**Why This Is OK**: The compiler works for the current use case (anan-as compilation). The architecture debt is manageable for V1 scope. The critical bugs must be fixed, but architectural improvements can wait.

---

## Resource Requirements

### Effort Estimates (REVISED)

| Task | Original Estimate | Revised Estimate | Reason |
|------|-------------------|------------------|--------|
| memory.copy fix | N/A | 2-3 days | Critical bug found in review |
| Division overflow | N/A | 1-2 days | Critical bug found in review |
| Import returns | N/A | 1 day | Bug found in review |
| PVM-in-PVM | 1 week | 2-3 weeks | Found to be broken, needs debugging |
| Architecture cleanup | N/A | 2 weeks | Review recommendation |
| Testing improvements | N/A | 2 weeks | Review recommendation |
| **Total to V1** | **~1 week** | **~10 weeks** | **Critical bugs + PVM-in-PVM issues** |

### Risk Assessment (UPDATED)

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| PVM-in-PVM has more bugs after memmove fix | High | Critical | Incremental debugging with traces |
| Division overflow fix impacts performance | Low | Low | Only adds checks on div ops |
| Architecture cleanup breaks existing tests | Medium | Medium | Extensive testing before merge |
| Timeline slips | High | Medium | Can ship V1 without Phase 21-22 (cleanup) |

---

## Success Criteria (REVISED)

### V1 MVP (Minimum)

- [ ] 3 critical bugs fixed
- [ ] PVM-in-PVM passes at least add/factorial/fibonacci/gcd tests
- [ ] 62 direct execution tests still pass
- [ ] Documentation reflects current state

### V1 Full (Target)

- [ ] All 62 tests pass in PVM-in-PVM mode
- [ ] Code quality issues from review addressed
- [ ] Test coverage > 60%
- [ ] Architecture review findings documented

---

## Next Immediate Actions

### Today
1. Read status.md to understand current debugging state
2. Verify memory.copy bug with minimal test case
3. Start implementing backward copy path

### This Week
1. Complete memory.copy fix
2. Test fix with Game of Life (uses memory operations)
3. Run PVM-in-PVM add test to see if memmove fix resolves PANIC

### Next Week
1. If PVM-in-PVM works: celebrate and move to division overflow fix
2. If PVM-in-PVM still broken: deep trace and debugging

---

## Conclusion

The original plan assumed PVM-in-PVM was "infrastructure complete" when it's actually broken. The architecture review revealed critical bugs that must be fixed before V1 can be declared complete.

**The hard truth**: V1 is ~2-3 months away, not weeks, due to:
1. Critical bugs that cause data corruption
2. PVM-in-PVM that doesn't actually work
3. Technical debt that makes debugging harder

**The path forward**: Fix the critical bugs first, then PVM-in-PVM. Architecture improvements are important but secondary to correctness.

---

*This document supersedes the original PLAN.md for V1 planning purposes.*
