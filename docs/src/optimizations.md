# Optimizations

All non-trivial optimizations can be individually toggled via `OptimizationFlags` (in `translate/mod.rs`, re-exported from `lib.rs`). Each defaults to enabled; CLI exposes `--no-*` flags.

## LLVM Passes (`--no-llvm-passes`)

Three-phase optimization pipeline:
1. `mem2reg`, `instcombine`, `simplifycfg` (pre-inline cleanup)
2. `cgscc(inline)` (optional, see `--no-inline`)
3. `instcombine<max-iterations=2>`, `simplifycfg`, `gvn`, `simplifycfg`, `dce`

## Function Inlining (`--no-inline`)

LLVM CGSCC inline pass for small callees. After inlining, `instcombine` may introduce new LLVM intrinsics (`llvm.abs`, `llvm.smax`, etc.) that the backend must handle.

## Peephole Optimizer (`--no-peephole`)

Post-codegen patterns in `pvm/peephole.rs`:
- **Fallthrough elimination**: remove redundant `Fallthrough` before jump/branch
- **Truncation NOP removal**: `[32-bit-producer] → AddImm32(x,x,0)` eliminated
- **Dead store elimination**: SP-relative stores never loaded from are removed
- **Immediate chain fusion**: `LoadImm + AddImm` → single `LoadImm`; chained `AddImm` → fused
- **Self-move elimination**: `MoveReg r, r` removed
- **Address calculation folding**: `AddImm` offsets folded into subsequent load/store offsets

## Register Cache (`--no-register-cache`)

Per-basic-block store-load forwarding. Tracks which stack slots are live in registers:
- **Cache hit, same register**: skip entirely (0 instructions)
- **Cache hit, different register**: emit register copy (1 instruction)
- **Cache miss**: normal load + record in cache

Impact: ~50% gas reduction, ~15-40% code size reduction.

Invalidated at block boundaries, after function calls, and after ecalli.

## Cross-Block Cache (`--no-cross-block-cache`)

When a block has exactly one predecessor and no phi nodes, the predecessor's cache snapshot is propagated instead of clearing. The snapshot is taken before the terminator instruction.

## ICmp+Branch Fusion (`--no-icmp-fusion`)

Combines an LLVM `icmp` + `br` pair into a single PVM branch instruction (e.g., `BranchLtU`), saving one instruction per conditional branch.

## Shrink Wrapping (`--no-shrink-wrap`)

For non-entry functions, only callee-saved registers (r9-r12) that are actually used are saved/restored in prologue/epilogue. Reduces frame header size from fixed 40 bytes to `8 + 8 * num_used_callee_regs`.

## Dead Store Elimination (`--no-dead-store-elim`)

Removes `StoreIndU64` instructions to SP-relative offsets that are never loaded from. Runs as part of the peephole optimizer.

## Constant Propagation (`--no-const-prop`)

Skips `LoadImm`/`LoadImm64` when the target register already holds the required constant value.

## Register Allocation (`--no-register-alloc`)

Linear-scan allocator assigns SSA values to physical registers, reducing `LoadIndU64` memory traffic. Allocates in all functions (looped and straight-line, leaf and non-leaf). Eviction uses a spill-weight model (`use_count × 10^loop_depth`) to keep loop-hot values in registers. In non-leaf functions, the existing call lowering (`spill_allocated_regs` + `clear_reg_cache` + lazy reload) handles spill/reload around calls automatically, and per-call-site arity-aware invalidation only clobbers registers used by each specific call. See the [Register Allocation](./regalloc.md) chapter for details.

## Aggressive Register Allocation (`--no-aggressive-regalloc`)

Lowers the minimum-use threshold for register allocation candidates from 2 to 1, capturing single-use values when a register is free. Enabled by default.

## Scratch Register Allocation (`--no-scratch-reg-alloc`)

Adds r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) to the allocatable set in all functions that don't clobber them (no bulk memory ops, no funnel shifts). Per-function LLVM IR scan detects clobbering operations. In non-leaf functions, r5/r6 are spilled before calls via `spill_allocated_regs` and lazily reloaded on next access. Doubles allocation capacity in the common case (e.g., 2-param function: 2 → 4 allocatable regs).

## Caller-Saved Register Allocation (`--no-caller-saved-alloc`)

Adds r7/r8 (`RETURN_VALUE_REG`/`ARGS_LEN_REG`) to the allocatable set in all functions. These registers are idle after the prologue. In non-leaf functions, r7/r8 are invalidated after calls via the arity-aware invalidation predicate (since calls always clobber r7 with the return value). Lowering paths that use them as scratch (signed div, NE compare, multi-phi) trigger `invalidate_reg`, forcing lazy reload from the write-through stack slot. Combined with r5/r6, gives up to 6 extra registers beyond callee-saved r9-r12.

## Dead Function Elimination (`--no-dead-function-elim`)

Removes functions not reachable from exports or the function table. Reduces code size for programs with unused library functions.

## Fallthrough Jump Elimination (`--no-fallthrough-jumps`)

When a block ends with an unconditional jump to the next block in layout order, the `Jump` is skipped — execution falls through naturally.

## Lazy Spill (`--no-lazy-spill`)

Eliminates write-through stack stores for register-allocated values. When a value is stored to a slot that has an allocated register, the value goes only into the register (marked "dirty") and the `StoreIndU64` to the stack is skipped. Values are flushed to the stack only when required:

- When the register is about to be clobbered by another instruction (auto-spill in `invalidate_reg`)
- Before function calls and ecalli (via `spill_allocated_regs()`)
- Before the function epilogue (return)
- Before terminators at block boundaries
- After prologue parameter stores

With **register-aware phi resolution** (Phase 5), phi copies between blocks use direct register-to-register moves when both the incoming value and the phi destination are in allocated registers, avoiding stack round-trips. The target block restores `alloc_reg_slot` for phi destinations after `define_label`, so subsequent reads use the register directly. For mixed cases (some values allocated, some not), a two-pass approach loads all incoming values into temp registers, then stores to destinations (registers or stack). This handles all dependency cases including cycles without needing a separate parallel move resolver.

Requires `register_allocation` to be effective.

## Store-Side Coalescing (Phase 7)

When a value has an allocated register, `result_reg()` / `result_reg_or()` helpers in `emitter.rs` return that register directly. ALU, memory load, and intrinsic lowering paths use the result register as their output destination instead of TEMP_RESULT (r4), so `store_to_slot` no longer needs to emit a `MoveReg` to copy from TEMP_RESULT into the allocated register.

This is a codegen-only optimization (no new flag) — it is always active when register allocation is enabled.

**Not coalesced** (store-side correctness constraints):
- `lower_select`: loading the default value into the allocated register corrupts register cache state needed by subsequent operand loads (load-side coalescing IS applied — see Phase 9 below)
- `emit_pvm_memory_grow`: TEMP_RESULT is used across control flow (branch between grow success/failure)
- `lower_abs` intrinsic: TEMP_RESULT is used across control flow (branch between positive/negative paths)

**`result_reg_or()` variant**: Some lowering paths (zext, sext, trunc) need TEMP1 as the fallback register instead of TEMP_RESULT to preserve register cache behavior in non-allocated paths. `result_reg_or(fallback)` returns the allocated register when available, or the specified fallback otherwise.

**Impact** (anan-as compiler): 54% reduction in store_moves (2720 to 1262), 4% reduction in total instructions (37225 to 35744), 2.9% reduction in JAM size (169,853 to 164,902 bytes).

## Load-Side Coalescing (Phase 8)

When a value is live in its allocated register, `operand_reg()` returns that register directly instead of requiring `load_operand()` to copy it into a temp register (TEMP1/TEMP2). The instruction's source operand fields use the allocated register, eliminating the `MoveReg` that `load_operand()` would otherwise emit.

Applied across all lowering modules:
- **alu.rs**: Binary arithmetic (register-register and immediate-folding paths), comparisons, zext/sext/trunc
- **memory.rs**: PVM load/store address and value operands, global store values
- **control_flow.rs**: Branch conditions, fused ICmp+Branch operands, switch values
- **intrinsics.rs**: min/max, bswap, ctlz/cttz/ctpop, rotation operands

**Not coalesced** (complexity/safety constraints):
- Div/rem operations: intermediate trap code may clobber scratch registers
- Non-rotation funnel shifts: use SCRATCH1/SCRATCH2 after spill_allocated_regs
- `lower_abs`: control flow between positive/negative paths
- Call argument setup: already loaded into specific registers
- Phi resolution: already uses register-aware moves

**Dst-conflict safety**: When an operand's allocated register matches the destination register (`result_reg`), the operand falls back to the temp register to avoid invalidation hazards from `emit() → invalidate_reg(dst)`.

This is a codegen-only optimization — always active when register allocation is enabled.

## Phase 9: Select Coalescing, Spill Weight Refinement & Call Return Hints

Three allocator improvements added in Phase 9:

### Select Coalescing (load-side)

`lower_select` now uses `operand_reg()` for all Cmov operands (default value, condition, and source). Values already in their allocated registers are used directly as CmovNz/CmovIz/CmovNzImm/CmovIzImm operands without MoveReg copies. Store-side coalescing (using `result_reg()` for the Cmov dst) remains deferred due to the `invalidate_reg` cache corruption issue documented in Phase 7.

### Spill Weight Refinement

Values whose live ranges span real call instructions receive a penalty to their spill weight. Each spanning call costs `CALL_SPANNING_PENALTY` (2.0) weight, representing the spill+reload pair required when a register is allocated across a call boundary. The formula:

```text
effective_weight = base_weight - (num_spanning_calls × 2.0)
```

Values spanning many calls get lower weights, making them more likely to be evicted in favor of values with fewer spanning calls. This improves allocation decisions in call-heavy functions. Call positions are collected during linearization using the same `is_real_call()` check from `emitter.rs`; counting uses binary search for efficiency.

### Call Return Value Coalescing (register hints)

When a value is defined by a real call instruction, the linear scan allocator prefers assigning r7 (`RETURN_VALUE_REG`). Since call return values are already in r7, this eliminates the `MoveReg` from r7 to the allocated register in `store_to_slot`. The hint is best-effort — if r7 is not free, a different register is used.

All three are codegen-only optimizations — always active when register allocation is enabled.

## Phase 10: Loop Phi Early Interval Expiration

Eliminates phi MoveReg instructions in loop headers by modifying the linear scan to expire loop phi destination intervals at their actual last use (before loop extension) instead of the loop-extended end. This frees the phi's register earlier, allowing the incoming back-edge value to naturally reuse it via the free register pool. When both values share the same register, the phi copy at the back-edge becomes a no-op (skipped entirely in `emit_phi_copies_regaware`).

Three coordinated changes:
1. **regalloc.rs**: `LiveInterval.expiration` field — for loop phi destinations where `pre_extension_end < end`, expires early. Pressure guard disables when `intervals > 2× registers`.
2. **control_flow.rs**: Phi copy no-op filter — when `incoming_reg == phi_reg` and `is_alloc_reg_valid(src_reg, incoming_slot)`, skips data movement.
3. **emitter.rs**: `store_to_slot` safety — spills dirty values before overwriting `alloc_reg_slot` with a different slot.

Impact: fib(20) -15.7% gas / -7.2% code, factorial -5.6% gas. No regressions.

This is a codegen-only optimization — always active when register allocation is enabled.

## Phase 11: Cross-Block Alloc State Propagation

Improves register allocation state propagation at block boundaries, particularly at loop headers with back-edges. Previously, blocks with unprocessed predecessors (back-edges) cleared `alloc_reg_slot` entirely, forcing reloads from the stack at every loop iteration start. Phase 11 instead propagates the dominator predecessor's alloc state, filtered by safety:

- **Non-leaf functions**: Only callee-saved registers beyond `max_call_args` are propagated (these are never clobbered by calls). Caller-saved registers (r5-r8) are excluded because they may be invalidated after calls on other paths.
- **Leaf functions with lazy spill**: All registers are propagated (no calls to clobber them).
- **Multi-predecessor blocks (leaf+lazy_spill)**: The existing intersection logic (keep only entries where all processed predecessors agree) is now also applied to leaf functions with lazy spill, not just non-leaf functions.

New emitter method `set_alloc_reg_slot_filtered()` selectively propagates alloc entries based on a register filter predicate, enabling the per-register-class filtering described above.

The predecessor map (`pred_map`) is now built for both non-leaf functions AND leaf functions with lazy spill (condition: `has_regalloc && (!is_leaf || lazy_spill_enabled)`).

Impact: fib(20) -5.1% gas, factorial(10) -7.1% gas, is_prime(25) -4.6% gas, PiP aslan-fib -0.52% gas.

This is a codegen-only optimization — always active when register allocation and lazy spill are enabled.

## Phase 12: Callee-Saved Preference for Call-Spanning Intervals

In non-leaf functions, the linear scan allocator now applies register class preferences based on whether an interval spans call instructions:

- **Call-spanning intervals** (live range contains at least one real call) prefer callee-saved registers (r9-r12 beyond `max_call_args`). These registers survive calls without invalidation, eliminating post-call reload traffic.
- **Non-call-spanning intervals** prefer caller-saved registers (r5-r8), leaving callee-saved registers available for call-spanning values.
- **Leaf functions** use the default `pop()` behavior — all registers are equal since there are no calls.

The `preferred_reg` hint (e.g., r7 for call return values) takes priority over the class preference.

Implementation: `LiveInterval.spans_calls` field set during interval construction based on `count_spanning_calls() > 0`. The `linear_scan()` function receives `is_leaf` and applies class-aware register selection.

Impact: Primarily benefits non-leaf functions with call-spanning values. anan-as PVM interpreter -0.2% code size (106,820→106,577 bytes).

This is a codegen-only optimization — always active when register allocation is enabled.

## Adding a New Optimization

1. Add a field to `OptimizationFlags` in `translate/mod.rs`
2. Thread it through `LoweringContext` → `EmitterConfig`
3. Guard the optimization code with `e.config.<flag>`
4. Add a `--no-*` CLI flag in `wasm-pvm-cli/src/main.rs`

## Benchmarks

All optimizations enabled (default):

| Benchmark | WASM size | JAM size | Code size | Gas Used |
|-----------|----------|----------|-----------|----------|
| add(5,7) | 68 B | 165 B | - | 28 |
| fib(20) | 110 B | 247 B | - | 511 |
| factorial(10) | 102 B | 209 B | - | 178 |
| is_prime(25) | 162 B | 293 B | - | 65 |
| AS fib(10) | 235 B | 640 B | - | 247 |
| AS factorial(7) | 234 B | 625 B | - | 209 |
| AS gcd(2017,200) | 229 B | 649 B | - | 176 |
| AS decoder | 1.5 KB | 20.8 KB | - | 637 |
| AS array | 1.4 KB | 20.0 KB | - | 557 |
| regalloc two loops | - | 595 B | - | 16,776 |
| aslan-fib accumulate | - | 38.5 KB | - | 11,089 |
| anan-as PVM interpreter | 54.6 KB | 158.9 KB | - | - |
