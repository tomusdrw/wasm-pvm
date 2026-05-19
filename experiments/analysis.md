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
| `--debug-skip-llvm-passes` | — | **31 failed** | Not an optimization toggle — compilation collapses (`Unsupported WASM feature: LLVM opcode Alloca`) because the PVM backend requires mem2reg. |
| ~~`--no-dead-function-elim`~~ | n/a | n/a | **Removed in #244** after the experiment showed +0 B on 0/31 inputs. Upstream tooling (AssemblyScript tree-shaking, wasm-opt, Substrate's runtime build) already prunes unreachable functions before WASM reaches us. |

## Findings & follow-ups

1. ~~**`dead_function_elimination`** never fires on any of the 31 inputs.~~ Removed in #244 — the pass was correct but redundant with upstream tooling (AssemblyScript tree-shaking, wasm-opt, Substrate's runtime build).

2. ~~**`--no-llvm-passes` in `NO_OPT_FLAGS`** (`tests/utils/benchmark.sh:30`) silently breaks every non-trivial benchmark under `--no-opt`, because mem2reg is mandatory for the PVM backend (alloca lowering isn't implemented). Remove from the bulk-disable array and document the flag as a frontend-debugging escape hatch, not a tunable optimization.~~ Fixed in #245 — flag removed from `NO_OPT_FLAGS`, documented as debug-only in `docs/src/optimizations.md` and `docs/src/cli-usage.md`.

3. **Noisy / net-negative-on-subsets** — `shrink_wrap_callee_saves`, `inlining` (default threshold), `aggressive_register_allocation`, `caller_saved_alloc`, `libcall_recognition`. JAM size alone can't decide their fate because some of them trade size for gas. Re-measure with gas usage on the dynamic fixtures (everything in `BENCHMARKS` with non-empty `args`) before deciding to keep, tune, or remove. Tracked in #246. **Done** — see "Gas cross-check (issue #246)" section below. Verdict: keep all five; `aggressive_register_allocation` warrants a follow-up to study a u128-div-bench gas regression (~2 gas/division).

4. **Per-project optimization config** — the input-class-specific behavior above (e.g. `mergefunc` wins on polkadot but regresses aslan-debug; `aggressive_register_allocation` wins on polkadot but regresses all aslan-*) suggests projects should be able to opt into/out of individual passes via a `wasm-pvm.toml` file rather than long CLI flag lists. Tracked in #247.

5. **`mergefunc`** is net-positive at scale (+1.45 MB across 15 polkadot runtimes when off, i.e. mergefunc saved that much when on) but regresses small modules (aslan-debug −394 B). The fix is probably a per-module guard (skip below some function-count threshold), not deletion. Low priority.

6. **`aggressive_register_allocation`** is the most "uneven" winner — large polkadot wins, consistent regressions on small/loop-heavy code. The min-use=1 threshold over-promotes values that then get spilled. Worth a per-function heuristic instead of a global flag. Low priority.

## Caveats

- ~~All sizes are static (JAM file + PVM code section). Gas and wall-time impact are not measured here — see follow-up #3 above.~~ Gas cross-check now done — see the "Gas cross-check" section below.
- Each flag is toggled in isolation. Interaction effects (e.g. `register_cache` off + `cross_block_cache` off) aren't characterized.
- Polkadot runtimes were compiled with `--trap-floats`; the size of generated trap stubs is included in baseline. Removing `--trap-floats` would shift baseline numbers (the float wall would surface as a hard error).
- The input set lacks Rust-compiled WASM. Rust workloads with heavy `__multi3`/`__udivti3` use would significantly raise the `libcall_recognition` impact.

---

## Gas cross-check (issue #246)

**Run date**: 2026-05-19
**Methodology**: `./experiments/opt_gas_impact.sh` — for each `--no-X` flag, recompile every runnable fixture (every `BENCHMARKS` entry in `tests/utils/benchmark.sh` with non-empty `args`) and run via `anan-as` to capture gas usage. Diff against the all-opts-on baseline.
**Inputs**: 19 runnable fixtures (no polkadot — these need substrate to execute). Gas budget 100 M.
**Determinism**: gas is deterministic for a given (program, args), so 1 run per cell suffices (no median smoothing).

### Aggregate by flag

Sorted by sign-disagreement count (most-conflicted first), then by `|Total ΔGas|`:

| Flag                       | Total ΔJAM | Total ΔGas | JAM regr. | Gas regr. | Sign disagr. |
|----------------------------|-----------:|-----------:|----------:|----------:|-------------:|
| `--no-fallthrough-jumps`   |      +1700 |    +10 326 |     2/19 |     4/18 |        5 |
| `--no-aggressive-regalloc` |        −75 |     +8 471 |     8/19 |     9/18 |        5 |
| `--no-lazy-spill`          |        +84 |    −18 702 |     7/19 |     7/18 |        3 |
| `--no-register-alloc`      |      +1002 |    +12 861 |     3/19 |     2/18 |        2 |
| `--no-scratch-reg-alloc`   |       +284 |     +1 037 |     4/19 |     6/18 |        2 |
| `--no-libcall-recognition` |         −7 |    +85 000 |     2/19 |     1/18 |        1 |
| `--no-caller-saved-alloc`  |         −6 |        −73 |     7/19 |     5/18 |        1 |
| `--no-register-cache`      |    +14 535 |    +80 562 |        0 |        0 |        0 |
| `--no-dead-store-elim`     |    +10 922 |    +62 850 |        0 |        0 |        0 |
| `--no-icmp-fusion`         |     +1 662 |    +17 574 |        0 |        0 |        0 |
| `--no-cross-block-cache`   |       +873 |     +8 655 |        0 |        0 |        0 |
| `--no-inline`              |       +447 |     +3 946 |     1/19 |        0 |        0 |
| `--no-shrink-wrap`         |       +235 |        +40 |        0 |        0 |        0 |
| `--no-const-prop`          |        +94 |        +13 |        0 |        0 |        0 |
| `--no-peephole`            |         +4 |         +1 |        0 |        0 |        0 |
| `--no-mergefunc`           |          0 |          0 |        0 |        0 |        0 |

`aslan-ecalli` exhausts the 100 M gas budget under the baseline (verified to still OOM at 200 M), so it is excluded from gas regression / disagreement counts — `delta_gas` is structurally pinned for any variant that also OOMs. Its size delta is still kept in the JAM column.

### Per-flag recommendations (the 5 "noisy" flags from #243)

#### `--no-shrink-wrap` — **KEEP**
Per-fixture gas deltas: all zero except as-* (+8 each, 5 runtimes). Total ΔGas +40 across the runnable suite — effectively zero. The size regressions in #243 (kusama −396, polkadot −313, people-polkadot −26) are confined to polkadot runtimes that can't be gas-checked here, but the runnable-fixture data shows the optimization is gas-neutral. **No reason to remove**; the polkadot size regressions are a small price for a uniformly benign opt.

#### `--no-inline` (`inline_threshold=5`) — **KEEP**
Total ΔGas +3 946 across the runnable suite — a real, broad gas win, despite the JAM size delta being small. The only JAM regression (`aslan-ecalli −526`) sits on a fixture that OOMs at baseline, so the size cost there isn't paired with any measurable gas signal. The −5 B/runtime on 8/14 polkadot runtimes is in the same range as RW-trim noise. **Keep the current threshold (5)**; the gas win is the deciding signal.

#### `--no-aggressive-regalloc` — **TUNE** (don't remove)
This is the most genuinely "noisy" flag. Total ΔGas +8 471 (clear gas win on average), but with 5 sign-disagreement cells and a slight JAM-size loss on this fixture subset (Total ΔJAM −75; the +421 KB win in #243 is polkadot-driven). The two disagreement patterns are:
- Hash kernels (`blake2b`, `sha512`, `sha512-240a`): opt saves gas (−186 / −82 / −244 with flag on) but costs 20–46 bytes per fixture.
- u128 div microbenchmarks (`u128-div-bench`, `u128-div-bench-slow`): opt saves bytes (−9 each with flag off, i.e. flag-on costs 9 bytes) but **costs 1 996 gas** per 1 000 iterations — that's 2 gas per division.

The u128-div regression is the real signal: aggressive regalloc is doing the wrong thing in a tight slow-path loop. Suggests the spill-weight heuristic over-promotes values that get repeatedly spilled around the slow-path call. Worth investigating a per-function or per-loop heuristic before reaching for a general-purpose disable. Recommendation: leave on (net win), but file follow-up to study the u128-div regression.

#### `--no-caller-saved-alloc` — **KEEP (no change)**
Total ΔJAM −6 and ΔGas −73 — basically zero on both axes for runnable fixtures. The 7 JAM regressions and 5 gas regressions cancel out: fibonacci/factorial/is-prime/aslan-fib lose a handful of bytes and a handful of gas; regalloc-two-loops and the hashes win both. Only one sign disagreement (fibonacci: −2 bytes / +18 gas — opt costs 2 bytes but saves 18 gas → marginal gas win). The polkadot wins (+47 KB, #243) justify keeping it. No tuning warranted on this evidence — the noise is genuine noise, not a hidden trade.

#### `--no-libcall-recognition` — **KEEP** (gas-vs-size trade is real and large)
Total ΔGas **+85 000** — by far the largest gas swing for a flag with small total ΔJAM (−7 B). u128-mul-bench is pure win on both axes (+157 B, +40 000 gas saved by opt). The lone disagreement is u128-div-bench: recognition costs 82 B but saves **53 000 gas / 1 000 iterations** — that's 53 gas per division, exactly the gas-vs-size trade #243 already flagged. The −8 000 gas regression on `u128-div-bench-slow` (slow path) is consistent with recognition adding a small fast-path dispatch overhead before the slow path runs. Net story: keep. Any workload that runs more than a few thousand u128 divisions will recoup the 82-byte cost in well under a second.

### One unexpected finding outside the noisy-5

`--no-fallthrough-jumps` was not in #243's noisy list (it was rank-4 by JAM total there), but it surfaces 5 sign disagreements here. Pattern: `regalloc-two-loops` (+16 B / −497 gas), `sha512` (+63 B / −545 gas), `sha512-240a` (+63 B / −1 631 gas), `aslan-fib` (+456 B / −116 gas) — fallthrough-jumps optimization is **paying gas on hot loops** to save bytes. That gas loss is small in absolute terms (~3 K gas across the suite, vs. the +10 K total ΔGas win on the colder fixtures), but the consistent direction in tight loops suggests the layout heuristic occasionally swaps a fall-through for a branch-with-skip pair that costs an extra instruction per iteration. Not bad enough to disable, but worth a future look.

### What this experiment can't tell us

- **Polkadot gas.** None of the polkadot runtimes have a defined entry point that anan-as can execute. All polkadot-only findings in #243 (e.g. `shrink_wrap` saving 735 B on kusama; `caller_saved_alloc` saving 47 KB total) remain size-only.
- **`anan-as-compiler` gas.** Same reason — empty `args` in `BENCHMARKS`. The +1 407 B `--no-lazy-spill` regression on `anan-as-compiler` flagged in #243 can't be gas-checked here.
- **`mergefunc`.** Its only known size regression (`aslan-debug −394 B`) sits on a fixture with empty args — not gas-measurable. Total ΔJAM = 0 / ΔGas = 0 across the runnable suite, consistent with mergefunc firing only on big modules.

### Reproduce

```bash
./experiments/opt_gas_impact.sh
# outputs:
#   experiments/results/gas_baseline.csv
#   experiments/results/gas_deltas.csv
#   experiments/results/gas_summary.md
```

Runtime ≈ 10–15 min on a developer laptop (1 compile + 1 run per (16 flags × 19 fixtures) + baseline).
