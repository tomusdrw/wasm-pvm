# Deep dive: gas & size of emitted PVM code (pattern mining session)

**Run date**: 2026-06-12
**Compiler revision**: `main` @ 418277a
**Goal**: go beyond per-flag toggling (`analysis.md`) and mine the *emitted
instruction streams* of large outputs for systematic inefficiencies, weight
them by dynamic gas, and prototype the top fixes.

## Corpus

| Input | Toolchain | Instrs | Code bytes |
|---|---|---:|---:|
| `polkadot_runtime-v2002002` (--trap-floats) | Rust/wasm-ld | 4,318,396 | 16.8 MB |
| `glutton-kusama_runtime-v2002002` (--trap-floats) | Rust/wasm-ld | 1,196,365 | 4.6 MB |
| `anan-as-compiler` | AssemblyScript | 21,898 | 76 KB |

Dynamic profiles (per-PC execution counts; gas in PVM is exactly 1/instruction,
verified: steps == gas used): `sha512(240×"a")`, `blake2b("abc")`,
`u128-div-bench(1000)`, `as-gcd`.

## Tooling added (this session)

- `crates/wasm-pvm/examples/disasm.rs` — JAM → one instruction per line
  (reuses `Instruction::decode`; mask-driven; dumps jump table).
- `experiments/mine_patterns.py` — opcode histograms, targeted idiom
  detectors, abstracted n-grams (canonical register renaming, immediate
  bucketing), basic-block segmentation from branch targets + jump table.
- `experiments/mine_dataflow.py` — in-block abstract interpretation
  (upper-bits knowledge, slot↔register tracking), branch-offset relaxation
  fixpoint, LoadImm64 constant census.
- `experiments/profile_pcs.ts` — single-step PC histogram via anan-as
  debugger API (`resetJAM`/`nextStep`/`getProgramCounter`).
- `experiments/profile_join.py` — joins profile × disasm → dynamic gas share
  per pattern category.

## Headline: where the bytes and the gas go

Static (polkadot runtime; glutton is the same shape):

| Category | % instrs | % bytes |
|---|---:|---:|
| SP-relative loads (spill reads) | 25.9 | 25.1 |
| SP-relative stores (spill writes) | 16.9 | 17.2 |
| zext32 shift-pairs (`ShloLImm64 32; ShloRImm64 32`) | 5.2 | 7.9 |
| sext32 idioms (`AddImm32 d,s,0`) | 3.5 | 1.8 |
| `MoveReg` | 2.1 | 1.1 |
| `LoadImm64` (always 10 B) | 1.1 | 2.8 |
| Branch/jump offsets (fixed 4-byte encoding waste) | — | 4.3 |

Dynamic (share of gas):

| Category | sha512-240a | blake2b | u128div | as-gcd |
|---|---:|---:|---:|---:|
| spill load + store | 29.2% | 37.5% | 39.7% | 31.7% |
| zext32 shift-pairs | 7.1% | 8.3% | 0% | 8.5% |
| LoadImm/LoadImm64 | 12.6% | 6.3% | 14.7% | 4.3% |
| MoveReg | 2.4% | 2.3% | 7.4% | 8.5% |
| sext32 | 0.1% | 3.9% | 0% | 6.7% |
| real work (ALU+mem+control) | ~49% | ~40% | ~35% | ~38% |

**Roughly half of all gas is overhead** (spill traffic, extension idioms,
constant rematerialization, phi-copy moves), not computation.

---

## Critical incidental finding: latent PVM-in-PVM miscompile on main

While validating the DCE prototype, layer5 `as-simpler-repro` PVM-in-PVM
tests failed — and bisection proved the failure is **not** caused by the
prototype: `wasm-pvm` (current main, *all* optimizations disabled)
miscompiles the anan-as interpreter in an input-data-dependent way. Two
semantically identical inner programs (differing by one provably-dead
instruction + byte shifts) behave differently **only** under the
PVM-compiled interpreter; the same `compiler.wasm` run natively executes
both correctly. Full writeup + minimal A/B reproducer + native cross-check
harness: `experiments/repro-pip-miscompile/`.

Consequences:
1. There is a real, reachable miscompile in core lowering (highest-priority
   correctness bug; predates this session).
2. PVM-in-PVM tests are byte-layout-sensitive: any compiler change that
   shifts inner-program bytes can flip them red/green spuriously. The DCE
   prototype below is blocked on fixing this, not on its own correctness.

---

## Findings & prototypes, ranked

### 1. Peephole DCE is gated off for ~every real function (prototyped: −2.0% JAM on glutton)

`peephole.rs::optimize()` only calls `eliminate_dead_code` when the function
has **no labels at all** — i.e. only single-block functions; effectively
never. The DCE implementation itself already handles labels conservatively
(resets liveness at label offsets and terminators), so the gate nullifies a
working pass. Two real blockers found and fixed in the prototype
(`experiments/dce-ungate.patch`):

- `optimize()` reused the `keep[]` array across DCE's internal compaction
  (index desync — latent miscompile risk in the current gated arrangement
  too, hidden by a debug_assert);
- `CmovIz/CmovNz/CmovIzImm/CmovNzImm` conditionally write `dst` (dst flows
  through when the condition fails) but `src_regs()` doesn't include `dst`,
  so backward liveness would wrongly clear it. Fixed by treating cmov dst as
  read-and-written.

Why so much dead code exists: `optimize_address_calculation` folds
`AddImm64 dst, base, C` into the following `Load/StoreInd` offset but leaves
the now-dead AddImm in place — one dead instruction per folded wasm memory
access with a static offset (verified with a minimal WAT reproducer). DCE
emulation on the corpus: **3.11% of all instructions removable on glutton
(2.6% of bytes), 3.06% on anan-as** — single backward sweep, dominated by
`AddImm64` (30,435 of 37,181 on glutton).

Measured with the patch: glutton JAM 6,401,998 → 6,273,712 B (**−128,286 B,
−2.0%**); all Rust unit tests, layers 1–3 and 142 differential tests pass.
Small hand-written fixtures barely move (LLVM already cleans them; e.g.
as-gcd −4 B/−1 gas) — the win concentrates exactly where the user cares:
large Rust modules. Gas win on polkadot-class workloads is expected
proportional to how many dead adds sit in hot blocks (statically 3.1%), but
is not directly measurable (runtimes need substrate to execute).

**Status**: patch ready, blocked by the PVM-in-PVM latent bug above (3
layer5 tests flip due to byte shifts). Land after that fix.

### 2. Branch/jump offsets are always fixed 4-byte (estimated −4.3% to −5.5% code size, gas-free)

`encode_one_reg_one_imm_one_off` / `encode_two_reg_one_off` / `Jump` emit
`offset.to_le_bytes()` — always 4 bytes — because fixups patch offsets in
place after layout. ALU immediates are already minimally encoded; offsets are
not. Iterative relaxation fixpoint over the real corpora (recomputing
distances as instructions shrink) gives:

- glutton: **−198,637 B (−4.32% of code)**
- anan-as-compiler: −4,670 B (−5.5%)
- polkadot runtime: same ratio ≈ −700 KB

Zero gas impact (gas is per instruction, not per byte). Implementation:
assembler-style relaxation loop in `resolve_fixups` / blob encoding
(start everything at minimal length, grow until fixpoint, then patch).
Moderate effort, pure size win on every module.

### 3. zext32 shift-pairs: 5.2% of instructions, 7–8% of hot-kernel gas

61,821 `ShloLImm64 32; ShloRImm64 32` pairs in glutton (plus 2,163
`LoadImm64 0xFFFFFFFF; And` — the same mask where the backend falls into the
generic path). Sources: wasm32 address zero-extension and `i64.extend_i32_u`
(LLVM canonicalizes `zext(trunc)` to `and x, 0xFFFFFFFF`, which doesn't fit
`AndImm`'s sign-extended i32 immediate).

In-block provable elimination is near zero (34 of 61,821) — the producers
sit behind spill slots/other blocks — so a peephole won't touch this. Two
real options, both at the *lowering* level where LLVM-IR producer info is
available (SSA, cross-block):

- **(a) Address-context elision.** Most pairs feed `__pvm_load/store`
  address operands. For any wasm memory < 2 GB, `sext32(addr)` and
  `zext32(addr)` agree on all valid addresses and both trap on all invalid
  ones (sext'd negative + membase wraps to ≥2^63 → unmapped → panic). When
  the address operand of a memory intrinsic is a single-use
  `and x, 0xFFFFFFFF` / `zext i32`, consume `x` directly. Caveat to
  document: programs that *deliberately* wrap i32 address arithmetic
  (well-defined in WASM, never emitted by LLVM/AS for valid pointers) would
  trap instead of accessing the wrapped address — same trust class as the
  existing absent bounds-check. Should be a flag.
- **(b) Known-bits zext/sext skipping.** When lowering `zext i32→i64`,
  inspect the LLVM operand's defining instruction: loads (`LoadIndU32`
  zero-extends), `icmp` results, `lshr ≥32`, `and` with positive constants
  etc. are already zero-extended in their slots — skip the pair. Same for
  `sext` after instructions lowered to PVM 32-bit ops (which sign-extend by
  definition; the adjacent-pair case is already peepholed, the cross-block
  case is not: 15.9 K `LoadIndU64; AddImm32 d,src,0` pairs in glutton).

### 4. Constant-address loads/stores don't use absolute forms (u128div: ~7–9% gas)

Hot loop (1000×/iteration each):

```text
LoadImm  r2, 256 ; LoadIndU64 r5, [r2+196872]     → LoadU64  r5, 197128
LoadImm  r2, 0   ; LoadImm r3, 196616 ; StoreIndU64 [r3+0], r2
                                                   → StoreImmU64 196616, 0
```

`optimize_address_calculation` tracks `AddImm`/`MoveReg` but not
`LoadImm rX, C` as a base — extending it to rewrite `LoadInd/StoreInd
{base:rX, offset:D}` → absolute `Load/Store {address:C+D}` removes 1–2
instructions per site; the dead `LoadImm` then falls out via finding #1.
Static counterpart: `loadimm_then_<op>` detectors (~12 K instances on
polkadot incl. the Xor/SetLtU imm-form misses).

### 5. Loop-invariant constants rematerialized every iteration (u128div: 14.7% of gas is LoadImm/64)

The 10-byte `LoadImm64 r3, 0xCAFE...` divisor constant plus several small
`LoadImm`s are re-executed 1000×: `reg_to_const` constant propagation is
per-block and cleared at every label, so loop bodies reload all constants
each iteration. Options: extend cross-block cache propagation to
`reg_to_const` (cheap, same single-pred rule as the register cache), or let
regalloc treat frequently-used constants as allocatable values
(loop-invariant hoisting).

### 6. Spill traffic is THE structural cost (40%+ of everything) — needs a plan, not a peephole

`43.5%` of glutton instructions are SP-relative loads/stores; 29–40% of
dynamic gas across all profiled benchmarks. The stack-slot-per-SSA-value
design pays one store per def + one load per (non-cached) use. In-block
provable redundancy is small (~1% — the caches work); the cost is the
*architecture*. Directions, in increasing ambition:

- Lower-hanging: phi-copy `MoveReg` chains (u128div spends 7.4% of gas on
  MoveReg, mostly 3-move phi rotations) — a copy-coalescing pass on phi
  copies could kill most.
- The dead-store-elimination pass only targets `StoreIndU64` never loaded
  *anywhere in the function* — per-path liveness would catch more.
- Real fix: extend linear-scan to allocate across more registers and make
  slots the exception rather than the rule (currently r9–r12 + conditionally
  r5–r8; non-leaf r7/r8 is a documented dead end, but the bulk of the
  traffic is mid-pressure functions where 6–8 allocatable regs would absorb
  most reloads).

### 7. Smaller observations

- `sp_loads_same_reg_noop` (in-block reload of a slot already in the same
  register): ~10 K instances on glutton — the register cache misses these
  around wasm-memory stores and call-arg staging; worth a targeted look at
  `clear_reg_cache` call sites in `calls.rs` (the cache is cleared for
  *all* calls including the arg-staging region before the jump).
- `LoadImm64` census (glutton): 0xFFFFFFFF (2,163 — the zext mask, see #3),
  0xFEFD0000 (1,630, ~1/function), 0x400000000, 0x80000000. A 4–8 KB
  ro-data constant pool + absolute `LoadU64` would convert most 10-byte
  encodings to ~5 bytes at equal gas, but #3/#5 eat most of this first.
- Peephole pattern 3 (`AddImm32 x,x,0` after 32-bit producer) and
  `optimize_immediate_chains` don't check label boundaries the way
  `optimize_store_then_load` does — they appear sound only because both
  instructions of each pattern are emitted adjacently by one lowering; worth
  an explicit label guard for robustness.

## Recommended order of attack

1. **Fix the PVM-in-PVM miscompile** (`experiments/repro-pip-miscompile/`) —
   correctness, and it blocks everything that shifts bytes.
2. **Land DCE ungating** (`experiments/dce-ungate.patch`) — ~2% size on
   runtimes + hot-path gas, change is ready.
3. **Branch-offset relaxation** — ~4.3% size on every module, no gas risk.
4. **Constant-address absolute folding** (#4) + **LoadImm base tracking** —
   small, contained peephole extensions with measured hot-loop wins.
5. **zext/sext elision at lowering** (#3) — biggest gas item after spills;
   needs the semantics flag discussion for the address-context variant.
6. **Cross-block constant cache** (#5), **phi-copy coalescing** (#6a).
7. **Regalloc expansion** (#6) — the structural 40%; separate design doc.

---

# Implemented results (2026-06-12, follow-up session)

Four fixes implemented on branches (commits unsigned — re-sign when upstreaming):

| Branch | Contents |
|---|---|
| `exp/dce-ungate` | DCE ungating + cmov-liveness + keep[] restructure |
| `exp/offset-relaxation` | Minimal-length branch/jump offsets, iterative relaxation in `resolve_fixups`, width-stable `LoadImmJump` + `JumpFixed` entry header (pc=5 ABI) |
| `exp/zext-address-elision` | Address-mask elision (`--no-address-mask-elision`, auto-off ≥ 2 GB memory) |
| `exp/abs-address-fold` | (stacked on dce-ungate) constant-base indexed mem ops → absolute forms |
| `exp/combined` | merge of all four |

## JAM size (bytes)

| Input | main | combined | Δ |
|---|---:|---:|---:|
| polkadot runtime | 20,908,730 | 18,749,375 | **−10.3%** |
| glutton runtime | 6,401,998 | 5,794,534 | **−9.5%** |
| anan-as-compiler | 109,259 | 101,624 | −7.0% |

Per-branch (polkadot): dce −2.3%, relaxation −3.3%, zext-elision −3.8%, abs-fold (incl. dce) −2.4%.

## Gas (runnable benchmarks, all results byte-identical to main)

| Benchmark | main | combined | Δ |
|---|---:|---:|---:|
| sha512("abc") | 14,459 | 11,555 | **−20.1%** |
| sha512(240×a) | 43,111 | 34,435 | **−20.1%** |
| blake2b | 14,819 | 13,903 | −6.2% |
| u128-div | 68,031 | 65,030 | −4.4% |
| u128-div-slow | 129,031 | 120,030 | −7.0% |
| u128-mul | 71,031 | 68,030 | −4.2% |
| as-gcd | 163 | 162 | −0.6% |
| fib/is-prime/two-loops/as-fib/as-fact | — | — | ±0 |

Dynamic profile shift (sha512-240a): zext shift-pairs 3,078 → 4 executions,
LoadImm/64 5,424 → 303. Remaining overhead is spill traffic (37.7% of gas) —
the regalloc item.

## Test status per branch

- Rust unit+integration (463), TS layers 1–3 (675), differential (142): **green on every branch**.
- PVM-in-PVM (277): green on `exp/offset-relaxation` and `exp/zext-address-elision`;
  3 fails on `exp/dce-ungate`/`exp/abs-address-fold` (as-simpler-repro) and 1 fail on
  `exp/combined` (as-tests-globals) — all with the latent-interpreter-miscompile
  signature (`experiments/repro-pip-miscompile/`): inner programs natively correct,
  empty pip result, r7==r8. Which inner program flips depends only on byte layout.

## Shipping decision (2026-06-13)

The four optimizations were validated as correct. Because DCE (and the
absolute-fold that builds on it) shift inner-test-program byte layout and
thereby **expose a pre-existing core-lowering miscompile of the anan-as
interpreter** (see `repro-pip-miscompile/README.md` — fully characterized
this session: it reproduces on `main`, no optimization flag fixes it, and the
interpreter *source* runs the affected programs correctly natively), the work
was split:

- **Shipped (PR #260)**: `offset-relaxation` + `zext-address-elision`. Both are
  fully green including PVM-in-PVM. Wins: polkadot runtime JAM ~−7%, plus the
  gas reductions in the table above for the zext pass.
- **Deferred to a follow-up** (`perf/dce-absfold` branch): `dce-ungation` +
  `absolute-address-folding`. These are correct and carry the larger gas wins,
  but are gated on first fixing the pre-existing interpreter miscompile so the
  PVM-in-PVM differential harness stays meaningful rather than being made green
  by masking a real (if pre-existing) bug.
