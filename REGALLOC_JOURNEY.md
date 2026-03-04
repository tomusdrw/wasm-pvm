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

## Discovery: Leaf detection was broken (THE MAIN WIN)

**Critical finding**: ALL functions with memory access were classified as non-leaf because
PVM intrinsics (`__pvm_load_i32`, `__pvm_store_i32`, etc.) are LLVM `Call` instructions.
These are NOT real function calls — they're lowered inline using temp registers and never
use the calling convention.

**Fix**: Added `is_real_call()` to distinguish real calls (`wasm_func_*`, `__pvm_call_indirect`)
from intrinsics (`__pvm_*`, `llvm.*`).

**Impact**: Significant improvements because leaf functions get smaller stack frames
(no callee-save prologue/epilogue):

| Benchmark | Code Change | Gas Change |
|-----------|-------------|------------|
| AS decoder | -2.9% | -4.0% |
| AS array | -3.2% | -3.7% |
| PiP TRAP | 0 | -3.3% |
| PiP add | 0 | -1.0% |
| PiP Jambrains | 0 | -1.9% |
| is_prime | +0.4% | +2.6% (tiny: +2 gas absolute) |

## Attempt: Phi node allocation (REVERTED)

**Hypothesis**: Phi nodes at loop headers represent loop-carried variables (induction
variables, accumulators). Allow them to be register-allocated.

**Result**: All tests pass, but **gas regressions on key benchmarks**:
- `is_prime`: +6.4% gas
- `AS factorial`: +8.2% gas
- `regalloc two loops`: +8.8% gas

**Root cause**: In PVM, all basic instructions cost 1 gas. Write-through adds 1 MoveReg
per phi copy per iteration. The "saved" load is just LoadIndU64 → MoveReg (same cost).
Net: **+1 gas per iteration per allocated phi node**. The write-through model makes phi
node allocation a gas regression in the current PVM gas model.

**Learning**: Register allocation for phi nodes only makes sense when:
- Loads are cheaper than stores (not the case in PVM: both cost 1 gas)
- OR the allocated register can be used directly without MoveReg to temp
  (not the case: allocated regs are r9-r12, temps are r2-r4)
- OR code size matters more than gas (MoveReg is 2 bytes vs LoadIndU64's 5 bytes)

## Final Results (Leaf Detection + Cross-Block Propagation)

| Benchmark | JAM Size | Code Size | Gas Change |
|-----------|----------|-----------|------------|
| AS decoder | -1.1% | -2.9% | -4.0% |
| AS array | -1.1% | -3.2% | -3.7% |
| anan-as PVM interpreter | -0.6% | -0.8% | - |
| PiP TRAP | 0 | 0 | -3.3% |
| PiP Jambrains | 0 | 0 | -1.9% |
| PiP JADE | 0 | 0 | -0.8% |
| is_prime | +0.3% | +0.4% | +2.6% |

## Log

### Step 1: Add targeted tests (DONE) — commit e0bfda7
- `regalloc-nested-loops.jam.wat` — nested loops with multiple carried values
- `regalloc-loop-with-call.jam.wat` — loop calling a function (non-leaf)

### Step 2: Blanket alloc_reg_slot persistence (FAILED)
- PVM-in-PVM: 2 failures in `as-decoder-subarray-test`
- Root cause: multi-predecessor blocks with inconsistent predecessor states

### Step 3: Leaf-only propagation + predecessor intersection (DONE) — commit e8694cd
- All 695 tests pass, zero benchmark impact (regalloc rarely activates)

### Step 4: Fix leaf detection (DONE) — commit 6960512
- Distinguish PVM intrinsics from real calls
- Up to -4% gas, -3.2% code size on real workloads

### Step 5: Phi node allocation (REVERTED) — commit 6af12fa → reverted 3445375
- Gas regression due to write-through MoveReg overhead

## Future Opportunities

1. **Direct phi-to-register allocation**: Instead of write-through to stack + MoveReg to
   allocated reg, emit phi copies directly to the allocated register and skip the stack
   store entirely (DSE would need to remove the dead store). This would make phi allocation
   gas-neutral and code-size-positive.

2. **Load-from-allocated-register without MoveReg**: When the consumer of an allocated
   value can use r9-r12 directly (instead of requiring TEMP1/TEMP2), avoid the MoveReg.
   This requires instruction selection awareness of allocated registers.

3. **Non-leaf loop-safe propagation**: For non-leaf functions, propagate alloc_reg_slot
   at loop headers where the loop body has no calls (requires loop-body analysis).
