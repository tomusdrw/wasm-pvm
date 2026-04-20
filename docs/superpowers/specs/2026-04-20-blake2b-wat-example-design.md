# Blake2b hand-crafted WAT example — design

**Status:** approved, pending implementation
**Date:** 2026-04-20
**Scope:** new example program + test coverage

## Goal

Add a hand-crafted WAT implementation of the RFC 7693 blake2b hash function (variable output length 1–64, unkeyed) as a non-trivial example program that compiles through the WASM→PVM pipeline. Validate correctness with standard test vectors and seeded randomized differential testing against `@noble/hashes`.

This serves three purposes:

1. **A real-world example** of a useful algorithm running on PVM (JAM uses blake2b-256).
2. **A stress test** for the compiler — 12-round unrolled-or-looped compression, heavy 64-bit mixing, rotations, and per-block memory I/O exercise many code paths.
3. **A regression fixture** with broad coverage, since a byte-level bug anywhere in the pipeline will alter the hash.

## Non-goals (YAGNI)

- Keyed blake2b
- Parallel blake2bp
- Salt / personalization parameters
- Streaming / incremental API
- Benchmarking blake2b specifically (the generic `./tests/utils/benchmark.sh` covers JAM size + gas)

## Files

| Path | Purpose |
|---|---|
| `tests/fixtures/wat/blake2b.jam.wat` | Hand-crafted WAT module |
| `tests/layer3/blake2b.test.ts` | Unit + differential tests (standalone, not via `defineSuite`) |
| `tests/helpers/run.ts` | Add `runJamBytes(jamFile, args, pc?) -> Uint8Array` that returns full raw result bytes (the existing `runJam` clips to 4 bytes and is unsuitable for hash outputs) |
| `tests/helpers/wasm-runner.ts` | Add `runWasmNativeBytes(binary, argsHex) -> { bytes, trapped, error? }` that returns full raw result bytes from native WASM execution |
| `tests/package.json` | Add `@noble/hashes` as a dev dependency |

Layer 3 (regression / edge cases) — too heavy for layer 1/2 smoke tests, but it is a broad regression fixture.

No new directory structure: existing pattern is flat `.jam.wat` under `tests/fixtures/wat/` + `.test.ts` under `tests/layerN/`.

### Why not `defineSuite()` / the auto-differential path

The existing `defineSuite()` / `defineDifferentialSuite()` contract takes `expected: number` and compares via `runJam`, which clips results to a 4-byte little-endian u32 — unsuitable for 32/64-byte hash outputs. Rather than retrofitting those helpers (which would ripple through ~50 suites), this fixture writes its own test bodies using the new byte-returning runners. The `defineSuite` registry stays focused on its current single-number contract.

## WAT module structure

```wat
(module
  (memory (export "memory") 1)
  (data (i32.const OFF_IV)    "<8 × u64 IV constants, little-endian>")
  (data (i32.const OFF_SIGMA) "<10 × 16 u8 permutation table>")

  ;; G(ia, ib, ic, id, mx, my): mix v[ia..id] with m[mx], m[my]
  (func $g (param i32 i32 i32 i32 i32 i32) ...)

  ;; compress(last: i32): 12 rounds over v[] using m[], then h ^= v_lo ^ v_hi
  (func $compress (param $last i32) ...)

  ;; main(args_ptr, args_len) -> i64: full blake2b driver
  (func (export "main") (param i32 i32) (result i64) ...))
```

### Memory layout (WASM-relative, total ~600 B — fits one 64 KB page)

| Offset | Size | Purpose |
|---|---|---|
| `0x000` | 64 B | output hash buffer |
| `0x040` | 64 B | h[8] state (mutable) |
| `0x080` | 64 B | IV[8] constants (data segment) |
| `0x0C0` | 128 B | v[16] working state |
| `0x140` | 128 B | m[16] current message block |
| `0x1C0` | 160 B | sigma table (10 × 16 bytes, data segment) |
| `0x260` | 16 B | t counter (i64) + last flag scratch |

Offsets are WASM-relative; the PVM harness adds `wasm_memory_base` at runtime.

The round loop selects the active sigma row by `round mod 10`, so sigma is stored as a data segment and indexed rather than unrolled into the G calls. This keeps the WAT small and exercises the compiler's loops+indexed-load handling.

### Algorithm steps (in `main`)

1. Validate `out_len`: read first byte of args, trap via `unreachable` if `== 0` or `> 64`.
2. Initialize `h[0..7] = IV[0..7]`; apply parameter block XOR: `h[0] ^= 0x0101_0000 ^ out_len` (unkeyed, fanout=1, depth=1, no salt/personal).
3. Stream input in 128-byte blocks starting at `args_ptr + 1`:
   - For each non-final full block: load 16 u64 LE into `m[]`, update `t += 128`, call `compress(last=0)`.
   - For the final block (including the empty-input case): zero-pad the remainder of `m`, set `t += remaining_byte_count`, call `compress(last=1)`.
4. Copy `h[0..out_len]` bytes from the h state into the output buffer (byte-level copy to respect little-endian encoding of h words and handle non-multiple-of-8 out_len).
5. Return `out_ptr | (out_len << 32)` as i64.

### Compress function

- Load `v[0..7] = h[0..7]`, `v[8..15] = IV[0..7]`.
- XOR the counter `t` (a 128-bit value, we use a single `t_low` i64 and treat `t_high = 0`) into `v[12]` (low) and `v[13]` (high). The WAT still XORs `t_high = 0` into `v[13]` for structural correctness — the 2 KB input cap means `t_high` is never actually nonzero, but the XOR is emitted.
- If `last != 0`: XOR `v[14] ^= 0xFFFF_FFFF_FFFF_FFFF`.
- 12 rounds; each round:
  - `s = sigma[round mod 10]`
  - 8 calls to G: column mix (4 × G) then diagonal mix (4 × G), using `s[0..15]` as message indices per the spec.
- `h[i] ^= v[i] ^ v[i+8]` for `i = 0..7`.

### G function

Standard blake2b G, 64-bit word mixing with rotation constants (32, 24, 16, 63):

```text
v[a] = v[a] + v[b] + m[x]
v[d] = rotr(v[d] ^ v[a], 32)
v[c] = v[c] + v[d]
v[b] = rotr(v[b] ^ v[c], 24)
v[a] = v[a] + v[b] + m[y]
v[d] = rotr(v[d] ^ v[a], 16)
v[c] = v[c] + v[d]
v[b] = rotr(v[b] ^ v[c], 63)
```

Indices `a, b, c, d, x, y` are passed as i32 params; operations use `i64` throughout. WAT `i64.rotr` compiles directly.

## ABI

- Entry: `main(args_ptr: i32, args_len: i32) -> i64` (standard PVM SPI).
- Args: `[out_len: u8][input: bytes]`.
- Return: `(out_ptr: i32) | ((out_len: i32) << 32)`.
- Invalid `out_len` (0 or > 64): WAT `unreachable` → PVM trap. The reference wrapper throws in the same conditions so differential tests observe matching failures.

## Test strategy

### Unit tests (always run)

- **RFC 7693 canonical**: `blake2b("abc", 64)` — the RFC's worked example — hardcoded from the spec as a check that the reference wrapper and the WAT agree with the standard.
- **JAM-relevant**: `blake2b("", 32)`, `blake2b("abc", 32)` — computed via `@noble/hashes` at test time.
- **Size-edge vectors**: input lengths **0, 1, 127, 128, 129, 255, 256, 257** at `out_len=32`, each computed via the reference at test time. These exercise:
  - empty input (one all-zero block, `last=1` on the first/only call)
  - strictly-less-than-one-block
  - exactly-one-block (boundary on the last-block detection)
  - just-over-one-block (first block full, second block 1 byte)
  - approaching / crossing the second and third block boundaries
- **Output-length endpoints**: one test at `out_len=1`, one at `out_len=64`, over a fixed small input.

All unit-test inputs use fixed deterministic bytes (e.g. a repeating pattern or `crypto.createHash("sha256")` seed expanded) so failures are reproducible.

### Differential test (one test, N iterations)

- **Seeded PRNG**: seed from `BLAKE2B_RANDOM_SEED` env var, defaulting to a fixed constant for reproducibility. A small xorshift / splitmix PRNG in the test file avoids a new dependency.
- **Iteration count** from `BLAKE2B_RANDOM_COUNT`, default **50**.
- **Per iteration**: random `out_len ∈ [1, 64]`, random input length `∈ [0, 2048]`, random input bytes.
- **Check**: uses the same three-way agreement harness described below — PVM (`runJamBytes`), native WASM (`runWasmNativeBytes`), and `@noble/hashes` reference must all agree byte-for-byte.
- **Failure output**: print `seed`, `iteration`, `out_len`, `input_hex` so the failing case is a one-liner to re-run locally.
- **Input size cap of 2 KB**: args travel as a hex string on the CLI (4 KB hex). `ARG_MAX` is MB-scale on macOS/Linux, but 2 KB is a comfortable bound.

### Three-way agreement check

Every test case (unit and random) compares **three** results byte-for-byte:

1. **PVM** — `runJamBytes(blake2bJam, argsHex)` → raw result bytes
2. **Native WASM** — `runWasmNativeBytes(watCompiledToWasm, argsHex)` → raw result bytes
3. **Reference** — `@noble/hashes/blake2b(input, { dkLen: out_len })`

All three must agree. This gives three distinct signals from the same test:

- **PVM == reference**: end-to-end correctness of the compiler + runtime
- **Native WASM == reference**: the hand-crafted WAT is itself a correct blake2b (independent of PVM)
- **PVM == native WASM**: the WASM→PVM compilation preserved semantics (this is the signal the existing `defineDifferentialSuite` provides, but for bytes instead of a u32)

Helper function: `assertBlake2b(args: { outLen, input }, expected?: Uint8Array)` runs all three and asserts pairwise equality. Unit tests with hardcoded RFC vectors pass `expected`; differential iterations omit it and just check the three agree with each other plus the reference.

## Gas budget

12 rounds × 8 G calls × ~18 64-bit ops per G ≈ ~1700 ops per compression, plus per-round sigma loads and per-block message loads. For a 2 KB input (~16 blocks) that's on the order of low hundreds of thousands of PVM instructions.

Plan: start with the default 100M gas hardcoded in `runJam` / `runJamBytes` (see `tests/helpers/run.ts:22` — `--gas=100000000`). If blake2b tests exceed it, extend the new `runJamBytes` (and only that helper — the existing `runJam` stays unchanged) with an optional `gas?: number` parameter. Do not alter the global default for the whole suite.

Measured gas will be captured in the PR description's benchmark table.

## Risks

1. **Hand-crafted WAT is error-prone.** Blake2b has a 10-permutation sigma, four rotation constants (32, 24, 16, 63), and little-endian word loading for input bytes. The seeded differential test plus size-edge vectors are the primary safety net; the RFC-canonical vector is the "both sides agree with the spec" backstop.
2. **Byte-endian boundary on output.** For `out_len` not a multiple of 8, the final partial word must be written in little-endian byte order. Covered by the `out_len=1` unit test and by random-length differential runs.
3. **Gas tuning** per test may be needed; if so, keep it per-call, not global.
4. **`@noble/hashes` API drift.** Minor risk; the library is stable and widely used. Pin a specific version in `package.json`.

## Documentation updates on merge

- `AGENTS.md` — add a one-line reference to the blake2b fixture in the "Where to Look" table (pattern: "Add hash-like byte-processing example" → `tests/fixtures/wat/blake2b.jam.wat` + `tests/layer3/blake2b.test.ts`). Call out the new `runJamBytes` / `runWasmNativeBytes` helpers in the same table.
- `docs/src/learnings.md` — record any PVM-compile surprises discovered while hand-crafting blake2b (likely candidates: rotation lowering, 12-round loop code size, sigma indexing).
- **PR description** — benchmark comparison table via `./tests/utils/benchmark.sh --base main --current td-blake2b-wat`.

## Success criteria

- `tests/fixtures/wat/blake2b.jam.wat` compiles to a valid JAM file via the existing pipeline.
- All unit tests pass (RFC vector + JAM-relevant + size edges + out_len endpoints).
- The differential test passes at the default iteration count (50) with the default seed.
- The differential test passes at `BLAKE2B_RANDOM_COUNT=1000` for at least one local run (sanity-check before merge).
- `bun run test` passes on the branch with no regressions elsewhere.
- PR description includes the benchmark comparison table.
