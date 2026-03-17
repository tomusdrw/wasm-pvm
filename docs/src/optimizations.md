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

Linear-scan allocator assigns SSA values to physical registers, reducing `LoadIndU64` memory traffic. Allocates in all functions (looped and straight-line). Eviction uses a spill-weight model (`use_count × 10^loop_depth`) to keep loop-hot values in registers. Non-leaf functions with calls inside loop bodies are skipped (reload traffic outweighs savings). See the [Register Allocation](./regalloc.md) chapter for details.

## Aggressive Register Allocation (`--no-aggressive-regalloc`)

Lowers the minimum-use threshold for register allocation candidates from 2 to 1, capturing single-use values when a register is free. Enabled by default.

## Scratch Register Allocation (`--no-scratch-reg-alloc`)

Adds r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) to the allocatable set in leaf functions that don't clobber them (no bulk memory ops, no funnel shifts). Per-function LLVM IR scan detects clobbering operations. Doubles allocation capacity in the common case (e.g., 2-param function: 2 → 4 allocatable regs).

## Caller-Saved Register Allocation (`--no-caller-saved-alloc`)

Adds r7/r8 (`RETURN_VALUE_REG`/`ARGS_LEN_REG`) to the allocatable set in leaf functions. These registers are idle after the prologue. Lowering paths that use them as scratch (signed div, NE compare, multi-phi) trigger `invalidate_reg`, forcing lazy reload from the write-through stack slot. Combined with r5/r6, gives up to 6 extra registers in leaf functions.

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

## Adding a New Optimization

1. Add a field to `OptimizationFlags` in `translate/mod.rs`
2. Thread it through `LoweringContext` → `EmitterConfig`
3. Guard the optimization code with `e.config.<flag>`
4. Add a `--no-*` CLI flag in `wasm-pvm-cli/src/main.rs`

## Benchmarks

All optimizations enabled (default):

| Benchmark | WASM size | JAM size | Code size | Gas Used |
|-----------|----------|----------|-----------|----------|
| add(5,7) | 68 B | 245 B | 169 B | 58 |
| fib(20) | 110 B | 280 B | 195 B | 721 |
| factorial(10) | 102 B | 253 B | 171 B | 327 |
| is_prime(25) | 162 B | 388 B | 292 B | 111 |
| AS fib(10) | 235 B | 741 B | 601 B | 383 |
| AS factorial(7) | 234 B | 727 B | 589 B | 321 |
| AS gcd(2017,200) | 229 B | 728 B | 595 B | 231 |
| AS decoder | 1.5 KB | 21.9 KB | 7.5 KB | 896 |
| AS array | 1.4 KB | 20.9 KB | 6.6 KB | 763 |
| aslan-fib accumulate | - | 39.7 KB | 19.1 KB | 12,467 |
| anan-as PVM interpreter | 54.6 KB | 174.1 KB | 123.1 KB | - |
