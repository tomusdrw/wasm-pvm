# Optimization-flag impact analysis

**Run date**: 2026-05-18
**Compiler revision**: `td-audit-unused-opts` (branched from `main` at 5ca8556)
**Methodology**: `./experiments/opt_impact.sh` — toggle each `--no-X` flag individually, recompile every input with all-other-opts on, diff JAM size + PVM code size vs. baseline.

## Input set (31)

- 13 hand-crafted WAT fixtures (`tests/fixtures/wat/*.jam.wat`) — `add`, `fibonacci`, `factorial`, `is-prime`, `regalloc-two-loops`, `blake2b`, `sha512`, `u128-mul-bench`, `u128-div-bench`, `u128-div-bench-slow`, `host-call-log`, `memory-copy-word`, `memory-copy-overlap`.
- 0 AS-built WASM (`tests/build/wasm/*.wasm`) — the script tries to pick up `as-fibonacci`, `as-factorial`, `as-gcd`, `as-decoder-test`, `as-array-test`, `as-alloc-test`, `as-string-test`, but none were present in this workspace (bun didn't produce them); the script silently skips missing inputs. Re-running in a tree where `cd tests && bun build.ts` has populated the AS artifacts will add ≤7 rows.
- 3 as-lan services — `aslan-fib`, `aslan-ecalli`, `aslan-debug`.
- 1 large real-world AS service — `anan-as-compiler` (PVM interpreter implemented in AssemblyScript).
- 14 polkadot-fellows v2.2.2 runtimes — full asset-hub / bridge-hub / collectives / coretime / encointer / glutton / kusama / people / polkadot / bulletin set, compiled with `--trap-floats`.

Total: 13 + 0 + 3 + 1 + 14 = **31 inputs**.

## Headline results

| Flag | Total JAM Δ | Affected | Verdict |
|------|---:|---:|---|
| `--no-register-cache` | **+78.9 MB** | 29/31 | Critical. Largest single saver. |
| `--no-dead-store-elim` | **+41.5 MB** | 31/31 | Critical. Fires on every input. |
| `--no-icmp-fusion` | **+7.5 MB** | 27/31 | Critical on polkadot. |
| `--no-fallthrough-jumps` | **+4.2 MB** | 29/31 | Critical. |
| `--no-mergefunc` | +1.45 MB | 15/31 | Net win; regresses aslan-debug by 394 B. |
| `--no-register-alloc` | +1.07 MB | 30/31 | Solid. |
| `--no-cross-block-cache` | +1.01 MB | 24/31 | Solid. |
| `--no-scratch-reg-alloc` | +975 KB | 24/31 | Solid. |
| `--no-const-prop` | +475 KB | 19/31 | Polkadot-dominated. |
| `--no-aggressive-regalloc` | +421 KB | 30/31 | **Mixed** — counterproductive on aslan-*, factorial, regalloc-two-loops, u128-div, glutton (−465 to −9). |
| `--no-lazy-spill` | +277 KB | 29/31 | Solid (polkadot-dominated). −1407 B on anan-as-compiler. |
| `--no-peephole` | +246 KB | 19/31 | Effectively polkadot-only — fires on almost nothing under 50 KB. |
| `--no-caller-saved-alloc` | +47 KB | 26/31 | Polkadot-only positive; negative on fibonacci, factorial, is-prime, u128-mul-bench, aslan-fib, aslan-ecalli (−2 to −14). |
| `--no-inline` | +3.4 KB | 19/31 | **Mostly noise.** −5 B on 8/14 polkadot runtimes, −526 B on aslan-ecalli. Default `inline_threshold=5` may be miscalibrated. |
| `--no-shrink-wrap` | +1.5 KB | 23/31 | **Marginal noise.** Negative on kusama (−396), polkadot (−313), people-polkadot (−26). |
| `--no-libcall-recognition` | +858 B | 17/31 | **Works as designed.** +62 B/runtime on polkadot, +157 B on u128-mul. −82 B on u128-div fast/slow (recognition adds static dispatch — gas-vs-size trade). |
| `--no-llvm-passes` | — | **31 failed** | Not an optimization toggle — compilation collapses (`Unsupported WASM feature: LLVM opcode Alloca`) because the PVM backend requires mem2reg. |
| ~~`--no-dead-function-elim`~~ | n/a | n/a | **Removed in #244** after the experiment showed +0 B on 0/31 inputs. Upstream tooling (AssemblyScript tree-shaking, wasm-opt, Substrate's runtime build) already prunes unreachable functions before WASM reaches us. |

## Findings & follow-ups

1. ~~**`dead_function_elimination`** never fires on any of the 31 inputs.~~ Removed in #244 — the pass was correct but redundant with upstream tooling (AssemblyScript tree-shaking, wasm-opt, Substrate's runtime build).

2. **`--no-llvm-passes` in `NO_OPT_FLAGS`** (`tests/utils/benchmark.sh:30`) silently breaks every non-trivial benchmark under `--no-opt`, because mem2reg is mandatory for the PVM backend (alloca lowering isn't implemented). Remove from the bulk-disable array and document the flag as a frontend-debugging escape hatch, not a tunable optimization. Tracked in #245.

3. **Noisy / net-negative-on-subsets** — `shrink_wrap_callee_saves`, `inlining` (default threshold), `aggressive_register_allocation`, `caller_saved_alloc`, `libcall_recognition`. JAM size alone can't decide their fate because some of them trade size for gas. Re-measure with gas usage on the dynamic fixtures (everything in `BENCHMARKS` with non-empty `args`) before deciding to keep, tune, or remove. Tracked in #246.

4. **Per-project optimization config** — the input-class-specific behavior above (e.g. `mergefunc` wins on polkadot but regresses aslan-debug; `aggressive_register_allocation` wins on polkadot but regresses all aslan-*) suggests projects should be able to opt into/out of individual passes via a `wasm-pvm.toml` file rather than long CLI flag lists. Tracked in #247.

5. **`mergefunc`** is net-positive at scale (+1.45 MB across 15 polkadot runtimes when off, i.e. mergefunc saved that much when on) but regresses small modules (aslan-debug −394 B). The fix is probably a per-module guard (skip below some function-count threshold), not deletion. Low priority.

6. **`aggressive_register_allocation`** is the most "uneven" winner — large polkadot wins, consistent regressions on small/loop-heavy code. The min-use=1 threshold over-promotes values that then get spilled. Worth a per-function heuristic instead of a global flag. Low priority.

## Caveats

- All sizes are static (JAM file + PVM code section). Gas and wall-time impact are not measured here — see follow-up #3 above.
- Each flag is toggled in isolation. Interaction effects (e.g. `register_cache` off + `cross_block_cache` off) aren't characterized.
- Polkadot runtimes were compiled with `--trap-floats`; the size of generated trap stubs is included in baseline. Removing `--trap-floats` would shift baseline numbers (the float wall would surface as a hard error).
- The input set lacks Rust-compiled WASM. Rust workloads with heavy `__multi3`/`__udivti3` use would significantly raise the `libcall_recognition` impact.
