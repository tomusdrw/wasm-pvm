# Regalloc Cross-Block Propagation Journey

**Issue**: [#127](https://github.com/tomusdrw/wasm-pvm/issues/127)
**Branch**: `feature/regalloc-cross-block-propagation`
**Goal**: Propagate allocated-register state across block boundaries to avoid unnecessary reloads, especially at loop headers.

## Current State (Baseline)

The register allocator assigns loop-carried values to callee-saved registers (r9-r12).
The runtime tracking (`alloc_reg_slot`) is cleared at every block boundary that doesn't
qualify for single-predecessor cross-block cache propagation. This means loop headers
(which have 2+ predecessors: preheader + back-edge) always start cold, requiring a
reload on first use of each allocated value per loop iteration.

## Attempt 1: Blanket alloc_reg_slot persistence (FAILED)

**Change**: Remove `clear_allocated_reg_state()` from `clear_reg_cache()` so
`alloc_reg_slot` is never cleared at block boundaries.

**Result**: Layers 1-3 (422 tests) pass. PVM-in-PVM fails on `as-decoder-subarray-test`
(2 failures). Direct execution of the same tests passes.

**Root cause analysis**: Multi-predecessor blocks (merge points) are unsafe because
different predecessors may leave allocated registers in different states:
- Block B has a call → r9 is clobbered at runtime, `alloc_reg_slot[r9] = None`
- Block C has no call → `alloc_reg_slot[r9] = Some(S)` at compile time
- Block D (successor of both B and C) inherits C's state (last processed)
- At runtime via B: r9 holds garbage but compile-time state says `Some(S)` → skip reload

The write-through argument only holds when NO instruction clobbers the register between
the last write-through and the block entry. Calls clobber r9-r12.

## Approach 2: Leaf-function-only + predecessor intersection (IMPLEMENTED)

**Key insight**: In leaf functions (no calls), allocated registers (r9-r12) are ONLY
written by `store_to_slot` (write-through) and `load_operand` (reload). Both correctly
update `alloc_reg_slot`. So `alloc_reg_slot` is ALWAYS accurate in leaf functions.

**For non-leaf functions**: Use predecessor exit snapshot intersection. At multi-predecessor
blocks, only keep `alloc_reg_slot` entries where ALL processed predecessors agree. For
back-edges (unprocessed predecessors), be conservative.

## Discovery: Leaf detection was broken

**Critical finding**: ALL functions with memory access were classified as non-leaf because
PVM intrinsics (`__pvm_load_i32`, `__pvm_store_i32`, etc.) are LLVM `Call` instructions.
These are NOT real function calls — they're lowered inline using temp registers and never
use the calling convention.

**Fix**: Added `is_real_call()` to distinguish real calls (`wasm_func_*`, `__pvm_call_indirect`)
from intrinsics (`__pvm_*`, `llvm.*`).

**Impact**: This fix alone (before cross-block propagation) produces significant improvements
because leaf functions get smaller stack frames (no callee-save prologue/epilogue):

| Benchmark | Code Change | Gas Change |
|-----------|-------------|------------|
| AS decoder | -2.9% | -4.0% |
| AS array | -3.2% | -3.7% |
| PiP TRAP | 0 | -3.3% |
| PiP add | 0 | -1.0% |
| PiP Jambrains | 0 | -1.9% |
| is_prime | +0.4% | +2.6% (tiny absolute: +2 gas) |

## Discovery: Regalloc candidate filtering too aggressive

**Finding**: After fixing leaf detection, regalloc now activates for more functions
(e.g., `is_prime` now gets `allocated_values=1`). However, most loop-heavy WAT fixtures
still show `total_intervals=0` because:

1. LLVM optimizes loop variables into phi nodes at the loop header (inside the loop)
2. The regalloc filter `defined_in_loop = true` excludes these
3. The filter requires `use_count >= 3` which eliminates many values after LLVM optimization

The regalloc was designed for loop-invariant values, NOT induction variables. This is a
separate optimization opportunity.

## Log

### Step 1: Add targeted tests (DONE) — commit e0bfda7
- `regalloc-nested-loops.jam.wat` — nested loops with multiple carried values
- `regalloc-loop-with-call.jam.wat` — loop calling a function (non-leaf)

### Step 2: Blanket alloc_reg_slot persistence (FAILED)
- Layers 1-3: 422 pass
- PVM-in-PVM: 2 failures in `as-decoder-subarray-test`

### Step 3: Leaf-only propagation + predecessor intersection (DONE) — commit e8694cd
- All 422 layer 1-3 tests pass
- All 273 pvm-in-pvm tests pass
- Zero benchmark impact (regalloc rarely activates due to broken leaf detection)

### Step 4: Fix leaf detection (DONE) — commit 6960512
- Distinguish PVM intrinsics from real calls
- Significant benchmark improvements (up to -4% gas, -3.2% code size)
- All 695 tests pass (422 layer + 273 pvm-in-pvm)

### Next steps
- Consider relaxing regalloc candidate filtering to include phi-node induction variables
- Investigate `is_prime` micro-regression (+2 gas from MoveReg overhead)
