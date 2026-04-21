# SHA-512 WAT example implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Post-implementation note (2026-04-21):** two things landed differently from this plan; follow-up work (e.g. ed25519 in PR B) should use the updated values rather than copying from the embedded code blocks below:
> - **Input cap is 32 KB, not 64 KB.** Linux `MAX_ARG_STRLEN` (128 KB per argv string) rejects a 64 KB hex-encoded input as `E2BIG`, so the cap was lowered to 32 KB end-to-end (WAT, tests, spec). All `65535` / `65536` references below should read `32767` / `32768`; `randInt(next, 0, 65536)` should read `randInt(next, 0, 32768)`. The WAT also explicitly rejects `args_len > 32768`.
> - **Round-function state is memory-backed, not 8 locals.** Keeping 8 loop-carried `i64` locals across 80 iterations created enough phi-header pressure to offset the savings from avoiding memory traffic. The landed WAT keeps the `a..h` state in memory at `0x640..0x67F` and uses only a small number of locals as per-iteration temporaries (`$a`, `$e`, `$t1`, `$t2`). See `tests/fixtures/wat/sha512.jam.wat`.

**Goal:** Add a hand-crafted WAT SHA-512 fixture that compiles through the WASM→PVM pipeline, with byte-level three-way differential testing against `@noble/hashes/sha2` and native WebAssembly. Fixed 64-byte output, input size cap 32 KB (see note above).

**Architecture:** FIPS 180-4 SHA-512. WAT module with `$bswap64`, `$compress`, and `main` functions plus active data segments for the initial H values and 80 K round constants. Round function carries `a..h` state through memory at `0x640..0x67F` (not 8 locals — see note above). Message schedule `W[80]` stored in memory. Padding split into two paths: single-block (`tail_len ≤ 111`) and two-block (`tail_len ≥ 112`).

**Tech Stack:** WAT (hand-crafted), existing Rust WASM→PVM compiler, Bun test runner, TypeScript, `@noble/hashes` reference library (already installed), `wabt` for WAT→WASM (already installed), existing `runJamBytes`/`runWasmNativeBytes` helpers (added in blake2b PR).

**Spec:** `docs/superpowers/specs/2026-04-20-sha512-wat-example-design.md`

---

## File Structure

**Created:**
- `tests/fixtures/wat/sha512.jam.wat` — the hand-crafted WAT module (~300–400 lines)
- `tests/layer3/sha512.test.ts` — unit tests + seeded random differential
- `tests/helpers/sha512-ref.ts` — thin wrapper around `@noble/hashes/sha2.sha512` matching the `[input]` ABI

**Modified:**
- `AGENTS.md` — add SHA-512 fixture to the "Where to Look" table

**No changes needed to**: `run.ts`, `wasm-runner.ts`, `package.json` — all infrastructure added in blake2b PR.

**Responsibilities:**
- `sha512-ref.ts`: ABI-matching reference, computes expected hashes
- `sha512.test.ts`: three-way agreement (PVM == native WASM == reference) per test
- `sha512.jam.wat`: the algorithm

---

## Verification Commands

From the `tests/` directory:

- **Build artifacts:** `bun build.ts` (compiles Rust CLI + all WATs→JAMs + AS→WASM)
- **Force rebuild:** `rm -f build/wasm/*.wasm && bun build.ts` (for stale AS caches; WAT is always rebuilt)
- **Run layer3 only:** `bun run build && bun test layer3/sha512.test.ts`
- **Run full suite:** `bun run test`
- **Run one test by name:** `bun run build && bun test layer3/sha512.test.ts -t "abc"`
- **1000-iteration differential:** `SHA512_RANDOM_COUNT=1000 bun run build && bun test layer3/sha512.test.ts -t "random"`

From the repo root:

- **Type/clippy check:** `cargo check --workspace` (fast; runs as pre-push hook)
- **Benchmark vs main:** `./tests/utils/benchmark.sh --base main --current td-sha512-wat`

---

## Key Constants (hardcode these — they are the algorithm)

**SHA-512 initial hash values** (FIPS 180-4 §5.3.5):
```text
H[0] = 0x6A09E667F3BCC908
H[1] = 0xBB67AE8584CAA73B
H[2] = 0x3C6EF372FE94F82B
H[3] = 0xA54FF53A5F1D36F1
H[4] = 0x510E527FADE682D1
H[5] = 0x9B05688C2B3E6C1F
H[6] = 0x1F83D9ABFB41BD6B
H[7] = 0x5BE0CD19137E2179
```

**SHA-512 round constants K[0..79]** (FIPS 180-4 §4.2.3, first 64 bits of fractional parts of cube roots of first 80 primes):
```text
K[ 0] = 0x428A2F98D728AE22  K[ 1] = 0x7137449123EF65CD  K[ 2] = 0xB5C0FBCFEC4D3B2F  K[ 3] = 0xE9B5DBA58189DBBC
K[ 4] = 0x3956C25BF348B538  K[ 5] = 0x59F111F1B605D019  K[ 6] = 0x923F82A4AF194F9B  K[ 7] = 0xAB1C5ED5DA6D8118
K[ 8] = 0xD807AA98A3030242  K[ 9] = 0x12835B0145706FBE  K[10] = 0x243185BE4EE4B28C  K[11] = 0x550C7DC3D5FFB4E2
K[12] = 0x72BE5D74F27B896F  K[13] = 0x80DEB1FE3B1696B1  K[14] = 0x9BDC06A725C71235  K[15] = 0xC19BF174CF692694
K[16] = 0xE49B69C19EF14AD2  K[17] = 0xEFBE4786384F25E3  K[18] = 0x0FC19DC68B8CD5B5  K[19] = 0x240CA1CC77AC9C65
K[20] = 0x2DE92C6F592B0275  K[21] = 0x4A7484AA6EA6E483  K[22] = 0x5CB0A9DCBD41FBD4  K[23] = 0x76F988DA831153B5
K[24] = 0x983E5152EE66DFAB  K[25] = 0xA831C66D2DB43210  K[26] = 0xB00327C898FB213F  K[27] = 0xBF597FC7BEEF0EE4
K[28] = 0xC6E00BF33DA88FC2  K[29] = 0xD5A79147930AA725  K[30] = 0x06CA6351E003826F  K[31] = 0x142929670A0E6E70
K[32] = 0x27B70A8546D22FFC  K[33] = 0x2E1B21385C26C926  K[34] = 0x4D2C6DFC5AC42AED  K[35] = 0x53380D139D95B3DF
K[36] = 0x650A73548BAF63DE  K[37] = 0x766A0ABB3C77B2A8  K[38] = 0x81C2C92E47EDAEE6  K[39] = 0x92722C851482353B
K[40] = 0xA2BFE8A14CF10364  K[41] = 0xA81A664BBC423001  K[42] = 0xC24B8B70D0F89791  K[43] = 0xC76C51A30654BE30
K[44] = 0xD192E819D6EF5218  K[45] = 0xD69906245565A910  K[46] = 0xF40E35855771202A  K[47] = 0x106AA07032BBD1B8
K[48] = 0x19A4C116B8D2D0C8  K[49] = 0x1E376C085141AB53  K[50] = 0x2748774CDF8EEB99  K[51] = 0x34B0BCB5E19B48A8
K[52] = 0x391C0CB3C5C95A63  K[53] = 0x4ED8AA4AE3418ACB  K[54] = 0x5B9CCA4F7763E373  K[55] = 0x682E6FF3D6B2B8A3
K[56] = 0x748F82EE5DEFB2FC  K[57] = 0x78A5636F43172F60  K[58] = 0x84C87814A1F0AB72  K[59] = 0x8CC702081A6439EC
K[60] = 0x90BEFFFA23631E28  K[61] = 0xA4506CEBDE82BDE9  K[62] = 0xBEF9A3F7B2C67915  K[63] = 0xC67178F2E372532B
K[64] = 0xCA273ECEEA26619C  K[65] = 0xD186B8C721C0C207  K[66] = 0xEADA7DD6CDE0EB1E  K[67] = 0xF57D4F7FEE6ED178
K[68] = 0x06F067AA72176FBA  K[69] = 0x0A637DC5A2C898A6  K[70] = 0x113F9804BEF90DAE  K[71] = 0x1B710B35131C471B
K[72] = 0x28DB77F523047D84  K[73] = 0x32CAAB7B40C72493  K[74] = 0x3C9EBE0A15C9BEBC  K[75] = 0x431D67C49C100D4C
K[76] = 0x4CC5D4BECB3E42B6  K[77] = 0x597F299CFC657E2A  K[78] = 0x5FCB6FAB3AD6FAEC  K[79] = 0x6C44198C4A475817
```

Each K is stored as LE in the data segment (so `i64.load` returns the correct value directly).

**Memory layout (WASM-relative offsets):**

| Offset | Size | Purpose |
|---|---|---|
| `0x000` | 64 B  | output hash buffer |
| `0x040` | 64 B  | h[8] state (mutable, 8 × i64) |
| `0x080` | 64 B  | initial H[8] constants (data segment) |
| `0x0C0` | 640 B | K[80] round constants (data segment) |
| `0x340` | 640 B | W[80] message schedule (mutable, 80 × i64) |
| `0x5C0` | 128 B | final-block padding buffer |

Total: 1600 B — comfortably within one 64 KB WASM page.

**Round function formulas** (FIPS 180-4 §4.1.3 / §6.4.2):

```text
Ch(x,y,z)   = (x AND y) XOR ((NOT x) AND z)
Maj(x,y,z)  = (x AND y) XOR (x AND z) XOR (y AND z)
BSIG0(x)    = ROTR(x, 28) XOR ROTR(x, 34) XOR ROTR(x, 39)
BSIG1(x)    = ROTR(x, 14) XOR ROTR(x, 18) XOR ROTR(x, 41)
SSIG0(x)    = ROTR(x,  1) XOR ROTR(x,  8) XOR SHR(x,  7)
SSIG1(x)    = ROTR(x, 19) XOR ROTR(x, 61) XOR SHR(x,  6)

Per round t:
  T1 = h + BSIG1(e) + Ch(e,f,g) + K[t] + W[t]
  T2 = BSIG0(a) + Maj(a,b,c)
  h = g; g = f; f = e
  e = d + T1
  d = c; c = b; b = a
  a = T1 + T2

Message schedule:
  W[t] = BSWAP64(load_le64(block + t*8))                      for t ∈ [0, 16)
  W[t] = SSIG1(W[t-2]) + W[t-7] + SSIG0(W[t-15]) + W[t-16]    for t ∈ [16, 80)

After 80 rounds: h[i] = h[i] + (a|b|c|d|e|f|g|h) for i = 0..7
```

---

## Task 1: Add `sha512-ref.ts` reference wrapper

**Why:** The test file needs a tidy `[input: bytes] → Uint8Array` reference that matches the WAT's ABI. `@noble/hashes` is already a dev dep (added in the blake2b PR).

**Files:**
- Create: `tests/helpers/sha512-ref.ts`

- [ ] **Step 1: Write a failing test for the reference wrapper**

Create `tests/helpers/sha512-ref.test.ts`:

```ts
import { test, expect } from "bun:test";
import { sha512Ref, encodeSha512Args } from "./sha512-ref";

test("sha512Ref matches FIPS 180-4 vector for sha512('abc')", () => {
  const hash = sha512Ref({ input: new TextEncoder().encode("abc") });
  // FIPS 180-4 Appendix A.1 worked example.
  const expected = new Uint8Array([
    0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba,
    0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31,
    0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2,
    0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a,
    0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8,
    0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd,
    0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
    0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
  ]);
  expect(hash).toEqual(expected);
});

test("sha512Ref matches FIPS 180-4 vector for sha512('')", () => {
  const hash = sha512Ref({ input: new Uint8Array(0) });
  const expected = new Uint8Array([
    0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd,
    0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d, 0x80, 0x07,
    0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc,
    0x83, 0xf4, 0xa9, 0x21, 0xd3, 0x6c, 0xe9, 0xce,
    0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0,
    0xff, 0x83, 0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f,
    0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81,
    0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e,
  ]);
  expect(hash).toEqual(expected);
});

test("encodeSha512Args just returns the input bytes unchanged", () => {
  const args = encodeSha512Args({ input: new Uint8Array([0xaa, 0xbb]) });
  expect(args).toEqual(new Uint8Array([0xaa, 0xbb]));
});
```

Run: `cd tests && bun test helpers/sha512-ref.test.ts`
Expected: FAIL — module does not exist.

- [ ] **Step 2: Implement the wrapper**

Create `tests/helpers/sha512-ref.ts`:

```ts
import { sha512 } from "@noble/hashes/sha2";

export interface Sha512Args {
  /** Input to hash. */
  input: Uint8Array;
}

/** Reference SHA-512 via `@noble/hashes`. Output is always 64 bytes. */
export function sha512Ref(args: Sha512Args): Uint8Array {
  return sha512(args.input);
}

/**
 * Encode args for the WAT entry-point: args = [input]. No prefix — SHA-512
 * has a fixed 64-byte output, so there is nothing to parameterize.
 */
export function encodeSha512Args(args: Sha512Args): Uint8Array {
  // Return a copy so callers can mutate the input without affecting the args.
  return new Uint8Array(args.input);
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `cd tests && bun test helpers/sha512-ref.test.ts`
Expected: PASS (all three tests). Pure-JS; no `bun run build` needed.

- [ ] **Step 4: Remove the temporary test file**

The reference helper is exercised indirectly through every SHA-512 test. Delete the standalone test:

```bash
rm tests/helpers/sha512-ref.test.ts
```

- [ ] **Step 5: Commit**

```bash
git add tests/helpers/sha512-ref.ts
git commit -m "$(cat <<'EOF'
test: add sha512-ref helper wrapping @noble/hashes

Reference wrapper matching the SHA-512 WAT fixture's [input] ABI.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Write the test file against a stub WAT

**Why:** TDD — write the tests first, run them against a dummy WAT that returns the wrong hash. This isolates test-harness bugs from WAT bugs. All tests fail (correctly) until Task 3.

The stub returns 64 zero bytes. All PVM/WASM assertions will fail (hash mismatch) but the harness plumbing — PRNG, three-way comparison, arg encoding — exercises correctly.

**Files:**
- Create: `tests/fixtures/wat/sha512.jam.wat` (stub)
- Create: `tests/layer3/sha512.test.ts`

- [ ] **Step 1: Create the stub WAT**

Create `tests/fixtures/wat/sha512.jam.wat`:

```wat
;; STUB — will be replaced in Task 3 with the real SHA-512 implementation.
;; Writes 64 zero bytes to offset 0 and returns (ptr=0, len=64).
;; Exists to let the test harness run end-to-end before the algorithm is written.
(module
  (memory (export "memory") 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $i i32)

    ;; Zero the 64-byte output buffer at offset 0..64.
    (local.set $i (i32.const 0))
    (block $exit
      (loop $zero_loop
        (br_if $exit (i32.ge_u (local.get $i) (i32.const 64)))
        (i32.store8 (local.get $i) (i32.const 0))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $zero_loop)))

    ;; Return (ptr=0) | (64 << 32) = 0x00000040_00000000.
    (i64.const 274877906944)))
```

- [ ] **Step 2: Verify the stub compiles to a JAM**

Run: `cd tests && bun build.ts`
Expected: Build succeeds. Verify the JAM is produced:

```bash
ls -l tests/build/jam/sha512.jam
```

If the build fails, fix syntax errors in the WAT and re-run.

- [ ] **Step 3: Write the test file**

Create `tests/layer3/sha512.test.ts`:

```ts
import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR, WAT_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";
import { runWasmNativeBytes, watToWasm } from "../helpers/wasm-runner";
import {
  sha512Ref,
  encodeSha512Args,
  bytesToHex,
  type Sha512Args,
} from "../helpers/sha512-ref";

// -----------------------------------------------------------------------------
// Fixture paths
// -----------------------------------------------------------------------------

const JAM_FILE = path.join(JAM_DIR, "sha512.jam");
const WAT_FILE = path.join(WAT_DIR, "sha512.jam.wat");

// -----------------------------------------------------------------------------
// Three-way agreement: PVM == native WASM == @noble/hashes reference
// -----------------------------------------------------------------------------

async function assertSha512Agreement(args: Sha512Args, expected?: Uint8Array) {
  const argsBytes = encodeSha512Args(args);
  const argsHex = bytesToHex(argsBytes);

  const ref = sha512Ref(args);
  if (expected) {
    expect(bytesToHex(ref)).toBe(bytesToHex(expected));
  }

  const pvm = runJamBytes(JAM_FILE, argsHex);
  expect(bytesToHex(pvm)).toBe(bytesToHex(ref));

  const wasm = await runWasmNativeBytes(await watToWasm(WAT_FILE), argsHex);
  expect(wasm.trapped).toBe(false);
  expect(bytesToHex(wasm.bytes!)).toBe(bytesToHex(ref));
}

// -----------------------------------------------------------------------------
// Deterministic input generator (repeating pattern — not random) for unit tests.
// -----------------------------------------------------------------------------

function patternInput(len: number): Uint8Array {
  const out = new Uint8Array(len);
  for (let i = 0; i < len; i++) out[i] = i & 0xff;
  return out;
}

// -----------------------------------------------------------------------------
// Seeded PRNG: splitmix64 → u8 stream
// -----------------------------------------------------------------------------

function splitmix64(seed: bigint): () => bigint {
  let state = seed;
  return () => {
    state = (state + 0x9e3779b97f4a7c15n) & 0xffffffffffffffffn;
    let z = state;
    z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & 0xffffffffffffffffn;
    z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & 0xffffffffffffffffn;
    return z ^ (z >> 31n);
  };
}

function randomBytes(next: () => bigint, len: number): Uint8Array {
  const out = new Uint8Array(len);
  let i = 0;
  while (i < len) {
    let w = next();
    for (let b = 0; b < 8 && i < len; b++, i++) {
      out[i] = Number(w & 0xffn);
      w >>= 8n;
    }
  }
  return out;
}

function randInt(next: () => bigint, min: number, maxInclusive: number): number {
  const span = BigInt(maxInclusive - min + 1);
  return Number(next() % span) + min;
}

// -----------------------------------------------------------------------------
// Unit tests
// -----------------------------------------------------------------------------

describe("sha512: FIPS 180-4 vectors", () => {
  test("sha512('abc')", async () => {
    const expected = new Uint8Array([
      0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba,
      0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31,
      0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2,
      0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a,
      0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8,
      0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd,
      0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
      0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
    ]);
    await assertSha512Agreement(
      { input: new TextEncoder().encode("abc") },
      expected,
    );
  });

  test("sha512('')", async () => {
    const expected = new Uint8Array([
      0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd,
      0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d, 0x80, 0x07,
      0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc,
      0x83, 0xf4, 0xa9, 0x21, 0xd3, 0x6c, 0xe9, 0xce,
      0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0,
      0xff, 0x83, 0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f,
      0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81,
      0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e,
    ]);
    await assertSha512Agreement(
      { input: new Uint8Array(0) },
      expected,
    );
  });
});

describe("sha512: padding boundaries", () => {
  // 111 = largest tail_len that fits in one padding block
  // 112 = smallest tail_len that requires a second padding block
  // 127, 128, 129 = compressions-per-message boundaries
  for (const len of [0, 1, 55, 56, 111, 112, 119, 120, 127, 128, 129]) {
    test(`input len = ${len}`, async () => {
      await assertSha512Agreement({ input: patternInput(len) });
    });
  }
});

describe("sha512: block boundaries", () => {
  for (const len of [255, 256, 257]) {
    test(`input len = ${len}`, async () => {
      await assertSha512Agreement({ input: patternInput(len) });
    });
  }
});

describe("sha512: cap endpoints", () => {
  // Upper end of the seeded-random differential range. Ensures nothing
  // unexpected happens at the max supported input size.
  test("input len = 65535", async () => {
    await assertSha512Agreement({ input: patternInput(65535) });
  });
  test("input len = 65536", async () => {
    await assertSha512Agreement({ input: patternInput(65536) });
  });
});

// -----------------------------------------------------------------------------
// Seeded random differential
// -----------------------------------------------------------------------------

describe("sha512: seeded random differential", () => {
  const seedHex = process.env.SHA512_RANDOM_SEED ?? "0123456789abcdef";
  const count = parseInt(process.env.SHA512_RANDOM_COUNT ?? "50", 10);
  const seed = BigInt("0x" + seedHex);

  test(`${count} random inputs (seed=${seedHex})`, async () => {
    const next = splitmix64(seed);
    for (let i = 0; i < count; i++) {
      const inputLen = randInt(next, 0, 65536);
      const input = randomBytes(next, inputLen);
      try {
        await assertSha512Agreement({ input });
      } catch (err) {
        const preview = bytesToHex(input.slice(0, 128));
        console.error(
          `[sha512 random failure] seed=${seedHex} iteration=${i} inputLen=${inputLen}`,
        );
        console.error(
          `  input_hex_preview=${preview}${input.length > 128 ? "..." : ""}`,
        );
        throw err;
      }
    }
  }, 600_000); // 10 minutes: 50 iterations at up to 65 KB each can be slow under PVM.
});
```

- [ ] **Step 4: Run the test file to confirm the harness works**

Run: `cd tests && bun run build && bun test layer3/sha512.test.ts -t "FIPS"`
Expected: ALL FAIL with hash mismatches — the stub returns 64 zeros, not the real SHA-512. The test framework, arg encoding, and three-way harness should all execute without unrelated errors (e.g. no "module not found", no WAT compilation failures, no hex parsing errors).

What we're specifically verifying here: that the assertion message shows **reference hash bytes vs 64 zeros**, not an error in the test plumbing.

- [ ] **Step 5: Commit (WAT stub + full test file)**

```bash
git add tests/fixtures/wat/sha512.jam.wat tests/layer3/sha512.test.ts
git commit -m "$(cat <<'EOF'
test: add sha512 test scaffolding and WAT stub

Stub returns 64 zero bytes so the three-way differential harness can
run end-to-end before the real algorithm lands. All assertions
currently fail with hash mismatches (expected).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Implement the real SHA-512 WAT

**Why:** Replace the stub with the actual FIPS 180-4 algorithm. This task is big (~350 lines of WAT) — we'll build it in small, testable sub-steps.

**Files:**
- Modify: `tests/fixtures/wat/sha512.jam.wat`

The sub-steps below are incremental commits, each moving the test suite closer to green. After each sub-step, run `cd tests && bun run build && bun test layer3/sha512.test.ts -t "FIPS"` and report which tests now pass.

### Step 3.1: Scaffold the module — data segments + empty compress + main that calls compress once on an empty block

Replace `tests/fixtures/wat/sha512.jam.wat` with:

```wat
;; SHA-512 (FIPS 180-4), fixed 64-byte output.
;;
;; Entry: main(args_ptr: i32, args_len: i32) -> i64
;;   args = [input: bytes]  (input may be 0..=65536 bytes; no prefix)
;;   returns (out_ptr: i32) | ((64: i32) << 32)
;;
;; WASM memory layout (all offsets WASM-relative):
;;   0x000..0x03F  output hash buffer (64 bytes)
;;   0x040..0x07F  h[8] state (mutable, 8 x i64)
;;   0x080..0x0BF  initial H[8] constants (data segment, 8 x i64 LE)
;;   0x0C0..0x33F  K[80] round constants (data segment, 80 x i64 LE)
;;   0x340..0x5BF  W[80] message schedule (mutable, 80 x i64)
;;   0x5C0..0x63F  final-block padding buffer (128 bytes)

(module
  (memory (export "memory") 1)

  ;; Initial hash values H[0..7] at 0x80 (64 bytes, 8 x i64 LE).
  (data (i32.const 0x080)
    "\08\c9\bc\f3\67\e6\09\6a"  ;; H[0] = 0x6a09e667f3bcc908
    "\3b\a7\ca\84\85\ae\67\bb"  ;; H[1] = 0xbb67ae8584caa73b
    "\2b\f8\94\fe\72\f3\6e\3c"  ;; H[2] = 0x3c6ef372fe94f82b
    "\f1\36\1d\5f\3a\f5\4f\a5"  ;; H[3] = 0xa54ff53a5f1d36f1
    "\d1\82\e6\ad\7f\52\0e\51"  ;; H[4] = 0x510e527fade682d1
    "\1f\6c\3e\2b\8c\68\05\9b"  ;; H[5] = 0x9b05688c2b3e6c1f
    "\6b\bd\41\fb\ab\d9\83\1f"  ;; H[6] = 0x1f83d9abfb41bd6b
    "\79\21\7e\13\19\cd\e0\5b") ;; H[7] = 0x5be0cd19137e2179

  ;; Round constants K[0..79] at 0xC0 (640 bytes, 80 x i64 LE).
  (data (i32.const 0x0c0)
    "\22\ae\28\d7\98\2f\8a\42"  ;; K[0]  = 0x428a2f98d728ae22
    "\cd\65\ef\23\91\44\37\71"  ;; K[1]  = 0x7137449123ef65cd
    "\2f\3b\4d\ec\cf\fb\c0\b5"  ;; K[2]  = 0xb5c0fbcfec4d3b2f
    "\bc\db\89\81\a5\db\b5\e9"  ;; K[3]  = 0xe9b5dba58189dbbc
    "\38\b5\48\f3\5b\c2\56\39"  ;; K[4]  = 0x3956c25bf348b538
    "\19\d0\05\b6\f1\11\f1\59"  ;; K[5]  = 0x59f111f1b605d019
    "\9b\4f\19\af\a4\82\3f\92"  ;; K[6]  = 0x923f82a4af194f9b
    "\18\81\6d\da\d5\5e\1c\ab"  ;; K[7]  = 0xab1c5ed5da6d8118
    "\42\02\03\a3\98\aa\07\d8"  ;; K[8]  = 0xd807aa98a3030242
    "\be\6f\70\45\01\5b\83\12"  ;; K[9]  = 0x12835b0145706fbe
    "\8c\b2\e4\4e\be\85\31\24"  ;; K[10] = 0x243185be4ee4b28c
    "\e2\b4\ff\d5\c3\7d\0c\55"  ;; K[11] = 0x550c7dc3d5ffb4e2
    "\6f\89\7b\f2\74\5d\be\72"  ;; K[12] = 0x72be5d74f27b896f
    "\b1\96\16\3b\fe\b1\de\80"  ;; K[13] = 0x80deb1fe3b1696b1
    "\35\12\c7\25\a7\06\dc\9b"  ;; K[14] = 0x9bdc06a725c71235
    "\94\26\69\cf\74\f1\9b\c1"  ;; K[15] = 0xc19bf174cf692694
    "\d2\4a\f1\9e\c1\69\9b\e4"  ;; K[16] = 0xe49b69c19ef14ad2
    "\e3\25\4f\38\86\47\be\ef"  ;; K[17] = 0xefbe4786384f25e3
    "\b5\d5\8c\8b\c6\9d\c1\0f"  ;; K[18] = 0x0fc19dc68b8cd5b5
    "\65\9c\ac\77\cc\a1\0c\24"  ;; K[19] = 0x240ca1cc77ac9c65
    "\75\02\2b\59\6f\2c\e9\2d"  ;; K[20] = 0x2de92c6f592b0275
    "\83\e4\a6\6e\aa\84\74\4a"  ;; K[21] = 0x4a7484aa6ea6e483
    "\d4\fb\41\bd\dc\a9\b0\5c"  ;; K[22] = 0x5cb0a9dcbd41fbd4
    "\b5\53\11\83\da\88\f9\76"  ;; K[23] = 0x76f988da831153b5
    "\ab\df\66\ee\52\51\3e\98"  ;; K[24] = 0x983e5152ee66dfab
    "\10\32\b4\2d\6d\c6\31\a8"  ;; K[25] = 0xa831c66d2db43210
    "\3f\21\fb\98\c8\27\03\b0"  ;; K[26] = 0xb00327c898fb213f
    "\e4\0e\ef\be\c7\7f\59\bf"  ;; K[27] = 0xbf597fc7beef0ee4
    "\c2\8f\a8\3d\f3\0b\e0\c6"  ;; K[28] = 0xc6e00bf33da88fc2
    "\25\a7\0a\93\47\91\a7\d5"  ;; K[29] = 0xd5a79147930aa725
    "\6f\82\03\e0\51\63\ca\06"  ;; K[30] = 0x06ca6351e003826f
    "\70\6e\0e\0a\67\29\29\14"  ;; K[31] = 0x142929670a0e6e70
    "\fc\2f\d2\46\85\0a\b7\27"  ;; K[32] = 0x27b70a8546d22ffc
    "\26\c9\26\5c\38\21\1b\2e"  ;; K[33] = 0x2e1b21385c26c926
    "\ed\2a\c4\5a\fc\6d\2c\4d"  ;; K[34] = 0x4d2c6dfc5ac42aed
    "\df\b3\95\9d\13\0d\38\53"  ;; K[35] = 0x53380d139d95b3df
    "\de\63\af\8b\54\73\0a\65"  ;; K[36] = 0x650a73548baf63de
    "\a8\b2\77\3c\bb\0a\6a\76"  ;; K[37] = 0x766a0abb3c77b2a8
    "\e6\ae\ed\47\2e\c9\c2\81"  ;; K[38] = 0x81c2c92e47edaee6
    "\3b\35\82\14\85\2c\72\92"  ;; K[39] = 0x92722c851482353b
    "\64\03\f1\4c\a1\e8\bf\a2"  ;; K[40] = 0xa2bfe8a14cf10364
    "\01\30\42\bc\4b\66\1a\a8"  ;; K[41] = 0xa81a664bbc423001
    "\91\97\f8\d0\70\8b\4b\c2"  ;; K[42] = 0xc24b8b70d0f89791
    "\30\be\54\06\a3\51\6c\c7"  ;; K[43] = 0xc76c51a30654be30
    "\18\52\ef\d6\19\e8\92\d1"  ;; K[44] = 0xd192e819d6ef5218
    "\10\a9\65\55\24\06\99\d6"  ;; K[45] = 0xd69906245565a910
    "\2a\20\71\57\85\35\0e\f4"  ;; K[46] = 0xf40e35855771202a
    "\b8\d1\bb\32\70\a0\6a\10"  ;; K[47] = 0x106aa07032bbd1b8
    "\c8\d0\d2\b8\16\c1\a4\19"  ;; K[48] = 0x19a4c116b8d2d0c8
    "\53\ab\41\51\08\6c\37\1e"  ;; K[49] = 0x1e376c085141ab53
    "\99\eb\8e\df\4c\77\48\27"  ;; K[50] = 0x2748774cdf8eeb99
    "\a8\48\9b\e1\b5\bc\b0\34"  ;; K[51] = 0x34b0bcb5e19b48a8
    "\63\5a\c9\c5\b3\0c\1c\39"  ;; K[52] = 0x391c0cb3c5c95a63
    "\cb\8a\41\e3\4a\aa\d8\4e"  ;; K[53] = 0x4ed8aa4ae3418acb
    "\73\e3\63\77\4f\ca\9c\5b"  ;; K[54] = 0x5b9cca4f7763e373
    "\a3\b8\b2\d6\f3\6f\2e\68"  ;; K[55] = 0x682e6ff3d6b2b8a3
    "\fc\b2\ef\5d\ee\82\8f\74"  ;; K[56] = 0x748f82ee5defb2fc
    "\60\2f\17\43\6f\63\a5\78"  ;; K[57] = 0x78a5636f43172f60
    "\72\ab\f0\a1\14\78\c8\84"  ;; K[58] = 0x84c87814a1f0ab72
    "\ec\39\64\1a\08\02\c7\8c"  ;; K[59] = 0x8cc702081a6439ec
    "\28\1e\63\23\fa\ff\be\90"  ;; K[60] = 0x90befffa23631e28
    "\e9\bd\82\de\eb\6c\50\a4"  ;; K[61] = 0xa4506cebde82bde9
    "\15\79\c6\b2\f7\a3\f9\be"  ;; K[62] = 0xbef9a3f7b2c67915
    "\2b\53\72\e3\f2\78\71\c6"  ;; K[63] = 0xc67178f2e372532b
    "\9c\61\26\ea\ce\3e\27\ca"  ;; K[64] = 0xca273eceea26619c
    "\07\c2\c0\21\c7\b8\86\d1"  ;; K[65] = 0xd186b8c721c0c207
    "\1e\eb\e0\cd\d6\7d\da\ea"  ;; K[66] = 0xeada7dd6cde0eb1e
    "\78\d1\6e\ee\7f\4f\7d\f5"  ;; K[67] = 0xf57d4f7fee6ed178
    "\ba\6f\17\72\aa\67\f0\06"  ;; K[68] = 0x06f067aa72176fba
    "\a6\98\c8\a2\c5\7d\63\0a"  ;; K[69] = 0x0a637dc5a2c898a6
    "\ae\0d\f9\be\04\98\3f\11"  ;; K[70] = 0x113f9804bef90dae
    "\1b\47\1c\13\35\0b\71\1b"  ;; K[71] = 0x1b710b35131c471b
    "\84\7d\04\23\f5\77\db\28"  ;; K[72] = 0x28db77f523047d84
    "\93\24\c7\40\7b\ab\ca\32"  ;; K[73] = 0x32caab7b40c72493
    "\bc\be\c9\15\0a\be\9e\3c"  ;; K[74] = 0x3c9ebe0a15c9bebc
    "\4c\0d\10\9c\c4\67\1d\43"  ;; K[75] = 0x431d67c49c100d4c
    "\b6\42\3e\cb\be\d4\c5\4c"  ;; K[76] = 0x4cc5d4becb3e42b6
    "\2a\7e\65\fc\9c\29\7f\59"  ;; K[77] = 0x597f299cfc657e2a
    "\ec\fa\d6\3a\ab\6f\cb\5f"  ;; K[78] = 0x5fcb6fab3ad6faec
    "\17\58\47\4a\8c\19\44\6c") ;; K[79] = 0x6c44198c4a475817

  ;; Placeholder — full compress follows in 3.3.
  (func $compress)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Placeholder: copy initial H into h state, then write h to output as-is.
    ;; Will NOT produce correct hashes — this step only verifies the module
    ;; compiles and the memory layout is consistent.
    (memory.copy (i32.const 0x040) (i32.const 0x080) (i32.const 64))
    (memory.copy (i32.const 0)     (i32.const 0x040) (i32.const 64))
    (i64.const 274877906944)))  ;; (0 | (64 << 32))
```

- [ ] **Step 3.1.1: Compile and smoke test**

Run: `cd tests && bun build.ts`
Expected: Build succeeds, `tests/build/jam/sha512.jam` exists.

Run: `cd tests && bun test layer3/sha512.test.ts -t "FIPS"`
Expected: Both FIPS tests FAIL with hash mismatches. No module-load, WAT-compile, or type errors.

- [ ] **Step 3.1.2: Commit**

```bash
git add tests/fixtures/wat/sha512.jam.wat
git commit -m "$(cat <<'EOF'
feat(sha512-wat): scaffold module with H + K data segments

Places the FIPS 180-4 initial H[8] and round constants K[0..79] into
data segments at fixed offsets. Empty compress; main is a placeholder
that just copies H into the output.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Step 3.2: Add the `$bswap64` helper

Insert this function into `sha512.jam.wat` after the `K` data segment and before `(func $compress)`:

```wat
  ;; --- Helper: byte-swap an i64 (LE ↔ BE) ---
  ;; WAT has no native bswap. Implemented via shifts + masks.
  (func $bswap64 (param $x i64) (result i64)
    (i64.or
      (i64.or
        (i64.or
          (i64.shl (local.get $x) (i64.const 56))
          (i64.shl (i64.and (local.get $x) (i64.const 0x000000000000FF00))
                   (i64.const 40)))
        (i64.or
          (i64.shl (i64.and (local.get $x) (i64.const 0x0000000000FF0000))
                   (i64.const 24))
          (i64.shl (i64.and (local.get $x) (i64.const 0x00000000FF000000))
                   (i64.const  8))))
      (i64.or
        (i64.or
          (i64.and (i64.shr_u (local.get $x) (i64.const  8))
                   (i64.const 0x00000000FF000000))
          (i64.and (i64.shr_u (local.get $x) (i64.const 24))
                   (i64.const 0x0000000000FF0000)))
        (i64.or
          (i64.and (i64.shr_u (local.get $x) (i64.const 40))
                   (i64.const 0x000000000000FF00))
          (i64.shr_u (local.get $x) (i64.const 56))))))
```

- [ ] **Step 3.2.1: Rebuild and verify still compiles**

Run: `cd tests && bun build.ts`
Expected: Build succeeds.

Run: `cd tests && bun test layer3/sha512.test.ts -t "FIPS"`
Expected: Still fails with the same hash mismatches — we haven't changed `main`/`$compress` yet.

- [ ] **Step 3.2.2: Commit**

```bash
git add tests/fixtures/wat/sha512.jam.wat
git commit -m "$(cat <<'EOF'
feat(sha512-wat): add bswap64 helper

WAT has no native i64 bswap. Implemented with masked shifts.
Called 16 times per block to convert LE input words to BE for the
SHA-512 message schedule.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Step 3.3: Implement `$compress`

Replace the placeholder `(func $compress)` with the full compression function:

```wat
  ;; --- compress(block_ptr: i32) ---
  ;;
  ;; Consumes one 128-byte block at $block_ptr, mutates h[].
  (func $compress (param $block_ptr i32)
    (local $a i64) (local $b i64) (local $c i64) (local $d i64)
    (local $e i64) (local $f i64) (local $g i64) (local $hh i64)
    (local $t1 i64) (local $t2 i64)
    (local $i i32) (local $w_ptr i32)

    ;; --- Build W[0..15] from the block: load LE then bswap to BE ---
    (local.set $i (i32.const 0))
    (block $w_load_exit
      (loop $w_load
        (br_if $w_load_exit (i32.ge_u (local.get $i) (i32.const 128)))
        (i64.store
          (i32.add (i32.const 0x340) (local.get $i))
          (call $bswap64
            (i64.load (i32.add (local.get $block_ptr) (local.get $i)))))
        (local.set $i (i32.add (local.get $i) (i32.const 8)))
        (br $w_load)))

    ;; --- Extend W[16..79] using SSIG0/SSIG1 ---
    ;; W[t] = SSIG1(W[t-2]) + W[t-7] + SSIG0(W[t-15]) + W[t-16]
    ;; SSIG0(x) = rotr(x,1) ^ rotr(x,8) ^ (x >> 7)
    ;; SSIG1(x) = rotr(x,19) ^ rotr(x,61) ^ (x >> 6)
    ;; We index W[] in memory: byte offset = t*8, from 0x340.
    (local.set $i (i32.const 16))
    (block $w_ext_exit
      (loop $w_ext
        (br_if $w_ext_exit (i32.ge_u (local.get $i) (i32.const 80)))
        ;; w_ptr = 0x340 + i*8
        (local.set $w_ptr
          (i32.add (i32.const 0x340)
            (i32.shl (local.get $i) (i32.const 3))))
        (i64.store (local.get $w_ptr)
          (i64.add
            (i64.add
              ;; SSIG1(W[i-2])
              (i64.xor
                (i64.xor
                  (i64.rotr (i64.load offset=-16 (local.get $w_ptr)) (i64.const 19))
                  (i64.rotr (i64.load offset=-16 (local.get $w_ptr)) (i64.const 61)))
                (i64.shr_u (i64.load offset=-16 (local.get $w_ptr)) (i64.const 6)))
              ;; W[i-7]
              (i64.load offset=-56 (local.get $w_ptr)))
            (i64.add
              ;; SSIG0(W[i-15])
              (i64.xor
                (i64.xor
                  (i64.rotr (i64.load offset=-120 (local.get $w_ptr)) (i64.const 1))
                  (i64.rotr (i64.load offset=-120 (local.get $w_ptr)) (i64.const 8)))
                (i64.shr_u (i64.load offset=-120 (local.get $w_ptr)) (i64.const 7)))
              ;; W[i-16]
              (i64.load offset=-128 (local.get $w_ptr)))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $w_ext)))

    ;; --- Initialize working variables from h[] ---
    (local.set $a  (i64.load offset=0  (i32.const 0x040)))
    (local.set $b  (i64.load offset=8  (i32.const 0x040)))
    (local.set $c  (i64.load offset=16 (i32.const 0x040)))
    (local.set $d  (i64.load offset=24 (i32.const 0x040)))
    (local.set $e  (i64.load offset=32 (i32.const 0x040)))
    (local.set $f  (i64.load offset=40 (i32.const 0x040)))
    (local.set $g  (i64.load offset=48 (i32.const 0x040)))
    (local.set $hh (i64.load offset=56 (i32.const 0x040)))

    ;; --- 80 rounds ---
    ;; T1 = hh + BSIG1(e) + Ch(e,f,g) + K[t] + W[t]
    ;; T2 = BSIG0(a) + Maj(a,b,c)
    ;; BSIG0(x) = rotr(x,28) ^ rotr(x,34) ^ rotr(x,39)
    ;; BSIG1(x) = rotr(x,14) ^ rotr(x,18) ^ rotr(x,41)
    ;; Ch(x,y,z)  = (x & y) ^ (~x & z)
    ;; Maj(x,y,z) = (x & y) ^ (x & z) ^ (y & z)
    ;;
    ;; K and W are 8-byte-indexed; we keep a byte-offset counter $i and read
    ;; K[t] from 0xC0 + i, W[t] from 0x340 + i.
    (local.set $i (i32.const 0))
    (block $rounds_exit
      (loop $rounds
        (br_if $rounds_exit (i32.ge_u (local.get $i) (i32.const 640))) ;; 80*8

        ;; T1 = hh + BSIG1(e) + Ch(e,f,g) + K[t] + W[t]
        (local.set $t1
          (i64.add
            (i64.add
              (i64.add (local.get $hh)
                ;; BSIG1(e)
                (i64.xor
                  (i64.xor
                    (i64.rotr (local.get $e) (i64.const 14))
                    (i64.rotr (local.get $e) (i64.const 18)))
                  (i64.rotr (local.get $e) (i64.const 41))))
              ;; Ch(e,f,g) = (e & f) ^ (~e & g)
              (i64.xor
                (i64.and (local.get $e) (local.get $f))
                (i64.and (i64.xor (local.get $e) (i64.const -1)) (local.get $g))))
            (i64.add
              (i64.load (i32.add (i32.const 0x0c0) (local.get $i))) ;; K[t]
              (i64.load (i32.add (i32.const 0x340) (local.get $i)))))) ;; W[t]

        ;; T2 = BSIG0(a) + Maj(a,b,c)
        (local.set $t2
          (i64.add
            ;; BSIG0(a)
            (i64.xor
              (i64.xor
                (i64.rotr (local.get $a) (i64.const 28))
                (i64.rotr (local.get $a) (i64.const 34)))
              (i64.rotr (local.get $a) (i64.const 39)))
            ;; Maj(a,b,c) = (a & b) ^ (a & c) ^ (b & c)
            (i64.xor
              (i64.xor
                (i64.and (local.get $a) (local.get $b))
                (i64.and (local.get $a) (local.get $c)))
              (i64.and (local.get $b) (local.get $c)))))

        ;; Shift state: hh = g; g = f; f = e; e = d + T1; d = c; c = b; b = a; a = T1 + T2.
        (local.set $hh (local.get $g))
        (local.set $g (local.get $f))
        (local.set $f (local.get $e))
        (local.set $e (i64.add (local.get $d) (local.get $t1)))
        (local.set $d (local.get $c))
        (local.set $c (local.get $b))
        (local.set $b (local.get $a))
        (local.set $a (i64.add (local.get $t1) (local.get $t2)))

        (local.set $i (i32.add (local.get $i) (i32.const 8)))
        (br $rounds)))

    ;; --- Add the compressed chunk to the current h[] ---
    (i64.store offset=0  (i32.const 0x040) (i64.add (i64.load offset=0  (i32.const 0x040)) (local.get $a)))
    (i64.store offset=8  (i32.const 0x040) (i64.add (i64.load offset=8  (i32.const 0x040)) (local.get $b)))
    (i64.store offset=16 (i32.const 0x040) (i64.add (i64.load offset=16 (i32.const 0x040)) (local.get $c)))
    (i64.store offset=24 (i32.const 0x040) (i64.add (i64.load offset=24 (i32.const 0x040)) (local.get $d)))
    (i64.store offset=32 (i32.const 0x040) (i64.add (i64.load offset=32 (i32.const 0x040)) (local.get $e)))
    (i64.store offset=40 (i32.const 0x040) (i64.add (i64.load offset=40 (i32.const 0x040)) (local.get $f)))
    (i64.store offset=48 (i32.const 0x040) (i64.add (i64.load offset=48 (i32.const 0x040)) (local.get $g)))
    (i64.store offset=56 (i32.const 0x040) (i64.add (i64.load offset=56 (i32.const 0x040)) (local.get $hh))))
```

- [ ] **Step 3.3.1: Smoke test — still fails (no real main yet)**

Run: `cd tests && bun build.ts && bun test layer3/sha512.test.ts -t "FIPS"`
Expected: Still fails. Compiles without errors.

- [ ] **Step 3.3.2: Commit**

```bash
git add tests/fixtures/wat/sha512.jam.wat
git commit -m "$(cat <<'EOF'
feat(sha512-wat): implement compress (message schedule + 80 rounds)

Builds W[0..79] from the block (W[0..15] via bswap, W[16..79] via
SSIG expansion), runs 80 SHA-512 rounds cycling 8 i64 locals, and
adds the compressed chunk into h[].

Main still stub — next step wires padding and stream driver.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Step 3.4: Implement `main` — full block streaming + padding

Replace the placeholder `main` with the full driver:

```wat
  ;; --- main ---
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $data_ptr i32)
    (local $remaining i32)
    (local $bit_len_lo i64)
    (local $tail_len i32)

    ;; h[0..7] = H[0..7] (one 64-byte copy from the initial-H data segment).
    (memory.copy (i32.const 0x040) (i32.const 0x080) (i32.const 64))

    ;; data_ptr = args_ptr; remaining = args_len
    (local.set $data_ptr  (local.get $args_ptr))
    (local.set $remaining (local.get $args_len))

    ;; Total bit length (for the final padding). SHA-512 uses a 128-bit big-
    ;; endian bit-count; our cap (64 KB) keeps the value in the low 64 bits.
    ;; Cast args_len -> i64 and << 3.
    (local.set $bit_len_lo
      (i64.shl (i64.extend_i32_u (local.get $args_len)) (i64.const 3)))

    ;; --- Stream full 128-byte blocks while remaining >= 128 ---
    (block $stream_exit
      (loop $stream
        (br_if $stream_exit (i32.lt_u (local.get $remaining) (i32.const 128)))
        (call $compress (local.get $data_ptr))
        (local.set $data_ptr  (i32.add (local.get $data_ptr)  (i32.const 128)))
        (local.set $remaining (i32.sub (local.get $remaining) (i32.const 128)))
        (br $stream)))

    ;; --- Final-block padding ---
    ;; tail_len = remaining (0..127)
    (local.set $tail_len (local.get $remaining))

    ;; Zero the padding buffer (128 bytes).
    (memory.fill (i32.const 0x5c0) (i32.const 0) (i32.const 128))

    ;; Copy tail_len bytes of input into the buffer.
    (if (i32.gt_u (local.get $tail_len) (i32.const 0))
      (then
        (memory.copy (i32.const 0x5c0) (local.get $data_ptr) (local.get $tail_len))))

    ;; Write the 0x80 terminator byte at [tail_len].
    (i32.store8
      (i32.add (i32.const 0x5c0) (local.get $tail_len))
      (i32.const 0x80))

    (if (i32.le_u (local.get $tail_len) (i32.const 111))
      (then
        ;; --- Single-block padding ---
        ;; Write the 128-bit BE bit length into bytes [112..127].
        ;; Upper 8 bytes are zero (memory.fill already zeroed them); write
        ;; the lower 8 as big-endian (bswap).
        (i64.store offset=120 (i32.const 0x5c0)
          (call $bswap64 (local.get $bit_len_lo)))
        (call $compress (i32.const 0x5c0)))
      (else
        ;; --- Two-block padding ---
        ;; First block: tail + 0x80 + zeros to end-of-block. (Already
        ;; assembled in 0x5C0..0x63F — terminator at tail_len, rest zero.)
        (call $compress (i32.const 0x5c0))
        ;; Second block: 112 zeros + 16-byte BE bit length.
        (memory.fill (i32.const 0x5c0) (i32.const 0) (i32.const 128))
        (i64.store offset=120 (i32.const 0x5c0)
          (call $bswap64 (local.get $bit_len_lo)))
        (call $compress (i32.const 0x5c0))))

    ;; --- Output: h[] as 8 × BE i64 into the output buffer at offset 0. ---
    (i64.store offset=0  (i32.const 0) (call $bswap64 (i64.load offset=0  (i32.const 0x040))))
    (i64.store offset=8  (i32.const 0) (call $bswap64 (i64.load offset=8  (i32.const 0x040))))
    (i64.store offset=16 (i32.const 0) (call $bswap64 (i64.load offset=16 (i32.const 0x040))))
    (i64.store offset=24 (i32.const 0) (call $bswap64 (i64.load offset=24 (i32.const 0x040))))
    (i64.store offset=32 (i32.const 0) (call $bswap64 (i64.load offset=32 (i32.const 0x040))))
    (i64.store offset=40 (i32.const 0) (call $bswap64 (i64.load offset=40 (i32.const 0x040))))
    (i64.store offset=48 (i32.const 0) (call $bswap64 (i64.load offset=48 (i32.const 0x040))))
    (i64.store offset=56 (i32.const 0) (call $bswap64 (i64.load offset=56 (i32.const 0x040))))

    ;; Return (0 | (64 << 32)) = 0x0000_0040_0000_0000 = 274877906944
    (i64.const 274877906944))
```

- [ ] **Step 3.4.1: Run the FIPS tests**

Run: `cd tests && bun build.ts && bun test layer3/sha512.test.ts -t "FIPS"`
Expected: Both FIPS vectors PASS. If they don't:

- Compare the first mismatched byte — if byte 0 is wrong, the `h[0]` final value is wrong (`$a` or the initial-H copy is off).
- If the pattern is "shifted" (e.g. all bytes shifted by one), check `$bswap64`.
- If the empty-input test passes but `"abc"` fails, the input loading is wrong (re-check W[0..15] construction).
- If `"abc"` passes but empty fails, the padding path is wrong.

- [ ] **Step 3.4.2: Run the padding-boundary tests**

Run: `cd tests && bun test layer3/sha512.test.ts -t "padding boundaries"`
Expected: All 11 tests PASS (0, 1, 55, 56, 111, 112, 119, 120, 127, 128, 129). The 111→112 transition is the most likely place for a padding bug.

- [ ] **Step 3.4.3: Run the block-boundary tests**

Run: `cd tests && bun test layer3/sha512.test.ts -t "block boundaries"`
Expected: All 3 tests PASS (255, 256, 257).

- [ ] **Step 3.4.4: Run the cap-endpoint tests**

Run: `cd tests && bun test layer3/sha512.test.ts -t "cap endpoints"`
Expected: Both tests PASS (65535, 65536). These are the slowest — each hashes ~65 KB through the streaming path.

- [ ] **Step 3.4.5: Commit**

```bash
git add tests/fixtures/wat/sha512.jam.wat
git commit -m "$(cat <<'EOF'
feat(sha512-wat): implement main (streaming + padding)

Drives the full SHA-512 algorithm:
- Initialize h[] from the initial-H data segment via memory.copy.
- Stream full 128-byte blocks through $compress.
- Build the final padding block at 0x5C0, splitting into one-block
  (tail_len <= 111) or two-block (tail_len >= 112) paths.
- Write the final h[] as 8 big-endian i64 to the output buffer.
- Return (0 | (64 << 32)).

All unit tests pass: FIPS vectors, padding boundaries, block
boundaries, cap endpoints.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Step 3.5: Run the seeded random differential

- [ ] **Step 3.5.1: Run with default count**

Run: `cd tests && bun test layer3/sha512.test.ts -t "random"`
Expected: PASS — 50 random inputs of length `[0, 65536]` all match three-way.

If a case fails, the log output contains `seed`, `iteration`, `inputLen`, and a 128-byte hex preview. Convert to a standalone unit test by adding the input length to a new `describe` block, rerun, and debug.

- [ ] **Step 3.5.2: Run the heavy random test (1000 iterations)**

This is the pre-merge sanity check. It's slow (several minutes) but catches rare bugs that 50 iterations miss.

Run: `cd tests && SHA512_RANDOM_COUNT=1000 bun test layer3/sha512.test.ts -t "random"`
Expected: PASS.

- [ ] **Step 3.5.3: Run the full test suite to confirm no regressions**

Run: `cd tests && bun run test`
Expected: Full suite PASS, all layer 1/2/3 tests green.

- [ ] **Step 3.5.4: Commit if any tweaks were needed**

If Steps 3.5.1–3.5.3 revealed and fixed any WAT bugs, commit them now. Otherwise nothing to commit.

---

## Task 4: Documentation updates

**Why:** `AGENTS.md` has a "Where to Look" table that should reference the new fixture and helpers. `learnings.md` captures non-obvious technical surprises.

**Files:**
- Modify: `AGENTS.md`

- [ ] **Step 1: Add SHA-512 to the "Where to Look" table**

Find the blake2b entry in `AGENTS.md` (`Hand-crafted crypto example`). Add an equivalent row for SHA-512 right after it.

Before:
```markdown
| Hand-crafted crypto example | `tests/fixtures/wat/blake2b.jam.wat` + `tests/layer3/blake2b.test.ts` | RFC 7693 blake2b (unkeyed, variable output 1..=64) with 3-way agreement tests vs `@noble/hashes` |
```

After:
```markdown
| Hand-crafted crypto example | `tests/fixtures/wat/blake2b.jam.wat` + `tests/layer3/blake2b.test.ts` | RFC 7693 blake2b (unkeyed, variable output 1..=64) with 3-way agreement tests vs `@noble/hashes` |
| Hand-crafted hash example (SHA-2) | `tests/fixtures/wat/sha512.jam.wat` + `tests/layer3/sha512.test.ts` | FIPS 180-4 SHA-512 (fixed 64-byte output) with 3-way agreement tests vs `@noble/hashes/sha2`. Input cap 32 KB (capped to keep the hex CLI encoding under Linux's 128 KB per-argv-string limit). |
```

- [ ] **Step 2: Capture any learnings discovered during implementation**

Open `docs/src/learnings.md`. If any of the following surprises came up during Task 3, add a short entry (one paragraph each):

- **`$bswap64` code size**: if the 10-op helper compiled to something surprisingly large or small, note it.
- **80-iteration round-loop code size**: if register-allocation / phi handling produced anything unusual for 8 loop-carried i64 locals.
- **Padding-path branching**: if the if-then-else for single-vs-two-block padding caused a subtle code-gen issue.

If none of these surprised, skip this step — don't invent content.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md docs/src/learnings.md
git commit -m "$(cat <<'EOF'
docs: document SHA-512 WAT fixture

Adds a "Where to Look" entry for sha512.jam.wat and captures any
lessons from hand-crafting SHA-512 on PVM.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Benchmark, PR, open

**Why:** Project policy requires a benchmark comparison in every PR description.

- [ ] **Step 1: Run the benchmark**

From the repo root:

```bash
./tests/utils/benchmark.sh --base main --current td-sha512-wat
```

This produces the standard Optimizations Impact + PVM-in-PVM tables. Capture the output.

- [ ] **Step 2: Push the branch**

```bash
git push -u origin td-sha512-wat
```

- [ ] **Step 3: Open the PR**

```bash
gh pr create --title "feat: hand-crafted SHA-512 WAT example with differential tests" --body "$(cat <<'EOF'
## Summary

- Hand-crafted SHA-512 WAT fixture at `tests/fixtures/wat/sha512.jam.wat`, following the blake2b pattern from #194.
- Layer-3 tests at `tests/layer3/sha512.test.ts`: FIPS 180-4 vectors, padding-boundary edges, block-boundary edges, cap-endpoint edges (32767, 32768), cap rejection (32769), and 50-iteration seeded random differential (32 KB cap).
- Three-way byte-level agreement per test: PVM == native WASM == `@noble/hashes/sha2`.
- First PR in a two-PR stack — ed25519 verify (PR B) follows.

## Benchmark

<!-- paste output of ./tests/utils/benchmark.sh --base main --current td-sha512-wat here -->

## Test plan

- [x] `bun run test` passes (full suite)
- [x] `SHA512_RANDOM_COUNT=1000 bun test layer3/sha512.test.ts -t "random"` passes locally
- [x] `cargo check --workspace` clean

## Notes

- Issue #197 tracks raising blake2b's cap from 2 KB to 64 KB to match.
- `runJamBytes` / `runWasmNativeBytes` / `@noble/hashes` dep all landed in #194 — no infra changes here.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 4: After opening the PR, paste the benchmark output into the PR body**

Edit the PR via `gh pr edit --body-file <file>` or the web UI to replace the `<!-- paste output ... -->` placeholder with the actual benchmark table.

---

## Self-review notes

- All "TBD"/placeholder sections removed. Every code block is complete.
- Types used consistently: `Sha512Args`, `sha512Ref`, `encodeSha512Args`, `assertSha512Agreement`.
- Coverage vs spec:
  - ✅ FIPS vectors (Task 2, Task 3.4)
  - ✅ Padding boundaries (Task 2 test list at 111, 112, 119, 120 — matches spec)
  - ✅ Block boundaries (Task 2 test list at 255, 256, 257)
  - ✅ Cap endpoints (Task 2 at 32767, 32768 — see post-implementation note at the top)
  - ✅ Seeded random differential (Task 2 + Task 3.5)
  - ✅ Three-way agreement (Task 2 `assertSha512Agreement`)
  - ✅ Memory layout matches spec (offsets `0x000`…`0x640`)
  - ✅ Both padding paths (Task 3.4)
  - ✅ Docs update (Task 4)
  - ✅ Benchmark (Task 5)
- K-constant LE byte encodings in Step 3.1 are the load-bearing correctness detail. Each is the 8-byte LE form of the constant; an error here would manifest as an immediate FIPS-vector mismatch in 3.4.1 — caught before any random tests run.
