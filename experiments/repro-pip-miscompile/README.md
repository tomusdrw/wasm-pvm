# PVM-in-PVM miscompile reproducer (pre-existing bug on main)

**Found**: 2026-06-12, during the gas/size optimization experimentation session.
**Refined**: 2026-06-13 (PR #260) — see "Refined root cause" below.

## Refined root cause (2026-06-13)

Definitively localized with a reliable oracle (`bun` calling the test helper
`runJamPvmInPvm` directly — NOT a hand-rolled `node index.js run`, which is
flaky under memory pressure and gives false PASS/FAIL):

1. The bug is in **wasm-pvm's always-on core lowering of the interpreter**.
   A 2×2 matrix (inner program ±DCE × interpreter ±DCE) showed the failure
   tracks the *inner program's byte layout*, and the `main`-built interpreter
   mis-runs the DCE-layout inner program too → pre-existing, not from this
   PR's passes.
2. A per-`--no-*`-flag sweep of the interpreter compilation fixes nothing →
   the miscompile is in core lowering, not any optional optimization.
3. The interpreter *source* (`compiler.wasm`) run natively (Bun WebAssembly)
   produces the correct result (17) for both the DCE and non-DCE inner
   programs → the inner programs are valid and the optimizations are correct.

Conclusion: this PR's optimizations are correct; DCE merely shifts inner-test
byte layout and exposes the pre-existing interpreter miscompile. The specific
mis-lowered core construct was not pinned down (22k-instruction interpreter,
data-dependent execution divergence makes trace-diffing inconclusive). DCE +
absolute-fold are therefore deferred (branch `perf/dce-absfold`) until this
core bug is fixed; offset-relaxation + zext-elision shipped in PR #260.

Newer minimal trigger pair (current compiler): `as-tests-globals.jam` built
with vs without DCE (`inner.main` passes through the interpreter, `inner.b049`
— one DCE pass — fails with empty result).

## Summary

`wasm-pvm` miscompiles `vendor/anan-as/dist/build/compiler.wasm` (the PVM
interpreter used for PVM-in-PVM tests) in an **input-data-dependent** way:
the compiled interpreter mis-executes certain inner programs that the same
WASM module executes correctly when run natively.

- `repro-17.jam` / `repro-18.jam` are two compilations of
  `tests/build/wasm/simpler-repro.wasm`. They are semantically identical:
  18 differs from 17 only by the removal of one provably-dead instruction
  (`AddImm64 r7 <- r2+12`, overwritten 2 instructions later with no
  intervening read, no labels/branch targets in between) plus the resulting
  byte-offset shifts.
- Both produce identical, correct results (0/1/2 for args 0500/0501/0502):
  - under the native anan-as engine (`vendor/anan-as/build/release.js`,
    `prepareProgram`+`runProgram`, both preallocate=0 and 128, block gas on),
  - under `vendor/anan-as/dist/bin/index.js run --spi`,
  - and — decisively — under **compiler.wasm itself executed natively** via
    Bun WebAssembly (gas left differs by exactly 1, as expected).
- Under the **wasm-pvm-compiled interpreter** (`anan-as-compiler.jam`),
  `repro-17` works but `repro-18` mis-executes: the inner run takes a
  divergent path near the end and main()'s result comes out with
  `totalLen == 0` (empty result span, r7 == r8).

## What it is NOT

- Not caused by any optimization flag: reproduced with the interpreter
  compiled with ALL `--no-*` flags (register cache, DSE, regalloc, lazy
  spill, peephole, cross-block cache, scratch/caller-saved/aggressive
  regalloc, icmp fusion, const prop, fallthrough jumps, shrink wrap,
  inline, mergefunc, libcall recognition) → **core lowering bug**.
- Not an anan-as bug: compiler.wasm executes both inputs correctly natively.
- Not a malformed inner jam: anan-as `disassemble()` parses both fine; all
  jump-table entries and branch targets land on valid basic-block starts in
  both files.
- Not buffer-length dependence alone: padding repro-17's metadata by 3 bytes
  (same total length as the original) still passes.

## Repro commands

```bash
# build args blob (inner jam + args 0500) — see tests/helpers/pvm-in-pvm.ts buildCompilerArgs
cd tests
bun -e 'import {buildCompilerArgs} from "./helpers/pvm-in-pvm";
  require("fs").writeFileSync("/tmp/pipargs.bin",
  Buffer.from(buildCompilerArgs("../experiments/repro-pip-miscompile/repro-18.jam","0500"),"hex"))'
node ../vendor/anan-as/dist/bin/index.js run --spi --no-logs --gas=10000000000 \
  build/jam/anan-as-compiler.jam /tmp/pipargs.bin
# -> Result: [0x]  (BUG; repro-17.jam in the same harness returns 21 bytes)

# native cross-check (both correct):
bun ../experiments/repro-pip-miscompile/native-compiler.ts
```

## Leads

- The divergence consumes ~97% of the passing run's outer gas before going
  wrong → the inner program mis-executes near its END (the result-returning
  function), not at parse time.
- The inner final r7/r8 from the interpreter imply main's packed result had
  garbage in the high (len) word, or readResultData length went negative
  (totalLen = 17 + (r8-r7) = 0 → r8-r7 = -17).
- Suspect areas (input-byte/position dependent): mask/skip-distance handling
  in the interpreter's hot decode path interacting with a mis-lowered
  construct; `memory.copy` word-loop tails (3105 vs 3108 byte code blobs:
  tails 1 vs 4 mod 8); unaligned multi-byte loads in unchecked AS.

## Why this matters

Any change to the compiler that shifts emitted bytes of *inner* test
programs (e.g. the DCE ungating prototype from this session) can flip
layer4/5 PVM-in-PVM tests between pass/fail without any real regression —
and conversely, the suite currently passes only because the specific byte
layouts happen to avoid the bug.
