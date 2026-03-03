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

## Approach 2: Leaf-function-only + predecessor intersection

**Key insight**: In leaf functions (no calls), allocated registers (r9-r12) are ONLY
written by `store_to_slot` (write-through) and `load_operand` (reload). Both correctly
update `alloc_reg_slot`. So `alloc_reg_slot` is ALWAYS accurate in leaf functions.

**For non-leaf functions**: Use predecessor exit snapshot intersection. At multi-predecessor
blocks, only keep `alloc_reg_slot` entries where ALL processed predecessors agree. For
back-edges (unprocessed predecessors), be conservative.

**Implementation plan**:
1. For leaf functions: keep alloc_reg_slot across all block boundaries
2. For non-leaf functions with single predecessor: already propagated via cross-block cache
3. For non-leaf functions with multiple predecessors (all processed): intersect
4. For non-leaf functions with loop headers (back-edge = unprocessed): clear

## Log

### Step 1: Add targeted tests (DONE)
- `regalloc-nested-loops.jam.wat` — nested loops with multiple carried values
- `regalloc-loop-with-call.jam.wat` — loop calling a function (non-leaf)
- Both pass on baseline, commit e0bfda7

### Step 2: Blanket alloc_reg_slot persistence (FAILED)
- Layers 1-3: 422 pass
- PVM-in-PVM: 2 failures in `as-decoder-subarray-test`
- Root cause: multi-predecessor blocks with inconsistent predecessor states

### Step 3: Implement leaf-function-only propagation
- Safe because leaf functions have no calls → no register clobbers
- Also implement predecessor intersection for forward merge points in non-leaf functions
