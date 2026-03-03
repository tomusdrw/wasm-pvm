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

## Approach

**Key insight**: Allocated registers use write-through — every `store_to_slot` writes to
both the physical register AND the stack slot. This means the allocated register always
holds the same value as the stack slot (unless the register was clobbered by a call).
Call clobbers are already handled by `reload_allocated_regs_after_call()`.

**Proposed change**: Stop clearing `alloc_reg_slot` at block boundaries. Only clear it
when something actually clobbers the register (calls, explicit invalidation). The general
register cache (`slot_cache`, `reg_to_slot`, `reg_to_const`) stays conservative.

**Safety argument**: Since writes to allocated slots always go through `store_to_slot`
(which updates both the register and the stack), and clobbers are tracked by
`invalidate_reg`/`reload_allocated_regs_after_call`, the register always holds the correct
value unless explicitly invalidated. The lazy reload in `load_operand` is a safety net.

## Log

### Step 1: Add targeted tests for cross-block regalloc

Before touching the optimization, add WAT fixtures and integration tests that
exercise cross-block allocated register behavior. These will serve as regression
tests regardless of the optimization outcome.

### Step 2: Implement alloc_reg_slot persistence

Modify `clear_reg_cache()` to NOT clear `alloc_reg_slot`. Only calls and explicit
invalidation should clear allocated register state.

### Step 3: Validate against full test suite

Run cargo test, integration tests, and pvm-in-pvm tests.

### Step 4: Benchmark and iterate

Compare gas usage with/without the optimization on targeted fixtures and the
standard benchmark set.
