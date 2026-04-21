# SHA-512 hand-crafted WAT example — design

**Status:** implemented (see `tests/fixtures/wat/sha512.jam.wat`, PR #199)
**Date:** 2026-04-20
**Scope:** new example program + test coverage (PR A in a two-PR stack; PR B will be ed25519 verify reusing the SHA-512 implementation internally).

## Implementation deltas vs this spec

The landed WAT diverges from this design in two places; both decisions were made during implementation to address load/phi-pressure issues discovered only at build-and-test time:

1. **Memory-backed round state** — the round function keeps `a..h` in memory at `0x640..0x67F`, not purely in WASM locals. Keeping 8 loop-carried i64 locals across 80 iterations produced phi-header pressure that degraded the round loop; the memory-backed form was cheaper overall. `$a` and `$e` are still `local`s used as temporaries within each round, but the state array is read from / written to memory. See the round function section below.
2. **32 KB input cap (not 64 KB)** — `posix_spawn` / Linux `MAX_ARG_STRLEN` is 128 KB per argv string, and the hex encoding of a 64 KB input overflows it. Cap is 32 KB and enforced in the WAT (`args_len > 32768` returns `(ptr=0, len=0)`).

## Goal

Add a hand-crafted WAT implementation of SHA-512 (FIPS 180-4, fixed 64-byte output) as a non-trivial example program that compiles through the WASM→PVM pipeline. Validate correctness with standard NIST vectors and seeded randomized differential testing against `@noble/hashes/sha2`.

Three purposes, same as blake2b:

1. **A real-world example** — SHA-512 is used as the inner hash of ed25519, so this is a direct stepping-stone to PR B.
2. **A compiler stress test** — 80-round compression, 64-bit big-endian word loads (bswap), two-path padding, and per-block indexed message-schedule access exercise many code paths.
3. **A regression fixture** — any byte-level bug anywhere in the pipeline alters the hash, so broad coverage comes for free from a small number of tests.

## Non-goals (YAGNI)

- SHA-512/224, SHA-512/256, SHA-384. Ed25519 uses SHA-512 proper.
- Keyed / HMAC modes.
- Streaming / incremental API.
- A standalone SHA-512 benchmark. The generic `./tests/utils/benchmark.sh` already covers JAM size + gas.

## Files

| Path | Purpose |
|---|---|
| `tests/fixtures/wat/sha512.jam.wat` | Hand-crafted WAT module |
| `tests/layer3/sha512.test.ts` | Unit + differential tests (standalone, same pattern as `blake2b.test.ts`) |
| `tests/helpers/sha512-ref.ts` | `@noble/hashes/sha2` wrapper + `encodeSha512Args` |

No new runtime helpers. `runJamBytes` and `runWasmNativeBytes` already exist from the blake2b PR.

Layer 3 (regression / edge cases) — same rationale as blake2b: too heavy for layer 1/2 smoke, but it is a broad regression fixture.

### Why not `defineSuite()` / auto-differential

Same as blake2b: `defineSuite()` clips to a u32 result. SHA-512 is 64-byte output. Reuse the `runJamBytes` / `runWasmNativeBytes` byte-level runners.

## WAT module structure

```wat
(module
  (memory (export "memory") 1)
  (data (i32.const 0x080) "<8 × u64 initial H constants, little-endian>")
  (data (i32.const 0x0c0) "<80 × u64 K constants, little-endian>")

  ;; Byte-swap an i64 (LE ↔ BE).
  (func $bswap64 (param i64) (result i64) ...)

  ;; compress(block_ptr: i32): consumes one 128-byte block at block_ptr, mutates h[].
  (func $compress (param $block_ptr i32) ...)

  ;; main(args_ptr, args_len) -> i64
  (func (export "main") (param i32 i32) (result i64) ...))
```

**Rationale for LE-stored constants**: WAT data segments are bytes, and `i64.load` is LE. Storing `H[0] = 0x6a09e667f3bcc908` in LE order on disk makes `i64.load` return the correct value directly, no bswap on constant access. The bswap cost is only on input bytes (which arrive in BE from the message).

### Memory layout (WASM-relative, ~37 KB — fits one 64 KB page)

| Offset | Size | Purpose |
|---|---|---|
| `0x000` | 64 B | output hash buffer |
| `0x040` | 64 B | h[8] state (mutable) |
| `0x080` | 64 B | initial H[8] constants (data segment) |
| `0x0C0` | 640 B | K[80] round constants (data segment) |
| `0x340` | 640 B | W[80] message schedule (mutable) |
| `0x5C0` | 128 B | final-block padding buffer (used only on the tail) |
| `0x640` | 64 B | round-function working state a..h (memory-backed — see implementation deltas) |
| `0x1000` | 32 KB | input buffer (args copied here once at entry) |

Offsets are WASM-relative; the PVM harness adds `wasm_memory_base` at runtime.

### Round function (inside `$compress`)

SHA-512 round, standard form:

```text
T1 = h + bsig1(e) + ch(e, f, g) + K[t] + W[t]
T2 = bsig0(a) + maj(a, b, c)
h  = g; g = f; f = e
e  = d + T1
d  = c; c = b; b = a
a  = T1 + T2
```

where `ch`, `maj`, `bsig0`, `bsig1` are the standard 64-bit sigma functions.

The initial design had the round function carry `a..h` as 8 `i64` locals across the 80-iteration loop, relying on SSA phi nodes. The landed implementation instead keeps the state array in memory at `0x640..0x67F` and uses a small number of `i64` locals (primarily `$a`, `$e`, `$t1`, `$t2`) as per-iteration temporaries. Rationale: 8 loop-carried i64 locals across 80 iterations created enough phi-header pressure to offset the savings from avoiding memory traffic. The memory-backed form proved cheaper when measured end-to-end. See `tests/fixtures/wat/sha512.jam.wat` for the canonical round body.

**Message schedule**:

```text
W[t] = load_be64(block + t*8)                      for t ∈ [0, 16)
W[t] = ssig1(W[t-2]) + W[t-7] + ssig0(W[t-15]) + W[t-16]   for t ∈ [16, 80)
```

The two halves are separate loops to keep each body small. `W[]` is stored in memory at offset `0x340` and indexed by `(t * 8)`.

### Padding (inside `main`)

SHA-512 padding is "append 0x80, zero-pad to 112 mod 128, append 128-bit BE bit-length". Two cases:

- **`tail_len ≤ 111`** (~88% of inputs): the padding block is `[tail bytes][0x80][zeros][16-byte BE length]`, one call to `$compress`.
- **`tail_len ≥ 112`** (the other ~12%): the padding spills into a second block.
  - Block 1: `[tail bytes][0x80][zeros to end of block]`.
  - Block 2: `[112 zero bytes][16-byte BE length]`.

This split is ~50 lines of WAT but makes the padding logic obvious and testable. The alternative (always conditionally pad-then-maybe-compress-twice) is more compact but mixes the two paths — more room for a padding bug to hide.

**Input size cap** is 32 KB (vs blake2b's 2 KB), so the bit length fits in a u32; the WAT still writes the full 128-bit length field (most bytes zero) for structural correctness. The cap is set at 32 KB — not the more intuitive 64 KB — because Linux's `posix_spawn` caps a single argv string at 128 KB (`MAX_ARG_STRLEN`); args travel to anan-as as a hex argument (`2 × args_len + 2` chars) and a 64 KB input would overflow the single-string limit. The cap is enforced in the WAT: inputs above it return `(ptr=0, len=0)`.

### Bswap helper

WAT has no `i64.bswap`. The helper:

```text
bswap64(x) =
  (x << 56)
  | ((x & 0x000000000000FF00) << 40)
  | ((x & 0x0000000000FF0000) << 24)
  | ((x & 0x00000000FF000000) <<  8)
  | ((x >> 8)  & 0x00000000FF000000)
  | ((x >> 24) & 0x0000000000FF0000)
  | ((x >> 40) & 0x000000000000FF00)
  |  (x >> 56)
```

Called 16 times per block from the W[0..15] load. Implemented as a real `$bswap64` function (not inlined — keeps the inline-threshold honest; `func_inline_threshold = 5` would skip it anyway).

## ABI

- Entry: `main(args_ptr: i32, args_len: i32) -> i64` (standard PVM SPI).
- Args: `[input: bytes]`. No prefix — output length is fixed at 64.
- Return: `(0: i32) | ((64: i32) << 32)`.
- No traps from the algorithm. Any trap would indicate a compiler bug.

## Test strategy

### Unit tests (always run)

- **FIPS 180-4 canonical vectors**:
  - `sha512("")` = `cf83e135…3e85a538`
  - `sha512("abc")` = `ddaf35a1…a538327af927da3e`
  - Hardcoded bytes from the standard — these are the "everyone agrees with the spec" backstops, independent of `@noble/hashes`.
- **Size-edge vectors** at deterministic pattern bytes:
  - `0, 1, 55, 56, 111, 112, 119, 120, 127, 128, 129` — critical padding boundaries.
    - `111`: last byte of the "fits in one block" window.
    - `112`: first byte where the padding overflows into a second block.
    - `119` / `120` / `127` / `128` / `129`: mid-second-block and two-block transitions.
  - `255, 256, 257` — general block-boundary sanity (matching blake2b's pattern).
- **Boundary checks on cap**:
  - `32767`, `32768` — upper end of the differential range, ensures no unexpected truncation.
- **Cap rejection**:
  - `32769` — verifies the WAT's `args_len > cap` guard returns an empty result via PVM and native WASM.

### Seeded random differential (one test, N iterations)

- **Seeded PRNG**: `SHA512_RANDOM_SEED` env var (hex), default `0123456789abcdef`. Uses the same splitmix64 helper pattern as blake2b.
- **Iteration count** from `SHA512_RANDOM_COUNT`, default **50**.
- **Per iteration**: input length `∈ [0, 32768]`, random bytes.
- **Check**: three-way byte agreement (PVM, native WASM, `@noble/hashes/sha2`).
- **Failure output**: prints `seed`, `iteration`, `inputLen`, `input_hex` (truncated if >4 KB for readability). Re-run reproducible via the env vars.

### Three-way agreement

Identical rationale to blake2b — see `2026-04-20-blake2b-wat-example-design.md`. Harness:

```text
assertSha512Agreement(input: Uint8Array, expected?: Uint8Array)
  → PVM == native WASM == reference, and (if expected) == expected
```

## Gas budget

80 rounds × ~10 64-bit ops per round + bswap overhead + schedule extension. Per-block cost is in the same order of magnitude as one blake2b block (blake2b: 12 rounds × 8 G calls × ~15 ops = ~1440 ops; SHA-512: 80 rounds × ~10 ops = ~800 ops — SHA-512 is lighter per block).

For a 32 KB input (256 blocks) that's roughly 200k PVM instructions per iteration. At 50 iterations plus the unit tests, total budget is ~10M instructions. Well within `runJamBytes`'s default 100M gas.

If tests need more, extend `runJamBytes` with an optional `gas?: number` param (already done for blake2b).

## Risks

1. **Hand-crafted SHA-512 is error-prone.** The padding split, big-endian loads, and 80-round schedule are all places for subtle off-by-one bugs. Mitigations: FIPS vectors as the spec backstop, size-edge tests on every padding-boundary byte in `[108, 132]`, and 50-iteration seeded random differential at 32 KB cap.
2. **Big-endian loads must go through `$bswap64`.** Forgetting a bswap shifts the entire hash output — obvious failure, easy to catch.
3. **Gas tuning** per test may be needed; if so, keep it per-call, not global. (Same policy as blake2b.)
4. **`@noble/hashes/sha2` API drift.** `@noble/hashes` is pinned via `package.json`; path is already stable (blake2b uses `/blake2b`, we use `/sha2`'s `sha512` export).

## Documentation updates on merge

- `AGENTS.md` — add a one-line reference to the SHA-512 fixture in the "Where to Look" table. Call it out alongside blake2b.
- `docs/src/learnings.md` — record any PVM-compile surprises discovered while hand-crafting SHA-512 (likely candidates: 80-round loop code size, 8-local phi-node quality, bswap lowering).
- **PR description** — benchmark comparison table via `./tests/utils/benchmark.sh --base main --current td-sha512-wat`.

## Success criteria

- `tests/fixtures/wat/sha512.jam.wat` compiles to a valid JAM file via the existing pipeline.
- All unit tests pass (FIPS vectors + size edges + cap endpoints).
- The differential test passes at the default iteration count (50) with the default seed.
- The differential test passes at `SHA512_RANDOM_COUNT=1000` for at least one local run before merge.
- `bun run test` passes on the branch with no regressions elsewhere.
- PR description includes the benchmark comparison table.

## Follow-up (out of scope for this PR)

- **PR B** (`td-ed25519-wat`, stacked on this): ed25519 verify, embedding a copy of the SHA-512 code inside the ed25519 WAT. Design spec and plan to be written separately.
- **Issue #197**: raise blake2b's differential input cap from 2 KB to 64 KB to match SHA-512.
