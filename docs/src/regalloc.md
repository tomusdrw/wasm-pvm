# Register Allocation

The compiler uses a **linear-scan register allocator** to assign frequently-used SSA values to physical callee-saved registers (r9-r12), reducing memory traffic.

## Overview

Every LLVM SSA value gets a dedicated 8-byte stack slot (the baseline). The register allocator improves on this by keeping hot values in registers across block boundaries and loop iterations.

### Eligibility

- Only functions with loop back-edges are considered (loop-free functions skip allocation)
- Values must have ≥3 uses (`MIN_USES_FOR_ALLOCATION`)
- Live intervals are computed from use-def analysis with loop extension

### Available Registers

Callee-saved registers r9-r12, minus those used for incoming parameters:
- A function with 2 parameters uses r9-r10 → r11-r12 are available for allocation
- In non-leaf functions, registers needed for outgoing call arguments are also reserved

### Allocation Strategy

1. Build candidate intervals from use-def live-interval analysis
2. Filter by minimum-use threshold
3. Run linear scan: assign to available callee-saved registers, evict lower-priority intervals when needed
4. Naturally expired intervals remain in the mapping (earlier uses still benefit)
5. Evicted intervals are removed entirely (whole-interval mapping invalid after eviction)

### Runtime Integration

- `load_operand` checks regalloc before stack: uses `MoveReg` from allocated reg instead of `LoadIndU64`
- `store_to_slot` uses write-through: copies to allocated reg AND stores to stack
- Dead store elimination removes the stack store if never loaded
- After calls in non-leaf functions, allocated register mappings are invalidated and lazily reloaded

## Cross-Block Propagation

- **Leaf functions**: `alloc_reg_slot` is preserved across all block boundaries (allocated registers are never clobbered by calls)
- **Non-leaf functions**: Predecessor exit snapshots are intersected at multi-predecessor blocks — only entries where ALL predecessors agree are kept
- Back-edges (unprocessed predecessors) are treated conservatively

## Debugging

Enable allocator logs with `RUST_LOG=wasm_pvm::regalloc=debug`:
- `regalloc::run()` prints candidate/assignment stats
- `lower_function()` prints per-function usage counters (`alloc_load_hits`, `alloc_store_hits`, etc.)

Quick triage:
- `allocatable_regs=0` → no allocation will happen
- Non-zero `allocated_values` with near-zero load/store hits → move/reload overhead dominates

For the full development journey, see [Regalloc Cross-Block Journey](./regalloc-journey.md).
