# Blake2b WAT example implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a hand-crafted WAT blake2b fixture that compiles through the WASM→PVM pipeline, with byte-level three-way differential testing against `@noble/hashes` and native WebAssembly.

**Architecture:** RFC 7693 blake2b, unkeyed, variable output length 1..=64. WAT module with `G`, `compress`, and `main` functions plus active data segments for IV and sigma. Tests extend existing helpers with byte-returning variants (the current `runJam`/`runWasmNative` clip to u32) and write standalone tests rather than retrofit `defineSuite`.

**Tech Stack:** WAT (hand-crafted), existing Rust WASM→PVM compiler, Bun test runner, TypeScript, `@noble/hashes` reference library, `wabt` for WAT→WASM (already installed).

**Spec:** `docs/superpowers/specs/2026-04-20-blake2b-wat-example-design.md`

---

## File Structure

**Created:**
- `tests/fixtures/wat/blake2b.jam.wat` — the hand-crafted WAT module (~250–350 lines)
- `tests/layer3/blake2b.test.ts` — unit tests + seeded random differential
- `tests/helpers/blake2b-ref.ts` — thin wrapper around `@noble/hashes/blake2b` that matches the `[out_len: u8][input]` ABI

**Modified:**
- `tests/helpers/run.ts` — add `runJamBytes()` returning raw `Uint8Array`
- `tests/helpers/wasm-runner.ts` — add `runWasmNativeBytes()` returning raw `Uint8Array`
- `tests/package.json` — add `@noble/hashes` dev dep
- `AGENTS.md` — update "Where to Look" table with blake2b fixture + new helpers

**Responsibilities:**
- `run.ts` / `wasm-runner.ts`: byte-accurate runners, no semantic layer
- `blake2b-ref.ts`: ABI-matching reference, computes expected hashes
- `blake2b.test.ts`: three-way agreement (PVM == native WASM == reference) per test
- `blake2b.jam.wat`: the algorithm

---

## Verification Commands

From the `tests/` directory:

- **Build artifacts:** `bun build.ts` (compiles Rust CLI + all WATs→JAMs + AS→WASM)
- **Force rebuild:** `rm -f tests/build/wasm/*.wasm && bun build.ts` (for stale AS caches)
- **Run layer3 only:** `bun test layer3/blake2b.test.ts`
- **Run full suite:** `bun run test`
- **Run one test by name:** `bun test layer3/blake2b.test.ts -t "abc"`

From the repo root:

- **Type/clippy check:** `cargo check --workspace` (fast, not strictly needed since we're touching WAT + TS, but runs as pre-push hook)
- **Benchmark vs main:** `./tests/utils/benchmark.sh --base main --current td-blake2b-wat`

---

## Key Constants (hardcode these — they are the algorithm)

**Blake2b IV** (RFC 7693 §2.6, same as SHA-512's IV):
```
IV[0] = 0x6A09E667F3BCC908
IV[1] = 0xBB67AE8584CAA73B
IV[2] = 0x3C6EF372FE94F82B
IV[3] = 0xA54FF53A5F1D36F1
IV[4] = 0x510E527FADE682D1
IV[5] = 0x9B05688C2B3E6C1F
IV[6] = 0x1F83D9ABFB41BD6B
IV[7] = 0x5BE0CD19137E2179
```

**Sigma permutation table** (RFC 7693 §2.7, 10 rows × 16 bytes):
```
Row 0:  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15
Row 1: 14 10  4  8  9 15 13  6  1 12  0  2 11  7  5  3
Row 2: 11  8 12  0  5  2 15 13 10 14  3  6  7  1  9  4
Row 3:  7  9  3  1 13 12 11 14  2  6  5 10  4  0 15  8
Row 4:  9  0  5  7  2  4 10 15 14  1 11 12  6  8  3 13
Row 5:  2 12  6 10  0 11  8  3  4 13  7  5 15 14  1  9
Row 6: 12  5  1 15 14 13  4 10  0  7  6  3  9  2  8 11
Row 7: 13 11  7 14 12  1  3  9  5  0 15  4  8  6  2 10
Row 8:  6 15 14  9 11  3  0  8 12  2 13  7  1  4 10  5
Row 9: 10  2  8  4  7  6  1  5 15 11  9 14  3 12 13  0
```

For rounds 10 and 11 (indexed 0..11), use rows 0 and 1 respectively (`round mod 10`).

**Memory layout (WASM-relative offsets):**

| Offset | Size | Purpose |
|---|---|---|
| `0x000` | 64 B  | output hash buffer (where the i64 return pointer will point) |
| `0x040` | 64 B  | h[8] state (mutable, 8 × i64) |
| `0x080` | 64 B  | IV[8] constants (data segment) |
| `0x0C0` | 128 B | v[16] working state (mutable, 16 × i64) |
| `0x140` | 128 B | m[16] current message block (mutable, 16 × i64) |
| `0x1C0` | 160 B | sigma[10][16] permutation table (data segment, u8) |
| `0x260` | 16 B  | t counter (i64 at 0x260) + scratch |

---

## Task 1: Add `runJamBytes` byte-returning helper

**Why:** The existing `runJam` clips the result to 4 bytes (see `tests/helpers/run.ts:27-49`). Blake2b outputs 1–64 bytes, so we need a runner that returns the full result as a `Uint8Array`.

**Files:**
- Modify: `tests/helpers/run.ts`
- Test: `tests/helpers/run-bytes.test.ts` (new — delete after Task 1, or leave; see Step 6)

- [ ] **Step 1: Write a failing test against an existing fixture**

Create `tests/helpers/run-bytes.test.ts`:

```ts
import { test, expect, beforeAll } from "bun:test";
import path from "node:path";
import { execSync } from "node:child_process";
import { runJamBytes } from "./run";
import { JAM_DIR, PROJECT_ROOT } from "./paths";

beforeAll(() => {
  // Ensure JAM is built; the build script is a no-op if nothing changed.
  execSync("bun build.ts", { cwd: path.join(PROJECT_ROOT, "tests"), stdio: "inherit" });
});

test("runJamBytes returns raw result bytes for add.jam", () => {
  // add.jam.wat: main(args_ptr, args_len) loads two u32 from args, stores sum at offset 0,
  // returns (ptr=0, len=4). So the result bytes are the little-endian u32 sum of the two args.
  // args = 05000000 07000000 (two i32 LE: 5 and 7), expected sum = 12 => bytes [0x0c, 0, 0, 0]
  const jamFile = path.join(JAM_DIR, "add.jam");
  const result = runJamBytes(jamFile, "0500000007000000");
  expect(result).toEqual(new Uint8Array([0x0c, 0x00, 0x00, 0x00]));
});
```

- [ ] **Step 2: Run test to verify it fails (symbol not defined)**

Run: `cd tests && bun test helpers/run-bytes.test.ts`
Expected: FAIL with "Export named 'runJamBytes' not found" (or similar import error).

- [ ] **Step 3: Add `runJamBytes` implementation**

Modify `tests/helpers/run.ts`. Add below the existing `parseExitValue` function:

```ts
/**
 * Parse the full raw result bytes from anan-as `Result: [0x...]` output.
 * Unlike `parseExitValue`, returns the complete byte string without truncation.
 */
function parseResultBytes(output: string): Uint8Array {
  const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]*)\]/);
  if (!resultMatch) {
    throw new Error(`Could not parse result from output: ${output}`);
  }
  let hex = resultMatch[1];
  if (hex.length % 2 !== 0) {
    hex = "0" + hex;
  }
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

/**
 * Run a JAM file and return the raw result bytes (no truncation).
 *
 * Unlike `runJam` which collapses to a u32, this preserves the full output.
 * Use for fixtures that return more than 4 bytes (e.g. hash functions).
 *
 * @param gas optional gas override. Defaults to 100_000_000 (matches `runJam`).
 */
export function runJamBytes(
  jamFile: string,
  args: string,
  pc?: number,
  gas: number = 100_000_000,
): Uint8Array {
  let cmd = `node ${ANAN_AS_CLI} run --spi --no-logs --gas=${gas}`;
  if (pc !== undefined) cmd += ` --pc=${pc}`;
  cmd += ` ${jamFile} 0x${args}`;

  try {
    const output = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    return parseResultBytes(output);
  } catch (error: any) {
    if (error.stdout) console.log(error.stdout.toString());
    if (error.stderr) console.error(error.stderr.toString());
    throw new Error(`Execution failed: ${error.message.split("\n")[0]}`, { cause: error });
  }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd tests && bun test helpers/run-bytes.test.ts`
Expected: PASS.

If the test fails because `add.jam` doesn't exist, run `bun build.ts` in the `tests/` dir first, then retry.

- [ ] **Step 5: Remove the temporary test file**

The helper test is infrastructure scaffolding. We'll exercise `runJamBytes` more thoroughly via the blake2b tests themselves. Delete it so we don't leave dead test files behind:

```bash
rm tests/helpers/run-bytes.test.ts
```

- [ ] **Step 6: Commit**

```bash
git add tests/helpers/run.ts
git commit -m "$(cat <<'EOF'
test: add runJamBytes helper for non-u32 results

Existing runJam clips the anan-as Result: [0x...] output to 4 bytes to fit
i32. Hash fixtures need the full byte string.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add `runWasmNativeBytes` byte-returning helper

**Why:** The existing `runWasmNative` (at `tests/helpers/wasm-runner.ts:115-188`) reads the result region from WASM memory but collapses it to a u32 via `parseLittleEndianU32`. We need the raw bytes for three-way byte-level agreement checks.

**Files:**
- Modify: `tests/helpers/wasm-runner.ts`

- [ ] **Step 1: Add `runWasmNativeBytes` implementation**

Modify `tests/helpers/wasm-runner.ts`. Add below `runWasmNative` (around line 188):

```ts
export interface WasmRunBytesResult {
  /** The raw result bytes from linear memory, or null if execution trapped. */
  bytes: Uint8Array | null;
  /** True if the module trapped. */
  trapped: boolean;
  /** Error message if trapped. */
  error?: string;
}

/**
 * Run a WASM module natively and return the full raw result bytes.
 *
 * Identical to `runWasmNative` but preserves the full byte string instead
 * of collapsing it to a u32.
 */
export async function runWasmNativeBytes(
  wasmBinary: Uint8Array,
  argsHex: string,
): Promise<WasmRunBytesResult> {
  const argsBytes = hexToBytes(argsHex);

  try {
    const module = new WebAssembly.Module(wasmBinary as BufferSource);
    const memory = new WebAssembly.Memory({ initial: 2 });
    const importObject: WebAssembly.Imports = {};

    const moduleImports = WebAssembly.Module.imports(module);
    for (const imp of moduleImports) {
      if (imp.kind === "memory") {
        if (!importObject[imp.module]) importObject[imp.module] = {};
        (importObject[imp.module] as Record<string, unknown>)[imp.name] = memory;
      }
    }

    const instance = new WebAssembly.Instance(module, importObject);
    const mainFn = instance.exports.main as (
      ptr: number,
      len: number,
    ) => bigint | number;
    if (!mainFn) {
      return { bytes: null, trapped: true, error: "No 'main' export found" };
    }

    const mem = (instance.exports.memory as WebAssembly.Memory) ?? memory;
    const memView = new Uint8Array(mem.buffer);
    memView.set(argsBytes, ARGS_OFFSET);

    const result = mainFn(ARGS_OFFSET, argsBytes.length);

    if (typeof result === "bigint") {
      const resultPtr = Number(result & 0xffffffffn);
      const resultLen = Number((result >> 32n) & 0xffffffffn);
      if (resultLen === 0) {
        return { bytes: new Uint8Array(0), trapped: false };
      }
      // Re-acquire view: memory may have grown during execution.
      const resultView = new Uint8Array(mem.buffer);
      const bytes = resultView.slice(resultPtr, resultPtr + resultLen);
      return { bytes, trapped: false };
    } else if (typeof result === "number") {
      // Legacy single-i32: pack as 4 little-endian bytes for consistency.
      const bytes = new Uint8Array(4);
      new DataView(bytes.buffer).setInt32(0, result, true);
      return { bytes, trapped: false };
    }

    return {
      bytes: null,
      trapped: true,
      error: `Unexpected return type: ${typeof result}`,
    };
  } catch (err: any) {
    const msg = err?.message ?? String(err);
    return { bytes: null, trapped: true, error: msg };
  }
}
```

- [ ] **Step 2: Verify the existing wasm-runner tests still pass**

The existing `defineDifferentialSuite` tests import `runWasmForSuite`, not `runWasmNative` directly. We haven't changed behavior — only added a new export. Sanity-check nothing broke:

Run: `cd tests && bun test layer1/`
Expected: PASS (all layer1 tests).

- [ ] **Step 3: Commit**

```bash
git add tests/helpers/wasm-runner.ts
git commit -m "$(cat <<'EOF'
test: add runWasmNativeBytes helper for byte-level differential

Existing runWasmNative collapses to u32 which is inadequate for hash fixtures.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add `@noble/hashes` dep and reference wrapper

**Why:** This is our third-party reference. Installed once, wrapped with an ABI-matching helper so tests can write `ref({ outLen, input })` without repeating the `dkLen` parameter style.

**Files:**
- Modify: `tests/package.json`
- Create: `tests/helpers/blake2b-ref.ts`

- [ ] **Step 1: Install `@noble/hashes`**

```bash
cd tests && bun add --dev @noble/hashes@^1.5.0
```

Verify `tests/package.json` now lists it under `devDependencies` and `tests/bun.lockb` was updated.

- [ ] **Step 2: Write a failing test for the reference wrapper**

Create `tests/helpers/blake2b-ref.test.ts`:

```ts
import { test, expect } from "bun:test";
import { blake2bRef, encodeBlake2bArgs } from "./blake2b-ref";

test("blake2bRef matches RFC 7693 vector for blake2b('abc', 64)", () => {
  const hash = blake2bRef({ outLen: 64, input: new TextEncoder().encode("abc") });
  // RFC 7693 Appendix A worked example output.
  const expected = new Uint8Array([
    0xba, 0x80, 0xa5, 0x3f, 0x98, 0x1c, 0x4d, 0x0d,
    0x6a, 0x27, 0x97, 0xb6, 0x9f, 0x12, 0xf6, 0xe9,
    0x4c, 0x21, 0x2f, 0x14, 0x68, 0x5a, 0xc4, 0xb7,
    0x4b, 0x12, 0xbb, 0x6f, 0xdb, 0xff, 0xa2, 0xd1,
    0x7d, 0x87, 0xc5, 0x39, 0x2a, 0xab, 0x79, 0x2d,
    0xc2, 0x52, 0xd5, 0xde, 0x45, 0x33, 0xcc, 0x95,
    0x18, 0xd3, 0x8a, 0xa8, 0xdb, 0xf1, 0x92, 0x5a,
    0xb9, 0x23, 0x86, 0xed, 0xd4, 0x00, 0x99, 0x23,
  ]);
  expect(hash).toEqual(expected);
});

test("encodeBlake2bArgs produces [out_len:u8][input]", () => {
  const args = encodeBlake2bArgs({ outLen: 32, input: new Uint8Array([0xaa, 0xbb]) });
  expect(args).toEqual(new Uint8Array([32, 0xaa, 0xbb]));
});
```

Run: `cd tests && bun test helpers/blake2b-ref.test.ts`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement the wrapper**

Create `tests/helpers/blake2b-ref.ts`:

```ts
import { blake2b } from "@noble/hashes/blake2b";

export interface Blake2bArgs {
  /** Output length in bytes, 1..=64. */
  outLen: number;
  /** Input to hash. */
  input: Uint8Array;
}

/** Reference blake2b via `@noble/hashes`. Unkeyed, no salt, no personalization. */
export function blake2bRef(args: Blake2bArgs): Uint8Array {
  if (args.outLen < 1 || args.outLen > 64) {
    throw new Error(`out_len out of range: ${args.outLen}`);
  }
  return blake2b(args.input, { dkLen: args.outLen });
}

/** Encode `(outLen, input)` into the WAT entry-point's args bytes: [outLen:u8][input]. */
export function encodeBlake2bArgs(args: Blake2bArgs): Uint8Array {
  const out = new Uint8Array(1 + args.input.length);
  out[0] = args.outLen;
  out.set(args.input, 1);
  return out;
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd tests && bun test helpers/blake2b-ref.test.ts`
Expected: PASS (both tests).

- [ ] **Step 5: Remove the temporary test file**

The reference helper will be exercised indirectly through every blake2b test. Delete the standalone test:

```bash
rm tests/helpers/blake2b-ref.test.ts
```

- [ ] **Step 6: Commit**

```bash
git add tests/package.json tests/bun.lockb tests/helpers/blake2b-ref.ts
git commit -m "$(cat <<'EOF'
test: add @noble/hashes dep and blake2b-ref helper

Wraps @noble/hashes/blake2b with the (outLen, input) shape used by the
upcoming WAT fixture. Exposes encodeBlake2bArgs so tests produce the same
[out_len:u8][input] byte layout that the WAT entry expects.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Write the test file against a stub WAT

**Why:** TDD — write the tests before the WAT, verify the test scaffolding (three-way harness, seeded PRNG, unit vectors) works by running them against a dummy WAT that returns the wrong hash. This isolates test bugs from WAT bugs.

The stub returns `out_len` zero bytes. All tests will fail (correctly) until the real WAT lands in Task 5.

**Files:**
- Create: `tests/fixtures/wat/blake2b.jam.wat` (stub version)
- Create: `tests/layer3/blake2b.test.ts`

- [ ] **Step 1: Create the stub WAT**

Create `tests/fixtures/wat/blake2b.jam.wat` with a stub that validates args and writes zeros:

```wat
;; STUB — will be replaced in Task 5 with the real blake2b implementation.
;; Writes out_len zero bytes to offset 0 and returns (ptr=0, len=out_len).
;; Exists to let the test harness run end-to-end before the algorithm is written.
(module
  (memory (export "memory") 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $out_len i32)
    (local $i i32)

    ;; Read out_len = args[0] (as u8)
    (local.set $out_len (i32.load8_u (local.get $args_ptr)))

    ;; Trap if out_len == 0 or > 64
    (if (i32.or (i32.eqz (local.get $out_len))
                (i32.gt_u (local.get $out_len) (i32.const 64)))
      (then (unreachable)))

    ;; Zero the output buffer at offset 0..out_len
    (local.set $i (i32.const 0))
    (block $exit
      (loop $zero_loop
        (br_if $exit (i32.ge_u (local.get $i) (local.get $out_len)))
        (i32.store8 (local.get $i) (i32.const 0))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $zero_loop)))

    ;; Return (ptr=0) | (out_len << 32)
    (i64.or
      (i64.const 0)
      (i64.shl (i64.extend_i32_u (local.get $out_len)) (i64.const 32)))))
```

- [ ] **Step 2: Verify the stub compiles to a JAM**

Run: `cd tests && bun build.ts`
Expected: Build succeeds. Check that `tests/build/jam/blake2b.jam` exists:

```bash
ls -l tests/build/jam/blake2b.jam
```

If the build fails, fix syntax errors in the WAT and re-run.

- [ ] **Step 3: Write the test file**

Create `tests/layer3/blake2b.test.ts`:

```ts
import { test, expect, describe } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { JAM_DIR, WAT_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";
import { runWasmNativeBytes } from "../helpers/wasm-runner";
import {
  blake2bRef,
  encodeBlake2bArgs,
  bytesToHex,
  type Blake2bArgs,
} from "../helpers/blake2b-ref";

// -----------------------------------------------------------------------------
// Fixture paths
// -----------------------------------------------------------------------------

const JAM_FILE = path.join(JAM_DIR, "blake2b.jam");
const WAT_FILE = path.join(WAT_DIR, "blake2b.jam.wat");

// -----------------------------------------------------------------------------
// WAT -> WASM (cached at module load)
// -----------------------------------------------------------------------------

let wasmBinary: Uint8Array | null = null;

async function getWasm(): Promise<Uint8Array> {
  if (wasmBinary) return wasmBinary;
  const watSource = fs.readFileSync(WAT_FILE, "utf8");
  const wabt = await import("wabt");
  const wabtModule = await wabt.default();
  const parsed = wabtModule.parseWat(WAT_FILE, watSource, {
    multi_value: true,
    mutable_globals: true,
    bulk_memory: true,
    sign_extension: true,
  });
  parsed.validate();
  const { buffer } = parsed.toBinary({});
  wasmBinary = new Uint8Array(buffer);
  return wasmBinary;
}

// -----------------------------------------------------------------------------
// Three-way agreement: PVM == native WASM == @noble/hashes reference
// -----------------------------------------------------------------------------

async function assertBlake2bAgreement(args: Blake2bArgs, expected?: Uint8Array) {
  const argsBytes = encodeBlake2bArgs(args);
  const argsHex = bytesToHex(argsBytes);

  const ref = blake2bRef(args);
  if (expected) {
    expect(bytesToHex(ref)).toBe(bytesToHex(expected));
  }

  const pvm = runJamBytes(JAM_FILE, argsHex);
  expect(bytesToHex(pvm)).toBe(bytesToHex(ref));

  const wasm = await runWasmNativeBytes(await getWasm(), argsHex);
  expect(wasm.trapped).toBe(false);
  expect(bytesToHex(wasm.bytes!)).toBe(bytesToHex(ref));
}

// -----------------------------------------------------------------------------
// Deterministic input generator (repeating pattern — not random) for unit tests.
// Gives us known inputs without depending on the PRNG.
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

describe("blake2b: RFC 7693 vector", () => {
  test("blake2b('abc', 64)", async () => {
    const expected = new Uint8Array([
      0xba, 0x80, 0xa5, 0x3f, 0x98, 0x1c, 0x4d, 0x0d,
      0x6a, 0x27, 0x97, 0xb6, 0x9f, 0x12, 0xf6, 0xe9,
      0x4c, 0x21, 0x2f, 0x14, 0x68, 0x5a, 0xc4, 0xb7,
      0x4b, 0x12, 0xbb, 0x6f, 0xdb, 0xff, 0xa2, 0xd1,
      0x7d, 0x87, 0xc5, 0x39, 0x2a, 0xab, 0x79, 0x2d,
      0xc2, 0x52, 0xd5, 0xde, 0x45, 0x33, 0xcc, 0x95,
      0x18, 0xd3, 0x8a, 0xa8, 0xdb, 0xf1, 0x92, 0x5a,
      0xb9, 0x23, 0x86, 0xed, 0xd4, 0x00, 0x99, 0x23,
    ]);
    await assertBlake2bAgreement(
      { outLen: 64, input: new TextEncoder().encode("abc") },
      expected,
    );
  });
});

describe("blake2b: JAM-relevant (blake2b-256)", () => {
  test("blake2b('', 32)", async () => {
    await assertBlake2bAgreement({ outLen: 32, input: new Uint8Array(0) });
  });
  test("blake2b('abc', 32)", async () => {
    await assertBlake2bAgreement({
      outLen: 32,
      input: new TextEncoder().encode("abc"),
    });
  });
});

describe("blake2b: size edges (out_len=32)", () => {
  for (const len of [0, 1, 127, 128, 129, 255, 256, 257]) {
    test(`input len = ${len}`, async () => {
      await assertBlake2bAgreement({ outLen: 32, input: patternInput(len) });
    });
  }
});

describe("blake2b: output length endpoints", () => {
  test("out_len=1", async () => {
    await assertBlake2bAgreement({ outLen: 1, input: patternInput(17) });
  });
  test("out_len=64", async () => {
    await assertBlake2bAgreement({ outLen: 64, input: patternInput(17) });
  });
});

// -----------------------------------------------------------------------------
// Seeded random differential
// -----------------------------------------------------------------------------

describe("blake2b: seeded random differential", () => {
  const seedHex =
    process.env.BLAKE2B_RANDOM_SEED ?? "0123456789abcdef";
  const count = parseInt(process.env.BLAKE2B_RANDOM_COUNT ?? "50", 10);
  const seed = BigInt("0x" + seedHex);

  test(`${count} random inputs (seed=${seedHex})`, async () => {
    const next = splitmix64(seed);
    for (let i = 0; i < count; i++) {
      const outLen = randInt(next, 1, 64);
      const inputLen = randInt(next, 0, 2048);
      const input = randomBytes(next, inputLen);
      try {
        await assertBlake2bAgreement({ outLen, input });
      } catch (err) {
        console.error(
          `[blake2b random failure] seed=${seedHex} iteration=${i} outLen=${outLen} inputLen=${inputLen}`,
        );
        console.error(`  input_hex=${bytesToHex(input)}`);
        throw err;
      }
    }
  }, 120_000); // bun test timeout: 2 minutes for 50 random inputs
});
```

- [ ] **Step 4: Verify tests RUN (and fail correctly, since the WAT is a stub)**

Run: `cd tests && bun test layer3/blake2b.test.ts`
Expected: all tests in the suite FAIL with hash mismatch (the stub returns zeros; reference returns real hashes). **Crucially:** the failures should be `expect(...).toBe(...)` assertion failures on hash bytes — NOT runtime errors about missing helpers or WASM parse errors. If you see runtime errors, fix the scaffolding before moving on.

- [ ] **Step 5: Commit the stub + test file**

```bash
git add tests/fixtures/wat/blake2b.jam.wat tests/layer3/blake2b.test.ts
git commit -m "$(cat <<'EOF'
test: add blake2b test scaffolding and WAT stub

Three-way agreement harness (PVM == native WASM == @noble/hashes), unit vectors,
seeded random differential. WAT is a stub returning zeros — tests fail
correctly, waiting on the real algorithm in the next commit.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Implement blake2b in WAT

**Why:** Now the real algorithm. This is one big block of careful work — blake2b has no useful partial-correctness intermediate state, so the development loop is "write, run tests, debug until green." The three-way harness built in Task 4 is the debugging tool.

**Files:**
- Modify: `tests/fixtures/wat/blake2b.jam.wat` (replace stub with full implementation)

- [ ] **Step 1: Replace the stub with the full implementation**

Replace the entire contents of `tests/fixtures/wat/blake2b.jam.wat` with the implementation below.

```wat
;; blake2b, unkeyed, variable output length 1..=64 (RFC 7693).
;;
;; Entry: main(args_ptr: i32, args_len: i32) -> i64
;;   args = [out_len: u8][input: bytes]
;;   returns (out_ptr: i32) | ((out_len: i32) << 32)
;;
;; WASM memory layout (all offsets WASM-relative):
;;   0x000..0x03F  output hash buffer (64 bytes)
;;   0x040..0x07F  h[8] state (mutable, 8 x i64 LE)
;;   0x080..0x0BF  IV[8] constants (data segment, 8 x i64 LE)
;;   0x0C0..0x13F  v[16] working state (mutable, 16 x i64 LE)
;;   0x140..0x1BF  m[16] current message block (mutable, 16 x i64 LE)
;;   0x1C0..0x25F  sigma[10][16] permutation table (data segment, u8)
;;   0x260..0x267  t counter (i64)

(module
  (memory (export "memory") 1)

  ;; IV at 0x80 (64 bytes, 8 x i64 LE)
  (data (i32.const 0x080)
    "\08\c9\bc\f3\67\e6\09\6a"  ;; IV[0] = 0x6a09e667f3bcc908
    "\3b\a7\ca\84\85\ae\67\bb"  ;; IV[1] = 0xbb67ae8584caa73b
    "\2b\f8\94\fe\72\f3\6e\3c"  ;; IV[2] = 0x3c6ef372fe94f82b
    "\f1\36\1d\5f\3a\f5\4f\a5"  ;; IV[3] = 0xa54ff53a5f1d36f1
    "\d1\82\e6\ad\7f\52\0e\51"  ;; IV[4] = 0x510e527fade682d1
    "\1f\6c\3e\2b\8c\68\05\9b"  ;; IV[5] = 0x9b05688c2b3e6c1f
    "\6b\bd\41\fb\ab\d9\83\1f"  ;; IV[6] = 0x1f83d9abfb41bd6b
    "\79\21\7e\13\19\cd\e0\5b") ;; IV[7] = 0x5be0cd19137e2179

  ;; Sigma at 0x1c0 (160 bytes, 10 rows x 16 u8)
  (data (i32.const 0x1c0)
    "\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f"  ;; row 0
    "\0e\0a\04\08\09\0f\0d\06\01\0c\00\02\0b\07\05\03"  ;; row 1
    "\0b\08\0c\00\05\02\0f\0d\0a\0e\03\06\07\01\09\04"  ;; row 2
    "\07\09\03\01\0d\0c\0b\0e\02\06\05\0a\04\00\0f\08"  ;; row 3
    "\09\00\05\07\02\04\0a\0f\0e\01\0b\0c\06\08\03\0d"  ;; row 4
    "\02\0c\06\0a\00\0b\08\03\04\0d\07\05\0f\0e\01\09"  ;; row 5
    "\0c\05\01\0f\0e\0d\04\0a\00\07\06\03\09\02\08\0b"  ;; row 6
    "\0d\0b\07\0e\0c\01\03\09\05\00\0f\04\08\06\02\0a"  ;; row 7
    "\06\0f\0e\09\0b\03\00\08\0c\02\0d\07\01\04\0a\05"  ;; row 8
    "\0a\02\08\04\07\06\01\05\0f\0b\09\0e\03\0c\0d\00") ;; row 9

  ;; --- Helper: G mixing function ---
  ;;
  ;; Takes four v-indices (0..15) and two m-indices (0..15). Mixes
  ;; v[ia], v[ib], v[ic], v[id] using m[mx] and m[my].
  ;; Loads/stores happen via explicit byte offsets = index * 8.
  (func $g (param $ia i32) (param $ib i32) (param $ic i32) (param $id i32)
          (param $mx i32) (param $my i32)
    (local $va i64) (local $vb i64) (local $vc i64) (local $vd i64)
    (local $mxw i64) (local $myw i64)
    (local $pa i32) (local $pb i32) (local $pc i32) (local $pd i32)

    ;; byte addresses into v and m
    (local.set $pa (i32.add (i32.const 0x0c0) (i32.shl (local.get $ia) (i32.const 3))))
    (local.set $pb (i32.add (i32.const 0x0c0) (i32.shl (local.get $ib) (i32.const 3))))
    (local.set $pc (i32.add (i32.const 0x0c0) (i32.shl (local.get $ic) (i32.const 3))))
    (local.set $pd (i32.add (i32.const 0x0c0) (i32.shl (local.get $id) (i32.const 3))))

    (local.set $va (i64.load (local.get $pa)))
    (local.set $vb (i64.load (local.get $pb)))
    (local.set $vc (i64.load (local.get $pc)))
    (local.set $vd (i64.load (local.get $pd)))

    (local.set $mxw
      (i64.load (i32.add (i32.const 0x140) (i32.shl (local.get $mx) (i32.const 3)))))
    (local.set $myw
      (i64.load (i32.add (i32.const 0x140) (i32.shl (local.get $my) (i32.const 3)))))

    ;; va = va + vb + mxw
    (local.set $va (i64.add (i64.add (local.get $va) (local.get $vb)) (local.get $mxw)))
    ;; vd = rotr(vd ^ va, 32)
    (local.set $vd (i64.rotr (i64.xor (local.get $vd) (local.get $va)) (i64.const 32)))
    ;; vc = vc + vd
    (local.set $vc (i64.add (local.get $vc) (local.get $vd)))
    ;; vb = rotr(vb ^ vc, 24)
    (local.set $vb (i64.rotr (i64.xor (local.get $vb) (local.get $vc)) (i64.const 24)))
    ;; va = va + vb + myw
    (local.set $va (i64.add (i64.add (local.get $va) (local.get $vb)) (local.get $myw)))
    ;; vd = rotr(vd ^ va, 16)
    (local.set $vd (i64.rotr (i64.xor (local.get $vd) (local.get $va)) (i64.const 16)))
    ;; vc = vc + vd
    (local.set $vc (i64.add (local.get $vc) (local.get $vd)))
    ;; vb = rotr(vb ^ vc, 63)
    (local.set $vb (i64.rotr (i64.xor (local.get $vb) (local.get $vc)) (i64.const 63)))

    (i64.store (local.get $pa) (local.get $va))
    (i64.store (local.get $pb) (local.get $vb))
    (i64.store (local.get $pc) (local.get $vc))
    (i64.store (local.get $pd) (local.get $vd)))

  ;; --- Helper: compress (F function) ---
  ;;
  ;; Consumes m[16] already filled, t counter at 0x260, and the last flag
  ;; passed as a parameter. Mutates h[].
  (func $compress (param $last i32)
    (local $r i32)         ;; round index 0..11
    (local $sigma_base i32) ;; pointer into sigma[round % 10]
    (local $t i64)

    ;; v[0..7] = h[0..7]
    (i64.store offset=0    (i32.const 0x0c0) (i64.load offset=0    (i32.const 0x040)))
    (i64.store offset=8    (i32.const 0x0c0) (i64.load offset=8    (i32.const 0x040)))
    (i64.store offset=16   (i32.const 0x0c0) (i64.load offset=16   (i32.const 0x040)))
    (i64.store offset=24   (i32.const 0x0c0) (i64.load offset=24   (i32.const 0x040)))
    (i64.store offset=32   (i32.const 0x0c0) (i64.load offset=32   (i32.const 0x040)))
    (i64.store offset=40   (i32.const 0x0c0) (i64.load offset=40   (i32.const 0x040)))
    (i64.store offset=48   (i32.const 0x0c0) (i64.load offset=48   (i32.const 0x040)))
    (i64.store offset=56   (i32.const 0x0c0) (i64.load offset=56   (i32.const 0x040)))

    ;; v[8..15] = IV[0..7]
    (i64.store offset=64   (i32.const 0x0c0) (i64.load offset=0    (i32.const 0x080)))
    (i64.store offset=72   (i32.const 0x0c0) (i64.load offset=8    (i32.const 0x080)))
    (i64.store offset=80   (i32.const 0x0c0) (i64.load offset=16   (i32.const 0x080)))
    (i64.store offset=88   (i32.const 0x0c0) (i64.load offset=24   (i32.const 0x080)))
    (i64.store offset=96   (i32.const 0x0c0) (i64.load offset=32   (i32.const 0x080)))
    (i64.store offset=104  (i32.const 0x0c0) (i64.load offset=40   (i32.const 0x080)))
    (i64.store offset=112  (i32.const 0x0c0) (i64.load offset=48   (i32.const 0x080)))
    (i64.store offset=120  (i32.const 0x0c0) (i64.load offset=56   (i32.const 0x080)))

    ;; v[12] ^= t_low
    (local.set $t (i64.load (i32.const 0x260)))
    (i64.store offset=96 (i32.const 0x0c0)
      (i64.xor (i64.load offset=96 (i32.const 0x0c0)) (local.get $t)))
    ;; v[13] ^= t_high (always 0 for our capped input size; XOR is structurally correct)
    (i64.store offset=104 (i32.const 0x0c0)
      (i64.xor (i64.load offset=104 (i32.const 0x0c0)) (i64.const 0)))

    ;; v[14] ^= ~0 if last
    (if (local.get $last)
      (then
        (i64.store offset=112 (i32.const 0x0c0)
          (i64.xor (i64.load offset=112 (i32.const 0x0c0))
                   (i64.const -1)))))

    ;; 12 rounds
    (local.set $r (i32.const 0))
    (block $rounds_exit
      (loop $rounds
        (br_if $rounds_exit (i32.ge_u (local.get $r) (i32.const 12)))

        ;; sigma_base = 0x1c0 + (r % 10) * 16
        (local.set $sigma_base
          (i32.add (i32.const 0x1c0)
            (i32.shl (i32.rem_u (local.get $r) (i32.const 10)) (i32.const 4))))

        ;; Column mixes: G(0,4,8,12, s[0],s[1]), G(1,5,9,13, s[2],s[3]), ...
        (call $g (i32.const 0) (i32.const 4) (i32.const 8)  (i32.const 12)
                 (i32.load8_u offset=0  (local.get $sigma_base))
                 (i32.load8_u offset=1  (local.get $sigma_base)))
        (call $g (i32.const 1) (i32.const 5) (i32.const 9)  (i32.const 13)
                 (i32.load8_u offset=2  (local.get $sigma_base))
                 (i32.load8_u offset=3  (local.get $sigma_base)))
        (call $g (i32.const 2) (i32.const 6) (i32.const 10) (i32.const 14)
                 (i32.load8_u offset=4  (local.get $sigma_base))
                 (i32.load8_u offset=5  (local.get $sigma_base)))
        (call $g (i32.const 3) (i32.const 7) (i32.const 11) (i32.const 15)
                 (i32.load8_u offset=6  (local.get $sigma_base))
                 (i32.load8_u offset=7  (local.get $sigma_base)))

        ;; Diagonal mixes
        (call $g (i32.const 0) (i32.const 5) (i32.const 10) (i32.const 15)
                 (i32.load8_u offset=8  (local.get $sigma_base))
                 (i32.load8_u offset=9  (local.get $sigma_base)))
        (call $g (i32.const 1) (i32.const 6) (i32.const 11) (i32.const 12)
                 (i32.load8_u offset=10 (local.get $sigma_base))
                 (i32.load8_u offset=11 (local.get $sigma_base)))
        (call $g (i32.const 2) (i32.const 7) (i32.const 8)  (i32.const 13)
                 (i32.load8_u offset=12 (local.get $sigma_base))
                 (i32.load8_u offset=13 (local.get $sigma_base)))
        (call $g (i32.const 3) (i32.const 4) (i32.const 9)  (i32.const 14)
                 (i32.load8_u offset=14 (local.get $sigma_base))
                 (i32.load8_u offset=15 (local.get $sigma_base)))

        (local.set $r (i32.add (local.get $r) (i32.const 1)))
        (br $rounds)))

    ;; h[i] ^= v[i] ^ v[i+8] for i in 0..7
    (i64.store offset=0  (i32.const 0x040)
      (i64.xor (i64.load offset=0  (i32.const 0x040))
               (i64.xor (i64.load offset=0  (i32.const 0x0c0))
                        (i64.load offset=64 (i32.const 0x0c0)))))
    (i64.store offset=8  (i32.const 0x040)
      (i64.xor (i64.load offset=8  (i32.const 0x040))
               (i64.xor (i64.load offset=8  (i32.const 0x0c0))
                        (i64.load offset=72 (i32.const 0x0c0)))))
    (i64.store offset=16 (i32.const 0x040)
      (i64.xor (i64.load offset=16 (i32.const 0x040))
               (i64.xor (i64.load offset=16 (i32.const 0x0c0))
                        (i64.load offset=80 (i32.const 0x0c0)))))
    (i64.store offset=24 (i32.const 0x040)
      (i64.xor (i64.load offset=24 (i32.const 0x040))
               (i64.xor (i64.load offset=24 (i32.const 0x0c0))
                        (i64.load offset=88 (i32.const 0x0c0)))))
    (i64.store offset=32 (i32.const 0x040)
      (i64.xor (i64.load offset=32 (i32.const 0x040))
               (i64.xor (i64.load offset=32 (i32.const 0x0c0))
                        (i64.load offset=96 (i32.const 0x0c0)))))
    (i64.store offset=40 (i32.const 0x040)
      (i64.xor (i64.load offset=40 (i32.const 0x040))
               (i64.xor (i64.load offset=40  (i32.const 0x0c0))
                        (i64.load offset=104 (i32.const 0x0c0)))))
    (i64.store offset=48 (i32.const 0x040)
      (i64.xor (i64.load offset=48 (i32.const 0x040))
               (i64.xor (i64.load offset=48  (i32.const 0x0c0))
                        (i64.load offset=112 (i32.const 0x0c0)))))
    (i64.store offset=56 (i32.const 0x040)
      (i64.xor (i64.load offset=56 (i32.const 0x040))
               (i64.xor (i64.load offset=56  (i32.const 0x0c0))
                        (i64.load offset=120 (i32.const 0x0c0))))))

  ;; --- main ---
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $out_len i32)
    (local $data_ptr i32)
    (local $remaining i32)   ;; bytes of input not yet consumed
    (local $i i32)           ;; generic loop counter
    (local $t i64)           ;; cumulative input bytes processed

    ;; out_len = args[0]
    (local.set $out_len (i32.load8_u (local.get $args_ptr)))
    (if (i32.or (i32.eqz (local.get $out_len))
                (i32.gt_u (local.get $out_len) (i32.const 64)))
      (then (unreachable)))

    ;; data_ptr = args_ptr + 1; remaining = args_len - 1
    (local.set $data_ptr (i32.add (local.get $args_ptr) (i32.const 1)))
    (local.set $remaining (i32.sub (local.get $args_len) (i32.const 1)))

    ;; h[0..7] = IV[0..7]
    (i64.store offset=0  (i32.const 0x040) (i64.load offset=0  (i32.const 0x080)))
    (i64.store offset=8  (i32.const 0x040) (i64.load offset=8  (i32.const 0x080)))
    (i64.store offset=16 (i32.const 0x040) (i64.load offset=16 (i32.const 0x080)))
    (i64.store offset=24 (i32.const 0x040) (i64.load offset=24 (i32.const 0x080)))
    (i64.store offset=32 (i32.const 0x040) (i64.load offset=32 (i32.const 0x080)))
    (i64.store offset=40 (i32.const 0x040) (i64.load offset=40 (i32.const 0x080)))
    (i64.store offset=48 (i32.const 0x040) (i64.load offset=48 (i32.const 0x080)))
    (i64.store offset=56 (i32.const 0x040) (i64.load offset=56 (i32.const 0x080)))

    ;; Apply parameter block: h[0] ^= 0x0101_0000 ^ out_len
    ;; (fanout=1, depth=1, node_depth=0, inner_len=0, key_len=0, digest_len=out_len)
    (i64.store offset=0 (i32.const 0x040)
      (i64.xor
        (i64.load offset=0 (i32.const 0x040))
        (i64.xor
          (i64.const 0x01010000)
          (i64.extend_i32_u (local.get $out_len)))))

    ;; t = 0
    (i64.store (i32.const 0x260) (i64.const 0))

    ;; Process non-final full 128-byte blocks: while remaining > 128
    (block $stream_exit
      (loop $stream
        (br_if $stream_exit (i32.le_u (local.get $remaining) (i32.const 128)))

        ;; Copy 128 bytes from data_ptr into m (m is 16 x i64 = 128 bytes).
        ;; Use i64.load with 1-byte alignment (WAT defaults to natural alignment
        ;; but the compiler tolerates unaligned loads via sub-byte load lowering).
        (i64.store offset=0   (i32.const 0x140)
          (i64.load align=1 offset=0   (local.get $data_ptr)))
        (i64.store offset=8   (i32.const 0x140)
          (i64.load align=1 offset=8   (local.get $data_ptr)))
        (i64.store offset=16  (i32.const 0x140)
          (i64.load align=1 offset=16  (local.get $data_ptr)))
        (i64.store offset=24  (i32.const 0x140)
          (i64.load align=1 offset=24  (local.get $data_ptr)))
        (i64.store offset=32  (i32.const 0x140)
          (i64.load align=1 offset=32  (local.get $data_ptr)))
        (i64.store offset=40  (i32.const 0x140)
          (i64.load align=1 offset=40  (local.get $data_ptr)))
        (i64.store offset=48  (i32.const 0x140)
          (i64.load align=1 offset=48  (local.get $data_ptr)))
        (i64.store offset=56  (i32.const 0x140)
          (i64.load align=1 offset=56  (local.get $data_ptr)))
        (i64.store offset=64  (i32.const 0x140)
          (i64.load align=1 offset=64  (local.get $data_ptr)))
        (i64.store offset=72  (i32.const 0x140)
          (i64.load align=1 offset=72  (local.get $data_ptr)))
        (i64.store offset=80  (i32.const 0x140)
          (i64.load align=1 offset=80  (local.get $data_ptr)))
        (i64.store offset=88  (i32.const 0x140)
          (i64.load align=1 offset=88  (local.get $data_ptr)))
        (i64.store offset=96  (i32.const 0x140)
          (i64.load align=1 offset=96  (local.get $data_ptr)))
        (i64.store offset=104 (i32.const 0x140)
          (i64.load align=1 offset=104 (local.get $data_ptr)))
        (i64.store offset=112 (i32.const 0x140)
          (i64.load align=1 offset=112 (local.get $data_ptr)))
        (i64.store offset=120 (i32.const 0x140)
          (i64.load align=1 offset=120 (local.get $data_ptr)))

        ;; t += 128
        (i64.store (i32.const 0x260)
          (i64.add (i64.load (i32.const 0x260)) (i64.const 128)))

        ;; compress(last=0)
        (call $compress (i32.const 0))

        (local.set $data_ptr (i32.add (local.get $data_ptr) (i32.const 128)))
        (local.set $remaining (i32.sub (local.get $remaining) (i32.const 128)))
        (br $stream)))

    ;; Final block: remaining is in 0..=128. Zero m[], then copy remaining bytes.
    (local.set $i (i32.const 0))
    (block $zero_exit
      (loop $zero_m
        (br_if $zero_exit (i32.ge_u (local.get $i) (i32.const 16)))
        (i64.store
          (i32.add (i32.const 0x140) (i32.shl (local.get $i) (i32.const 3)))
          (i64.const 0))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $zero_m)))

    (local.set $i (i32.const 0))
    (block $copy_exit
      (loop $copy_final
        (br_if $copy_exit (i32.ge_u (local.get $i) (local.get $remaining)))
        (i32.store8
          (i32.add (i32.const 0x140) (local.get $i))
          (i32.load8_u (i32.add (local.get $data_ptr) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $copy_final)))

    ;; t += remaining
    (i64.store (i32.const 0x260)
      (i64.add (i64.load (i32.const 0x260)) (i64.extend_i32_u (local.get $remaining))))

    ;; compress(last=1)
    (call $compress (i32.const 1))

    ;; Copy h[0..out_len] bytes to output at offset 0 (byte-level to handle
    ;; non-multiple-of-8 out_len and to respect little-endian h word encoding).
    (local.set $i (i32.const 0))
    (block $out_exit
      (loop $out_copy
        (br_if $out_exit (i32.ge_u (local.get $i) (local.get $out_len)))
        (i32.store8
          (local.get $i)
          (i32.load8_u (i32.add (i32.const 0x040) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $out_copy)))

    ;; Return (0 | (out_len << 32))
    (i64.shl (i64.extend_i32_u (local.get $out_len)) (i64.const 32))))
```

- [ ] **Step 2: Rebuild and run tests**

```bash
cd tests
rm -f build/wasm/*.wasm  # per AGENTS.md, clear stale AS caches (no effect on WAT but per guidance)
bun build.ts
bun test layer3/blake2b.test.ts
```

Expected: ALL tests in `blake2b.test.ts` PASS (RFC vector, JAM-relevant, size edges, output-length endpoints, seeded random 50 iterations).

**If tests fail:** the first-failing test name + the logged `input_hex` (for random failures) pinpoints the input. Compare:
- **PVM vs. native WASM** identical but both wrong → algorithm bug in the WAT
- **Native WASM == reference but PVM differs** → compiler regression (unlikely, but possible). Report with the failing input.

Common mistakes (worth checking first):
- IV bytes wrong byte-order (must be little-endian)
- Sigma row indexing: `r % 10` for rounds 10 and 11
- `i64.rotr` constants: **32, 24, 16, 63** (not 32, 24, 16, 7 — that's the 32-bit blake2s)
- Parameter block XOR: `0x01010000 ^ out_len` (not `0x0101_0020` or other constant — the low byte IS out_len)
- Last-block detection: the last compression call uses `last=1` even for empty input (one all-zero block)
- Empty input edge case: `remaining` starts at 0 (since `args_len=1` for the 1-byte out_len prefix); the `stream` loop's `le_u 128` condition immediately exits, the final-block path zeros m and compresses once with `last=1` and `t=0`.

- [ ] **Step 3: Stress-run at 1000 iterations**

Per the spec's success criteria:

```bash
cd tests && BLAKE2B_RANDOM_COUNT=1000 bun test layer3/blake2b.test.ts -t "random"
```

Expected: PASS. Note gas/time usage — if any iteration hits the 2-minute test timeout, the bun timeout in the test file may need tuning (currently `120_000` ms).

- [ ] **Step 4: Commit**

```bash
git add tests/fixtures/wat/blake2b.jam.wat
git commit -m "$(cat <<'EOF'
feat: hand-crafted blake2b WAT example

RFC 7693 blake2b, unkeyed, variable output length 1..=64. Validated against
@noble/hashes and native WebAssembly with 1000 seeded-random inputs plus
RFC canonical vector and size-edge cases.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Run full test suite and benchmark

**Why:** Confirm no regressions in other fixtures, measure gas/JAM-size impact.

- [ ] **Step 1: Full suite**

```bash
cd tests && bun run test
```

Expected: PASS (all existing tests + new blake2b tests).

- [ ] **Step 2: Benchmark**

From the repo root:

```bash
./tests/utils/benchmark.sh --base main --current td-blake2b-wat
```

Save the output — it goes into the PR description per the project's PR Description Policy.

If `benchmark.sh` fails because `main` hasn't been built, check the script's expectations (`./tests/utils/benchmark.sh --help` or read the script). Worst case, run it with explicit branch args per the existing convention.

---

## Task 7: Documentation updates

**Files:**
- Modify: `AGENTS.md`

- [ ] **Step 1: Update the "Where to Look" table**

Open `AGENTS.md`. Find the "Where to Look" table. Add these rows, keeping alphabetical/logical order consistent with neighboring rows:

```markdown
| Add a hash/byte-processing example | `tests/fixtures/wat/blake2b.jam.wat` | Hand-crafted blake2b (RFC 7693), unkeyed, variable output length |
| Add a byte-level test | `tests/helpers/run.ts` (`runJamBytes`) + `tests/helpers/wasm-runner.ts` (`runWasmNativeBytes`) | Byte-returning variants of `runJam`/`runWasmNative` for non-u32 outputs |
```

- [ ] **Step 2: Commit**

```bash
git add AGENTS.md
git commit -m "$(cat <<'EOF'
docs: note blake2b fixture and byte-returning test helpers

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Open PR

**Why:** Per the project's PR Description Policy: every PR must include a benchmark comparison table.

- [ ] **Step 1: Push branch and open PR**

```bash
git push -u origin td-blake2b-wat
gh pr create --title "feat: hand-crafted blake2b WAT example with differential tests" --body "$(cat <<'EOF'
## Summary

- Adds a hand-crafted RFC 7693 blake2b WAT fixture (`tests/fixtures/wat/blake2b.jam.wat`), unkeyed, variable output length 1..=64.
- Adds unit tests (RFC canonical vector, blake2b-256 for JAM use cases, block-boundary edges) and seeded-random differential tests (default 50 iterations, `BLAKE2B_RANDOM_COUNT=1000` stress-validated locally).
- Each test checks three-way agreement: PVM output == native WASM output == `@noble/hashes` reference, byte-for-byte.
- Adds `runJamBytes` and `runWasmNativeBytes` helpers — existing `runJam`/`runWasmNative` collapse results to u32.

## Design

`docs/superpowers/specs/2026-04-20-blake2b-wat-example-design.md`

## Benchmark

<paste output of `./tests/utils/benchmark.sh --base main --current td-blake2b-wat` here>

## Test plan

- [ ] `bun run test` passes
- [ ] `BLAKE2B_RANDOM_COUNT=1000 bun test layer3/blake2b.test.ts -t "random"` passes
- [ ] Benchmark table populated above

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Then edit the PR body with the actual benchmark output from Task 6 Step 2.

---

## Self-review summary

**Spec coverage:** Every section of the spec maps to a task:

- Goal, non-goals, files → Task structure
- WAT module / memory layout / algorithm → Task 5
- ABI → Task 4 stub + Task 5 real implementation
- Unit tests (RFC, JAM, size edges, out_len endpoints) → Task 4 scaffolding
- Differential test (seeded, env-configurable) → Task 4
- Three-way agreement check → `assertBlake2bAgreement` in Task 4
- Gas budget → `runJamBytes` has optional `gas` override (Task 1 Step 3)
- Documentation updates → Task 7
- Benchmark → Task 6 Step 2 / Task 8

**Placeholder scan:** No TBDs / TODOs / vague "handle errors" instructions. All code blocks are complete. Every file path is absolute relative to repo root.

**Type consistency:** `Blake2bArgs` / `blake2bRef` / `encodeBlake2bArgs` / `runJamBytes` / `runWasmNativeBytes` / `assertBlake2bAgreement` signatures are consistent across Tasks 1–5.
