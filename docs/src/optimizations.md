# Optimizations

All non-trivial optimizations can be individually toggled via `OptimizationFlags` (in `translate/mod.rs`, re-exported from `lib.rs`). Each defaults to enabled; CLI exposes `--no-*` flags.

## LLVM Pass Pipeline

Four phases run on every compile. The whole pipeline is gated by the `llvm_passes` flag (CLI `--no-llvm-passes`); the inlining and mergefunc phases also have individual toggles.

1. `mem2reg`, `instcombine`, `simplifycfg` (pre-inline cleanup)
2. `cgscc(inline)` (optional, see `--no-inline`)
3. `instcombine<max-iterations=20>`, `simplifycfg`, `gvn`, `simplifycfg`, `dce`
4. `mergefunc` (optional, see `--no-mergefunc`)

### `--no-llvm-passes` (debug only)

**Not a tunable optimization.** This flag skips the *entire* pipeline above, including `mem2reg`. The PVM backend cannot lower `alloca` / unpromoted SSA — every input non-trivial enough to use locals (i.e. virtually every real WASM module) fails with:

```text
Error: Unsupported WASM feature: LLVM opcode Alloca (in function #N during PVM lowering)
```

Per the `experiments/opt_impact.sh` sweep, **31 of 31 representative inputs** (fixture WATs, AS-built WASM, polkadot runtimes) fail to compile with this flag set. Use only to inspect the raw frontend IR (`--verbose` / dumps) before any optimization runs. Do not include it in `--no-opt` bundles or treat it as comparable to `--no-peephole`, `--no-register-cache`, etc.

## Function Inlining (`--no-inline`)

LLVM CGSCC inline pass for small callees. After inlining, `instcombine` may introduce new LLVM intrinsics (`llvm.abs`, `llvm.smax`, etc.) that the backend must handle.

## Function Merging (`--no-mergefunc`)

LLVM's `mergefunc` pass, run as Phase 4 after the function-level cleanup. Two behaviors:
- **Aliasing**: when two functions have byte-identical bodies and their linkage permits, one becomes an alias of the other and only one PVM body survives.
- **Thunking**: when functions are "weakly identical" (same shape, parameterizable differences), the pass factors a canonical body and emits thunks (`call canonical; ret`) for the originals.

Targets rustc monomorphizations — `quicksort` instantiated for several comparator types, `scale_info::TypeInfo::type_info` instantiated for many newtype wrappers. Their bodies share opcode shape but differ in inner call targets; the thunk parameterization handles this.

**Must run after inlining.** If `cgscc(inline)` ran after `mergefunc`, the thunks (very small bodies) would inline back into every caller and undo the merge. No trailing `dce` because the thunks are reachable from their callers — `dce` would drop nothing and only cost compile time.

**Net effect on tiny functions can be negative** because each thunked call costs ~5 bytes of call setup, which exceeds the saved body for very short functions. The polkadot wins come from large monomorphized helpers where the body dwarfs the call overhead.

**Impact** (polkadot fellowship v2.2.2, `--trap-floats`):

| Runtime | WASM | Baseline code | With mergefunc | Δ |
|---------|-----:|--------------:|---------------:|---:|
| `glutton-kusama` | 2.04 MiB | 4,636,361 B | 4,600,277 B | **−0.78%** |
| `kusama` | 8.43 MiB | 17,965,423 B | 17,832,758 B | **−0.74%** |

Saving scales roughly linearly with binary size. Compile-time impact: negligible (~+150 ms on glutton; within noise on kusama).

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

Lowers the minimum-use threshold for register allocation candidates from 2 to 1, capturing more values when a register is free. Enabled by default.

## Scratch Register Allocation (`--no-scratch-reg-alloc`)

Adds r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) to the allocatable set in all functions that don't clobber them (no bulk memory ops, no funnel shifts). Per-function LLVM IR scan detects clobbering operations. In non-leaf functions, r5/r6 are spilled before calls via `spill_allocated_regs` and lazily reloaded on next access. Doubles allocation capacity in the common case (e.g., 2-param function: 2 → 4 allocatable regs).

## Caller-Saved Register Allocation (`--no-caller-saved-alloc`)

Adds r7/r8 (`RETURN_VALUE_REG`/`ARGS_LEN_REG`) to the allocatable set in leaf functions. These registers are idle after the prologue and are never clobbered by calls in leaf functions. In non-leaf functions, r7/r8 are not allocated because every call clobbers r7 (return value) and r8 (scratch), making the constant invalidation/reload overhead a net negative. Combined with r5/r6, gives up to 4 extra registers (r5, r6, r7, r8) beyond callee-saved r9-r12 in leaf functions. The full register convention: r0=return address, r1=SP, r2-r4=temps, r5-r6=scratch, r7=return value/args ptr, r8=args len, r9-r12=callee-saved locals.

## Fallthrough Jump Elimination (`--no-fallthrough-jumps`)

Two coupled steps that elide trailing `Jump` instructions when the jump target is the next block in emission order:

1. **Block layout reorder.** `compute_block_layout` in `llvm_backend/mod.rs` constructs the per-function emission order via greedy trace: from each unplaced block, walk preferred-successor links (uncond `br dest` → `dest`, cond `br cond, then, else` → `else` since `lower_br` emits `BranchIfX then; Jump else_label`, `switch` → `default`). Iterate the original IR order to pick trace starts. The resulting layout is shared with the register allocator so live intervals are computed against the order the emitter actually executes; `regalloc::run` accepts the layout as the `block_order` parameter for that reason.
2. **Jump elision.** When `emit_jump_to_label` is invoked with the next block in layout already known (`next_block_label`), the `Jump` is dropped — `define_label` emits a `Fallthrough` marker (1 byte) instead.

Trampoline paths in `lower_br` / `lower_switch` (used when phi copies are needed on every outgoing edge) emit a final `Jump` to a different target than the layout's preferred-next. Such blocks miss the fallthrough but remain correct.

## Libcall Recognition (`--no-libcall-recognition`)

Replaces the body of recognized **compiler-builtins runtime functions** with hand-crafted PVM-friendly implementations. WASM has no `i128` type, so `rustc` for `wasm32-unknown-unknown` lowers every `(a as u128) * b`, `a / b` and similar to calls into runtime helpers (`__multi3`, `__udivti3`). Those helpers carry their full Knuth-style bodies into the WASM (~30 IR instructions for `__multi3`, ~800+ for `__udivti3` + `specialized_div_rem`); when we recognize them by name we can swap the body for something that uses PVM's native opcodes directly.

**Recognition is name-based**, by matching the function's WASM custom `name` section entry against a fixed table:

| Name | Replacement |
|------|-------------|
| `__multi3` | 8 PVM instructions: `Mul64 + MulUpperUU + 2×Mul64 + 2×Add64 + 2×StoreIndU64` |
| `__udivti3` | Fast/slow dispatch on `(a_hi \| b_hi) == 0`: fast path is `DivU64` + 2 stores; slow path forwards to the original `specialized_div_rem` (compiler-builtins) via the same stack-frame setup as the original wrapper |

Each recognition also checks the signature (5 `i64` params, no return — the C `sret` convention) so a user function that happens to share a name isn't silently mis-translated. For `__udivti3`, the body is also scanned to extract the slow-path callee and the `__stack_pointer` global; without both we silently no-op.

**Impact** (microbenchmark, 1000 iterations of the underlying operation):

| Workload | With | Without | Δ Gas | Δ Size |
|----------|------|---------|-------|--------|
| u128 mul | 75,029 | 119,029 | **−37%** | **−170 B** |
| u128 div (fast path, `a_hi = b_hi = 0`) | 76,029 | 129,029 | **−41%** | +110 B |
| u128 div (slow path, `b_hi != 0`) | 143,029 | 129,029 | +11% | +110 B |

The `__udivti3` fast path is the b_hi specialization win: when callers pass `i64 0` for the high halves (the dominant shape in substrate's `Perbill` / currency arithmetic), it becomes a 5-PVM-instruction inline divide. The 11% slow-path regression is the cost of the dispatch (`Or + ICmp + Branch`) — accepted because real workloads are dominated by the fast path.

**Limitations** (documented in `crates/wasm-pvm/src/llvm_frontend/libcall_recognition.rs`):

- Strips of the WASM `name` custom section disable recognition silently (no correctness impact).
- Aggressive inlining (`--inline-threshold` > body size) inlines the libcall everywhere; recognition still applies but the inlined call sites still run the slow original. A separate IR pattern matcher would be needed to catch those — explicitly out of scope.
- A user function literally named `__multi3` with the exact 5-i64-param signature would be silently replaced. Mitigation: signature gate + the names are reserved by the C/Rust ABI.

## Lazy Spill (`--no-lazy-spill`)

Eliminates write-through stack stores for register-allocated values. When a value is stored to a slot that has an allocated register, the value goes only into the register (marked "dirty") and the `StoreIndU64` to the stack is skipped. Values are flushed to the stack only when required:

- When the register is about to be clobbered by another instruction (auto-spill in `invalidate_reg`)
- Before function calls and ecalli (via `spill_allocated_regs()`)
- Before the function epilogue (return)
- Before terminators at block boundaries
- After prologue parameter stores

With **register-aware phi resolution**, phi copies between blocks use direct register-to-register moves when both the incoming value and the phi destination are in allocated registers, avoiding stack round-trips. The target block restores `alloc_reg_slot` for phi destinations after `define_label`, so subsequent reads use the register directly. For mixed cases (some values allocated, some not), a two-pass approach loads all incoming values into temp registers, then stores to destinations (registers or stack). This handles all dependency cases including cycles without needing a separate parallel move resolver.

Requires `register_allocation` to be effective.

---

The sections below are **codegen-only optimizations**: no individual flag, always active when `register_allocation` is enabled. Implementation in `llvm_backend/emitter.rs` and `llvm_backend/regalloc.rs`.

## Store-Side Coalescing

`result_reg()` / `result_reg_or(fallback)` in `emitter.rs` return a value's allocated register so ALU / memory-load / intrinsic lowering writes the result there directly, eliminating the `MoveReg` from TEMP_RESULT that `store_to_slot` would otherwise emit. The `_or(TEMP1)` variant is used by zext/sext/trunc to preserve TEMP1-based cache behavior in the non-allocated path.

**Not coalesced** (TEMP_RESULT live across control flow, or load corrupts cache for subsequent operand loads): `lower_select`, `emit_pvm_memory_grow`, `lower_abs`.

**Impact** (anan-as compiler): store_moves 2720 → 1262 (−54%), instructions 37,225 → 35,744 (−4%), JAM 169,853 → 164,902 B (−2.9%).

## Load-Side Coalescing

`operand_reg()` returns a value's allocated register when it currently holds the right slot, so lowering uses it directly as the instr's source operand instead of going through `load_operand()` + temp copy. Applied across binary arith (incl. immediate-folding), comparisons, zext/sext/trunc, load/store addresses and values, branch conditions, fused ICmp+Branch, switch values, min/max, bswap, ctlz/cttz/ctpop, rotations, and `lower_select` Cmov operands.

**Not coalesced**: div/rem (trap code clobbers SCRATCH1), non-rotation funnel shifts (use SCRATCH1/2 after spill), `lower_abs`, call argument setup, phi resolution.

**Dst-conflict safety** (`apply_dst_conflict_fallback`): when an operand's allocated register matches the dst, fall back to the temp register to avoid `invalidate_reg` hazards. Exception: `dst == TEMP_RESULT` keeps the alias (PVM reads both srcs before writing dst), eliminating `MoveReg r4 → r2` chains. `bitreverse` keeps the conservative fallback (clobbers TEMP_RESULT mid-sequence to materialize i64 masks).

**Impact** of dst==TEMP_RESULT relaxation alone on polkadot/glutton-kusama: MoveReg −61% (70,141 → 27,155), PVM instructions −4%, JAM −1.97%.

## Spill Weight Refinement

`effective_weight = base_weight − num_spanning_calls × 2.0`. Live ranges that cross real call instructions get a 2.0 penalty per spanning call (representing the spill+reload pair), pushing the allocator toward values that don't cross call boundaries. Call positions collected during linearization via `is_real_call()`, counted via binary search.

## Call Return Value Coalescing

`LiveInterval.preferred_reg` hints r7 (`RETURN_VALUE_REG`) for values defined by real calls — the return value is already in r7, so picking r7 (when free) eliminates the post-call `MoveReg`. Best-effort; if r7 is taken, a different register is used.

## Loop Phi Early Interval Expiration

Loop phi destination intervals expire at their actual last use (before loop extension), freeing the register early so the incoming back-edge value can take it via the free pool. When both share the register, the phi copy becomes a no-op (`emit_phi_copies_regaware` skips it when `incoming_reg == phi_reg` AND `is_alloc_reg_valid` confirms the register still holds the incoming value). `store_to_slot` spills dirty values before overwriting `alloc_reg_slot` with a different slot.

A blanket pressure guard (`intervals > 2× registers`) disables this under register pressure, preventing freed registers from being stolen by unrelated values. Per-phi guards are unworkable — see `learnings.md` "Per-Phi Early Expiration Guard".

**Impact**: fib(20) −15.7% gas / −7.2% code, factorial −5.6% gas.

## Cross-Block Alloc State Propagation

At block boundaries with unprocessed predecessors (back-edges at loop headers), the dominator predecessor's `alloc_reg_slot` is propagated instead of cleared. Filtered per register class to stay correct:

- **Non-leaf**: only callee-saved beyond `max_call_args` (r5–r8 may be invalidated after calls on other paths).
- **Leaf + lazy spill**: all registers (no calls to clobber them).
- **Multi-predecessor blocks** (both flavors): intersection logic keeps entries all processed predecessors agree on.

`pred_map` is built when `has_regalloc && (!is_leaf || lazy_spill_enabled)`; `set_alloc_reg_slot_filtered()` applies the per-class filter.

**Impact**: fib(20) −5.1% gas, factorial(10) −7.1%, is_prime(25) −4.6%, PiP aslan-fib −0.52%.

## Callee-Saved Preference for Call-Spanning Intervals

In non-leaf functions, the linear scan prefers callee-saved (r9–r12 beyond `max_call_args`) for intervals that span real calls (these survive calls without invalidation) and caller-saved (r5–r8) for intervals that don't. `LiveInterval.spans_calls` set during interval construction; `linear_scan()` reads `is_leaf` and picks accordingly. `preferred_reg` hints (e.g. r7 for call returns) take priority. Leaf functions use default `pop()` order (no calls = no preference needed).

**Impact**: anan-as PVM interpreter −0.2% code (106,820 → 106,577 B). Primarily helps non-leaf functions with call-spanning values.

## Adding a New Optimization

1. Add a field to `OptimizationFlags` in `translate/mod.rs`
2. Thread it through `LoweringContext` → `EmitterConfig`
3. Guard the optimization code with `e.config.<flag>`
4. Add a `--no-*` CLI flag in `wasm-pvm-cli/src/main.rs`

## Benchmarks

All optimizations enabled (default):

| Benchmark | WASM size | JAM size | Code size | Gas Used |
|-----------|----------|----------|-----------|----------|
| add(5,7) | 68 B | 164 B | 99 B | 28 |
| fib(20) | 110 B | 226 B | 148 B | 409 |
| factorial(10) | 102 B | 198 B | 124 B | 156 |
| is_prime(25) | 162 B | 285 B | 201 B | 62 |
| AS fib(10) | 235 B | 631 B | 504 B | 245 |
| AS factorial(7) | 234 B | 616 B | 490 B | 207 |
| AS gcd(2017,200) | 229 B | 640 B | 517 B | 174 |
| AS decoder | 1.5 KB | 6.6 KB | 4,944 B | 953 |
| AS array | 1.4 KB | 6.1 KB | 4,427 B | 820 |
| regalloc two loops | 252 B | 587 B | 461 B | 16,769 |
| host-call-log | 171 B | 458 B | 104 B | 40 |
| aslan-fib accumulate | - | 20.7 KB | 13,365 B | 11,474 |
| blake2b("abc", 32) | 1.1 KB | 3.8 KB | 2,558 B | 17,930 |
| sha512("abc") | 1.7 KB | 3.7 KB | 2,559 B | 17,981 |
| anan-as PVM interpreter | 53.4 KB | 115.6 KB | 84,281 B | - |
