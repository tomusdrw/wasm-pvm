# Saturating-arithmetic intrinsic lowering — design

**Issue:** [#217](https://github.com/tomusdrw/wasm-pvm/issues/217) — lower `llvm.{u,s}{add,sub}.sat.iN` in the LLVM backend.

**Goal:** unblock `wasm-pvm compile --trap-floats` against polkadot-fellows v2.2.2 runtimes (currently fails on 8 of 14 runtimes the moment it sees a saturating-arithmetic intrinsic). These show up from Rust's `u32::saturating_sub` family, used heavily in fee/weight code.

**Scope:** all four scalar variants × all four widths — `{uadd, usub, sadd, ssub}.sat` × `{i8, i16, i32, i64}`. SIMD vector forms are out of scope.

---

## 1. Where the code lives

`crates/wasm-pvm/src/llvm_backend/intrinsics.rs`, in `lower_llvm_intrinsic`. Dispatched after the existing `bitreverse`/`bswap`/`abs` blocks, before the final `unsupported LLVM intrinsic` error.

Four guards:

```rust
if name.contains("uadd.sat") { return lower_uadd_sat(e, instr); }
if name.contains("usub.sat") { return lower_usub_sat(e, instr); }
if name.contains("sadd.sat") { return lower_sadd_sat(e, instr); }
if name.contains("ssub.sat") { return lower_ssub_sat(e, instr); }
```

Each helper dispatches on `operand_bit_width(instr)` (8/16/32/64) and returns `Error::Unsupported` for unrecognized widths (matches the bitreverse pattern).

Rationale for four helpers rather than one mega-function: the four algorithms differ enough (clamp vs. wrap-detection vs. mask-then-MinU) that interleaving them obscures each one. The narrow signed paths *are* uniform across `sadd`/`ssub`, so they share a small inner helper that takes the `Add64`/`Sub64` opcode + the per-width sign-extend instruction + the `imin`/`imax` constants. Same for the i64 signed paths, which share their tail (XOR-pair feeds the overflow detection; the rest is identical).

---

## 2. Result form

Per discussion: signed intrinsics produce sign-extended results in their 64-bit register (matches PVM's `Add32`/`Sub32` convention), unsigned produce zero-extended (matches what `LoadIndU32` etc. produce, and what subsequent unsigned operations expect).

This shapes the algorithm choice for each width — the chosen sequences naturally end with the value in the correct form, no extra mask step.

---

## 3. Algorithms

### Register conventions

- `TEMP1`, `TEMP2`, `TEMP_RESULT` — short-lived scratch, no spill bookkeeping.
- `SCRATCH1`, `SCRATCH2` — long-lived scratch; used **only** by the i64 signed paths. Those paths bracket the sequence with `e.spill_allocated_regs()` / `e.reload_allocated_regs_after_scratch_clobber()` (mirroring the existing non-rotation `fshl`/`fshr` lowering).

`dst` is `result_reg(e, instr)` — store-side coalescing writes directly into the allocated register when one is assigned, `TEMP_RESULT` otherwise. `store_to_slot(slot, dst)` at the end.

Operand load uses `operand_reg(e, val, TEMP1)` / `load_operand` (load-side coalescing pattern, identical to bitreverse).

### `usub.sat` — `result = (a < b) ? 0 : a - b`

| Width | Sequence | Count |
|---|---|---|
| i8  | `AndImm t1, a, 0xFF` ; `AndImm t2, b, 0xFF` ; `SetLtU cond, t1, t2` ; `Sub64 dst, t1, t2` ; `CmovNzImm dst, cond, 0` | 5 |
| i16 | same shape, mask `0xFFFF` | 5 |
| i32 | `Shl64+Shr64` ×2 (zero-extend, since `0xFFFFFFFF` doesn't fit `AndImm`) ; `SetLtU` ; `Sub64` ; `CmovNzImm dst, cond, 0` | 7 |
| i64 | `SetLtU cond, a, b` ; `Sub64 dst, a, b` ; `CmovNzImm dst, cond, 0` | 3 |

Result is naturally zero-extended: when `cond=0`, `t1 ≥ t2` and the subtract result fits in the width's positive range; when `cond=1`, the value is replaced by literal 0.

### `uadd.sat`

Narrow widths use the "zero-extend, 64-bit add (cannot overflow), MinU with width's UMAX" trick. i64 uses standard wrap-detection.

| Width | Sequence | Count |
|---|---|---|
| i8  | `AndImm` ×2 (`0xFF`) ; `Add64` ; `LoadImm umax, 255` ; `MinU dst, sum, umax` | 5 |
| i16 | same with `0xFFFF` / `65535` | 5 |
| i32 | `Shl64+Shr64` ×2 (zero-ext) ; `Add64` ; `LoadImm umax,-1; ShloRImm64 umax, umax, 32` (= `0xFFFFFFFF`) ; `MinU` | 8 |
| i64 | `Add64 sum, a, b` ; `SetLtU cond, sum, a` (overflow iff `sum < a` unsigned) ; `LoadImm umax, -1` (= `UINT64_MAX`) ; `CmovNz sum, umax, cond` | 4 |

### `ssub.sat`

Narrow widths sign-extend, do a 64-bit subtract (the true result fits in i64 because two i32 differ by at most 2³³), and clamp via signed `Max`/`Min`. i64 uses Hacker's Delight overflow detection.

| Width | Sequence | Count |
|---|---|---|
| i8  | `SignExtend8` ×2 ; `Sub64` ; `LoadImm imin, -128` ; `Max sum, sum, imin` ; `LoadImm imax, 127` ; `Min sum, sum, imax` | 7 |
| i16 | `SignExtend16` ×2 ; same tail with `-32768` / `32767` | 7 |
| i32 | `AddImm32 _, _, 0` ×2 (sign-extend idiom) ; same tail with `INT32_MIN` / `INT32_MAX` | 7 |
| i64 | see below | 10 |

`ssub.sat.i64` (Hacker's Delight, *uses* SCRATCH registers — wrap with `spill_allocated_regs` / `reload_allocated_regs_after_scratch_clobber`):

```text
Sub64       sum, a, b                   ; wrapping sub
Xor         t1, a, b                    ; signs differ?
Xor         t2, a, sum                  ; sign(a) != sign(sum)?
And         t1, t1, t2                  ; bit 63 = overflow
SharRImm64  cond, t1, 63                ; 0 or -1
SharRImm64  sgn,  a,  63                ; sign of a: 0 or -1
LoadImm     imax, -1                    ; all 1s
ShloRImm64  imax, imax, 1               ; INT64_MAX = 0x7FFF...
Xor         sat,  sgn, imax             ; INT_MAX (a≥0) or INT_MIN (a<0)
CmovNz      sum,  sat, cond             ; if cond, sum = sat
```

### `sadd.sat`

Same shape as `ssub.sat`, only differences are `Add64` vs `Sub64` and the i64 overflow test (`(a^sum) & (b^sum)` instead of `(a^b) & (a^sum)` — both isolate "same-sign-input + different-sign-output" overflow correctly).

Narrow paths: 7 instructions each. i64 path: 10 instructions, identical structure to `ssub.sat.i64` with the XOR pair swapped.

### Why these algorithm splits

- **Narrow unsigned use mask + MinU**: `MinU` with a u32/u16/u8 max immediate is the one-instruction saturation we already have. Once both operands are zero-extended, the 64-bit add cannot overflow, so the only constraint is "result ≤ UMAX".
- **i64 unsigned uses wrap-detection**: there's no wider register, so we detect overflow via `sum < a` (unsigned). `CmovNz` with a register saturates without needing an immediate.
- **Narrow signed use clamp**: PVM has signed `Min`/`Max` directly. Sign-extending narrow operands gives a true 64-bit signed difference/sum that fits in i64; clamping to `[INT_MIN, INT_MAX]` is two instructions.
- **i64 signed needs the XOR overflow trick**: same reason as unsigned (no wider register), and there's no signed equivalent of the unsigned `sum < a` test (signed addition's overflow depends on operand signs, not result magnitude). Hacker's Delight is the standard branchless form.

---

## 4. Operand handling and existing patterns

The lowering follows the bitreverse template:

1. `let val_a = get_operand(instr, 0)?; let val_b = get_operand(instr, 1)?;`
2. `let dst = result_reg(e, instr);`
3. Load both operands into `TEMP1`/`TEMP2` via `operand_reg` + conditional `load_operand`. Apply the standard "if `operand_reg == dst` then fall back to TEMP" guard so we don't clobber `dst` mid-sequence.
4. Emit the per-width instruction sequence.
5. `e.store_to_slot(slot, dst)`.

The two i64 signed helpers additionally bracket steps 3-4 with `e.spill_allocated_regs()` and `e.reload_allocated_regs_after_scratch_clobber()`, since they touch `SCRATCH1`/`SCRATCH2`.

---

## 5. Test coverage

### Layer 3 fixture: `tests/fixtures/wat/saturating_arith.jam.wat`

Args layout:

```text
byte 0..3   : op selector (u32 LE) — encodes (intrinsic, width):
              op = intrinsic_idx * 4 + width_idx
              intrinsic_idx: 0=uadd, 1=usub, 2=sadd, 3=ssub
              width_idx:     0=i8,   1=i16,  2=i32,  3=i64
              ⇒ 16 distinct ops (0..15)
byte 4..11  : operand a (low N bytes used; remainder ignored)
byte 12..19 : operand b
```

Output: written at address 0 with width-specific length (1/2/4/8 bytes), returned as packed `(ptr=0) | (len << 32)` (the standard unified entry ABI).

**WAT body** for each `(intrinsic, width)`: compute the result using the canonical pattern that LLVM's `instcombine` recognises:

- `usub.sat(a,b)` → `select (i32.gt_u a b) (i32.sub a b) (i32.const 0)`
- `uadd.sat(a,b)` → `let s = a+b in select (i32.lt_u s a) UMAX s`
- `ssub.sat(a,b)` → standard signed-overflow form
- `sadd.sat(a,b)` → standard signed-overflow form

These are the same shapes Rust generates from `u32::saturating_sub` / `i32::saturating_add` etc., which is exactly the source of the polkadot-runtime IR that hits the unsupported-intrinsic error today.

### Layer 3 test: `tests/layer3/saturating_arith.test.ts`

For each `(intrinsic, width)`, ~8 cases covering:

- both operands zero
- both operands `UMAX` / `INT_MAX` / `INT_MIN`
- the saturating boundary (`UMAX-1 + 5`, `INT_MIN - 1`, etc.)
- a non-saturating mid-range case
- the issue's hand-calculated cases at i32:
  - `usub_sat_i32(5, 10) = 0`, `usub_sat_i32(10, 5) = 5`, `usub_sat_i32(0, 0xFFFFFFFF) = 0`
  - `uadd_sat_i32(0xFFFFFFFE, 5) = 0xFFFFFFFF`, `uadd_sat_i32(1, 1) = 2`
  - `ssub_sat_i32(INT_MIN, 1) = INT_MIN`, `sadd_sat_i32(INT_MAX, 1) = INT_MAX`

Total: ~16 op/width combinations × ~8 cases = ~128 layer3 tests.

### Differential tests

Register the fixture in the differential runner. Bun's WebAssembly engine evaluates the canonical WAT patterns natively, so the differential check validates byte-for-byte parity.

### Lowering-was-actually-exercised check

The closest precedent in this codebase, the bitreverse fixture, trusts `instcombine` to fold its canonical WAT pattern into the expected intrinsic call without explicit verification. To avoid replicating that gap in the new saturating-arithmetic fixture (e.g., if a future LLVM version stops folding one of the patterns), add **one Rust unit test** in `crates/wasm-pvm/tests/saturating_arith_lowering.rs` that:

1. Compiles the `saturating_arith.jam.wat` fixture through the full pipeline.
2. Asserts the resulting LLVM IR (captured at the end of phase 3 optimisation passes) contains calls to `llvm.uadd.sat.*`, `llvm.usub.sat.*`, `llvm.sadd.sat.*`, `llvm.ssub.sat.*` — confirming the new lowering paths are actually traversed.

If LLVM ever stops folding one pattern, this test fails immediately rather than silently exercising the long-form lowering.

---

## 6. Pre-merge validation

Two pre-merge gates, both required (per the project's `AGENTS.md` PR policy):

### 6a. Polkadot smoke check

Manually run

```bash
cargo run -p wasm-pvm-cli -- compile <some_runtime>.wasm --trap-floats -o /tmp/out.jam
```

against one of the 8 affected polkadot-fellows v2.2.2 runtimes (e.g. `polkadot_runtime`) to confirm the `Unsupported WASM feature: unsupported LLVM intrinsic: llvm.usub.sat.i32` error no longer fires. Not a CI test — a manual gate before merging.

### 6b. Benchmark

Run

```bash
./tests/utils/benchmark.sh --base main --current <branch>
```

and paste the resulting comparison tables (JAM file size, gas usage, execution time — both direct and PVM-in-PVM) into the PR description. Per `AGENTS.md`: "Every PR description MUST include benchmark results. PRs without benchmark results should not be merged."

---

## 7. Documentation updates

- `AGENTS.md` "Where to Look" table — add a row pointing at `lower_{u,s}{add,sub}_sat` in `intrinsics.rs`.
- `docs/src/learnings.md` — new subsection "Saturating-arithmetic lowering" covering the per-width algorithm split and the rationale (narrow widths can clamp/MinU because operands fit in i64; i64 needs wrap-detection / Hacker's Delight).

No new entry in `docs/src/optimizations.md` — this is a feature gate, not an optimization toggle.

---

## 8. Out of scope

- No new PVM instruction or peephole pattern.
- No LLVM-IR-level rewrite — purely a backend lowering.
- SIMD widths (`<N x i32>` etc.) — issue is scoped to scalar; substrate IR doesn't surface vector saturating intrinsics.
- A new `OptimizationFlags` toggle — saturating lowering is a correctness gate, not an optimization.

---

## 9. Risk / open questions

- **`instcombine` pattern recognition on WAT**: the test fixture's correctness as a *coverage* test depends on LLVM folding the canonical WAT shape into the intrinsic call. The lowering-was-exercised Rust unit test (§5) gates this — if it fails on a future LLVM bump, we'd need to either rewrite the WAT closer to LLVM's expected shape or generate the fixture from Rust source.
- **Fixture size**: 16 op variants in one WAT module is medium-sized but well within what the existing pipeline handles (the bitreverse fixture has 4 variants in a similar shape).
- **i64 signed paths use SCRATCH registers**: this is the same compromise made for non-rotation `fshl`/`fshr`. The spill/reload bracket adds overhead but is unavoidable without burning more TEMP registers; the alternative would be a longer sequence using only TEMPs.
