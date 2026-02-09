# Comprehensive Step-by-Step V1 Completion Plan

**Objective**: Get WASM→PVM compiler to V1 completion state  
**Duration**: 10 weeks (revised from 1 week based on review findings)  
**Goal**: Working PVM-in-PVM with all critical bugs fixed

---

## Overview

### Current State
- ✅ 62 direct execution tests pass
- ✅ anan-as compiles to PVM (423KB JAM)
- ❌ 3 critical bugs cause data corruption/crashes
- ❌ PVM-in-PVM inner interpreter PANICs
- ⚠️ Architecture has technical debt but works

### V1 Definition (Revised)
**MUST HAVE** (Blocks V1):
1. 3 critical bugs fixed
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

## Phase-by-Phase Execution Plan

### PHASE 1: Emergency Bug Fixes (Weeks 1-2)
**Priority**: CRITICAL  
**Goal**: Fix the 3 bugs that cause data corruption

#### Week 1: Fix memory.copy (Days 1-5)

**Day 1: Investigation**
- [ ] Read current memory.copy implementation in codegen.rs:2481-2599
- [ ] Verify backward copy path exists (from status.md, it was added)
- [ ] Create minimal test case for overlapping copy
- [ ] Run test to see if bug still reproduces

**Day 2: Fix Implementation**
- [ ] If fix incomplete, complete backward copy implementation
- [ ] Ensure backward path triggers when `dest > src && regions_overlap`
- [ ] Add comments explaining memmove semantics

**Day 3: Testing**
- [ ] Add test case: copy 10 bytes with dest = src + 5
- [ ] Verify source data preserved after copy
- [ ] Test with large buffers (> 64KB)
- [ ] Test with exact overlap (dest == src) - should be no-op

**Day 4: Integration**
- [ ] Run Game of Life test (uses memory operations)
- [ ] Run all 62 integration tests
- [ ] Verify no regressions

**Day 5: PVM-in-PVM Test**
- [ ] Re-run PVM-in-PVM add test
- [ ] If still PANICs, document new symptoms
- [ ] Compare memory state with direct execution

**Deliverable**: memory.copy works correctly for all overlap cases

#### Week 2: Division & Import Fixes (Days 6-10)

**Day 6: Division Overflow**
- [ ] Locate I32DivS, I64DivS in codegen.rs
- [ ] Add div by zero check:
  ```rust
  // Before: emitter.emit(Instruction::DivS32 { ... });
  // After:
  //   check divisor != 0, trap if zero
  //   check !(dividend == INT_MIN && divisor == -1), trap if overflow
  //   emit normal division
  ```
- [ ] Add test: div by zero should trap
- [ ] Add test: INT_MIN / -1 should trap
- [ ] Run tests to verify

**Day 7: Import Return Values**
- [ ] Locate import handling in codegen.rs:2210-2241
- [ ] Add dummy value push:
  ```rust
  if has_return {
      let dst = emitter.spill_push();
      emitter.emit(Instruction::LoadImm { reg: dst, value: 0 });
  }
  ```
- [ ] Create test with mock import that returns value
- [ ] Verify stack balance

**Day 8-9: Comprehensive Testing**
- [ ] Run all 62 tests with fixes
- [ ] Add regression tests for fixed bugs
- [ ] Document fixes in KNOWN_ISSUES.md

**Day 10: Buffer**
- [ ] Fix any regressions
- [ ] Prepare for PVM-in-PVM debugging

**Deliverable**: 3 critical bugs fixed, all tests pass

---

### PHASE 2: PVM-in-PVM Debugging (Weeks 3-5)
**Priority**: CRITICAL  
**Goal**: Make inner interpreter work

#### Week 3: Diagnostic Setup (Days 11-17)

**Day 11: Trace Infrastructure**
- [ ] Review scripts/trace-steps.ts
- [ ] Create PVM-in-PVM specific tracer
- [ ] Add register dump at each step
- [ ] Focus on first 100 steps of inner execution

**Day 12: Baseline Comparison**
- [ ] Run add.jam.wat directly, capture:
  - Initial register values
  - Memory state at 0xFEFF0000
  - First 50 instructions executed
- [ ] Run same through PVM-in-PVM, capture same data
- [ ] Diff the two traces

**Day 13: Identify Divergence**
- [ ] Find first instruction where state differs
- [ ] Determine if it's register value, memory content, or instruction flow
- [ ] Hypothesize root cause

**Day 14-15: Root Cause Analysis**
- [ ] If register values differ: trace back to where they diverged
- [ ] If memory differs: identify which store corrupted it
- [ ] If instruction flow differs: check jump table/branch logic

**Day 16-17: Fix Implementation**
- [ ] Implement fix for identified issue
- [ ] Test fix
- [ ] Document findings

**Deliverable**: Root cause identified and fixed

#### Week 4: Validation (Days 18-24)

**Day 18-20: PVM-in-PVM Test Suite**
- [ ] Run add.jam.wat → should return 12
- [ ] Run factorial.jam.wat → should return 120 (5!)
- [ ] Run fibonacci.jam.wat → should return 55 (fib(10))
- [ ] Run gcd.jam.wat → should return 6 (gcd(48,18))

**Day 21-22: Fix Remaining Issues**
- [ ] If any test fails, debug and fix
- [ ] Use same tracing approach
- [ ] Document patterns in failures

**Day 23-24: Expanded Testing**
- [ ] Test all 62 programs through PVM-in-PVM
- [ ] Record which pass/fail
- [ ] Create spreadsheet of results

**Deliverable**: At least 4 core tests passing in PVM-in-PVM

#### Week 5: Full Suite (Days 25-31)

**Day 25-28: Remaining Test Fixes**
- [ ] For each failing test:
  - Identify if it's a known issue pattern
  - Apply fix if generic
  - Skip if requires major work (document)

**Day 29-30: Final Validation**
- [ ] Run full test suite: `bun scripts/test-all.ts`
- [ ] All 62 should pass directly
- [ ] Core tests should pass PVM-in-PVM

**Day 31: Documentation**
- [ ] Document PVM-in-PVM results
- [ ] Update status.md
- [ ] Create issue for remaining failing tests (if any)

**Deliverable**: PVM-in-PVM working for core tests

---

### PHASE 3: Quality & Hardening (Weeks 6-7)
**Priority**: HIGH  
**Goal**: Address code quality, add tests

#### Week 6: Code Quality (Days 32-38)

**Day 32: Remove Dead Code**
- [ ] Delete `check_for_floats` function (unused, marked dead_code)
- [ ] Delete `is_float_op` function (unused)
- [ ] Remove commented-out code
- [ ] Run tests to ensure nothing breaks

**Day 33: Fix Clippy Warnings**
- [ ] Remove `#![allow(...)]` from lib.rs
- [ ] Fix cast warnings with explicit conversions:
  ```rust
  // Instead of: let x = y as u32;
  // Use: let x = u32::try_from(y).expect("value fits in u32");
  ```
- [ ] Break up long functions (extract helpers)
- [ ] Add missing error documentation

**Day 34: Memory Layout Abstraction**
- [ ] Create `MemoryLayout` struct
- [ ] Move magic numbers:
  ```rust
  const GLOBAL_MEMORY_BASE: i32 = 0x30000;
  const SPILLED_LOCALS_BASE: i32 = 0x40000;
  // etc.
  ```
- [ ] Replace scattered constants with `layout.global_addr(idx)`
- [ ] Run tests

**Day 35: Assertions**
- [ ] Add debug assertions in spill_push/spill_pop
- [ ] Assert stack depth within bounds
- [ ] Assert register indices valid
- [ ] Run with debug builds to catch issues

**Day 36-37: Documentation**
- [ ] Document register conventions
- [ ] Document calling convention
- [ ] Add module-level docs

**Day 38: Review**
- [ ] Run clippy: should have 0 warnings
- [ ] Run tests: all should pass
- [ ] Review changes with team (if applicable)

**Deliverable**: Clean, documented, warning-free code

#### Week 7: Testing (Days 39-45)

**Day 39: Unit Test StackMachine**
- [ ] Test push/pop at various depths
- [ ] Test spill detection
- [ ] Test register assignment

**Day 40: Test Spilling**
- [ ] Test with stack depth 0-10
- [ ] Verify values preserved correctly
- [ ] Test interactions with function calls

**Day 41: Regression Tests**
- [ ] Add test for memory.copy overlap
- [ ] Add test for division overflow
- [ ] Add test for import return values

**Day 42: Edge Cases**
- [ ] Test with maximum locals (64)
- [ ] Test with deep call stack
- [ ] Test with empty functions

**Day 43-44: Coverage**
- [ ] Run `cargo tarpaulin`
- [ ] Identify uncovered code
- [ ] Add tests for uncovered paths

**Day 45: Validation**
- [ ] Coverage should be > 60%
- [ ] All new tests should pass
- [ ] Document test strategy

**Deliverable**: Comprehensive test suite with >60% coverage

---

### PHASE 4: Documentation & Completion (Weeks 8-10)
**Priority**: MEDIUM  
**Goal**: Complete V1, prepare for V2

#### Week 8: Documentation (Days 46-52)

**Day 46: Update AGENTS.md**
- [ ] Add section on critical bugs
- [ ] Document PVM-in-PVM current status
- [ ] Add "Common Pitfalls" section

**Day 47: Update PLAN.md**
- [ ] Mark completed phases
- [ ] Update V1 definition
- [ ] Add V2 planning section

**Day 48: Architecture Documentation**
- [ ] Create ARCHITECTURE.md
- [ ] Document current design (translator pattern)
- [ ] Document known limitations

**Day 49: API Documentation**
- [ ] Document public API
- [ ] Add examples
- [ ] Generate docs with `cargo doc`

**Day 50: User Guide**
- [ ] Create USAGE.md
- [ ] Step-by-step examples
- [ ] Troubleshooting guide

**Day 51-52: Review**
- [ ] Review all documentation
- [ ] Ensure consistency
- [ ] Fix typos and formatting

**Deliverable**: Complete documentation

#### Week 9: Final Validation (Days 53-59)

**Day 53-54: Full Test Suite**
- [ ] Run `cargo test` (all Rust tests)
- [ ] Run `bun scripts/test-all.ts` (62 integration tests)
- [ ] Run PVM-in-PVM core tests
- [ ] Record all results

**Day 55: Performance Check**
- [ ] Time compilation of anan-as
- [ ] Compare to baseline
- [ ] Ensure no major regressions

**Day 56: CI/CD**
- [ ] Ensure GitHub Actions workflow passes
- [ ] Add PVM-in-PVM test to CI
- [ ] Set up coverage reporting

**Day 57-58: V1 Review**
- [ ] Create V1 checklist
- [ ] Verify all MUST HAVE items complete
- [ ] Document SHOULD HAVE status

**Day 59: Prepare Release**
- [ ] Tag commit as v1.0
- [ ] Create release notes
- [ ] Update version numbers

**Deliverable**: V1 ready for release

#### Week 10: Buffer & Transition (Days 60-66)

**Day 60-62: Buffer Time**
- [ ] Fix any last-minute issues
- [ ] Address feedback
- [ ] Final polish

**Day 63-64: V2 Planning**
- [ ] Document V2 goals
- [ ] Create roadmap
- [ ] Prioritize features

**Day 65: Final Documentation**
- [ ] Update README.md
- [ ] Update CHANGELOG.md
- [ ] Final review

**Day 66: V1 Complete** 🎉

**Deliverable**: V1 complete, V2 planned

---

## Daily Workflow Template

### Morning (2-3 hours)
1. Review yesterday's notes
2. Pick task from current phase
3. Write/update test first (TDD style)
4. Implement/fix
5. Run tests frequently

### Afternoon (3-4 hours)
6. Complete implementation
7. Run full test suite
8. Document changes
9. Update this plan (mark tasks complete)
10. Plan tomorrow

### End of Day
- Commit changes with descriptive messages
- Update todo list
- Note any blockers

---

## Checkpoint Milestones

### Week 2 (Day 10)
**Checkpoint**: 3 critical bugs fixed
- [ ] memory.copy works for overlaps
- [ ] division overflow checks added
- [ ] import returns handled
- [ ] All 62 tests pass

### Week 5 (Day 31)
**Checkpoint**: PVM-in-PVM core tests pass
- [ ] add.jam.wat returns 12 in PVM-in-PVM
- [ ] factorial, fibonacci, gcd pass
- [ ] Root cause of PANIC identified and fixed

### Week 7 (Day 45)
**Checkpoint**: Code quality improved
- [ ] 0 clippy warnings
- [ ] >60% test coverage
- [ ] Documentation complete

### Week 10 (Day 66)
**Checkpoint**: V1 Complete
- [ ] All MUST HAVE items done
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Ready to tag v1.0

---

## Risk Mitigation

### Risk: PVM-in-PVM takes longer than 3 weeks
**Mitigation**:
- Focus on core 4 tests (add, factorial, fibonacci, gcd)
- Can declare V1 with partial PVM-in-PVM if direct tests all pass
- Document remaining PVM-in-PVM issues for V1.1

### Risk: Bug fixes cause regressions
**Mitigation**:
- Comprehensive testing after each fix
- Git history to revert if needed
- Fix one bug at a time

### Risk: Running out of time
**Mitigation**:
- Phase 4 (Documentation) can be shortened
- Can ship with minimal docs and improve later
- Core functionality is priority

---

## Success Criteria (Final)

### Technical
- [ ] 62 direct execution tests pass
- [ ] 4+ PVM-in-PVM tests pass (add, factorial, fibonacci, gcd)
- [ ] 0 critical bugs open
- [ ] >60% test coverage

### Quality
- [ ] 0 clippy warnings
- [ ] Code documented
- [ ] Architecture documented

### Completion
- [ ] V1 tagged in git
- [ ] Release notes published
- [ ] V2 roadmap created

---

## Appendix: Quick Reference

### Key Files
- `crates/wasm-pvm/src/translate/codegen.rs` - Main compiler (2,400 lines)
- `crates/wasm-pvm/src/translate/mod.rs` - Compilation orchestration
- `scripts/test-all.ts` - Test runner
- `scripts/test-pvm-in-pvm.ts` - PVM-in-PVM harness

### Key Test Commands
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

### Critical Code Locations
- memory.copy: `codegen.rs:2481-2599`
- I32DivS: `codegen.rs:1472-1481`
- I64DivS: `codegen.rs:1716-1725`
- Import handling: `codegen.rs:2210-2241`
- Stack spilling: `stack.rs:21-88`

---

*This is a living document. Update it as progress is made.*
