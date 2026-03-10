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

Linear-scan allocator assigns frequently-used values (filtered by live-interval analysis and a minimum-use threshold) to available callee-saved registers (r9-r12). Only functions with loops are considered; loop-free functions skip allocation entirely. See the [Register Allocation](./regalloc.md) chapter for details.

## Dead Function Elimination (`--no-dead-function-elim`)

Removes functions not reachable from exports or the function table. Reduces code size for programs with unused library functions.

## Fallthrough Jump Elimination (`--no-fallthrough-jumps`)

When a block ends with an unconditional jump to the next block in layout order, the `Jump` is skipped — execution falls through naturally.

## Adding a New Optimization

1. Add a field to `OptimizationFlags` in `translate/mod.rs`
2. Thread it through `LoweringContext` → `EmitterConfig`
3. Guard the optimization code with `e.config.<flag>`
4. Add a `--no-*` CLI flag in `wasm-pvm-cli/src/main.rs`

## Benchmarks

All optimizations enabled (default):

| Benchmark | WASM size | JAM size | Code size | Gas Used |
|-----------|----------|----------|-----------|----------|
| add(5,7) | 68 B | 201 B | 130 B | 39 |
| fib(20) | 110 B | 270 B | 186 B | 612 |
| factorial(10) | 102 B | 242 B | 161 B | 269 |
| is_prime(25) | 162 B | 328 B | 239 B | 80 |
| AS fib(10) | 234 B | 708 B | 572 B | 324 |
| AS factorial(7) | 233 B | 697 B | 562 B | 281 |
| AS gcd(2017,200) | 228 B | 686 B | 558 B | 190 |
| AS decoder | 1.5 KB | 20.8 KB | 6.8 KB | 721 |
| AS array | 1.4 KB | 19.9 KB | 6.0 KB | 623 |
| aslan-fib accumulate | 7.8 KB | 37.1 KB | 17.6 KB | 15,968 |
| anan-as PVM interpreter | 57.7 KB | 180.2 KB | 127.8 KB | - |
