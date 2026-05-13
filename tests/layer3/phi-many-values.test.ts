// Runtime verification for the slot-based phi-copy parallel-move resolver
// (issue #219).
//
// The Rust-side unit tests in `crates/wasm-pvm/tests/phi_many_values.rs`
// prove that previously-bailing programs now *compile*. These tests run
// the produced JAM through anan-as and assert the post-merge / post-loop
// values are correct — they would fail if the resolver misordered a
// cycle or dropped a copy, even when the JAM still appears valid.
//
// Each test reads a 4-byte little-endian i32 input from `args_ptr` and
// returns a packed (ptr=0, len=N) i64; `runJamBytes` parses the
// resulting N bytes back out of anan-as.

import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/** Encode an i32 as a 4-byte little-endian hex string (no `0x`). */
function i32LeHex(value: number): string {
  return [
    value & 0xff,
    (value >>> 8) & 0xff,
    (value >>> 16) & 0xff,
    (value >>> 24) & 0xff,
  ]
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/** Decode an array of 6 (or 8) little-endian u64s out of the result bytes. */
function decodeU64s(bytes: Uint8Array, count: number): bigint[] {
  expect(bytes.length).toBeGreaterThanOrEqual(count * 8);
  const out: bigint[] = [];
  for (let i = 0; i < count; i++) {
    let v = 0n;
    for (let b = 0; b < 8; b++) {
      v |= BigInt(bytes[i * 8 + b]) << BigInt(b * 8);
    }
    out.push(v);
  }
  return out;
}

/** Bigint version of the expected 6-element left-rotation. */
function rotateLeft(values: bigint[], n: number): bigint[] {
  const k = ((n % values.length) + values.length) % values.length;
  return [...values.slice(k), ...values.slice(0, k)];
}

// ----------------------------------------------------------------------------
// Fixture 1: a single 6-cycle on the back-edge.
// Six locals rotate left by one each iteration; the back-edge phis form
// one length-6 permutation cycle.
// ----------------------------------------------------------------------------

describe("phi-cycle-rotation", () => {
  const JAM = path.join(JAM_DIR, "phi-cycle-rotation.jam");
  const initial = [1n, 2n, 3n, 4n, 5n, 6n];

  // N=0 — loop body never runs; the resolver's invariants must still hold
  // when the back-edge phi copies are emitted but the loop is degenerate.
  test("N=0: no rotation", () => {
    const bytes = runJamBytes(JAM, i32LeHex(0));
    expect(decodeU64s(bytes, 6)).toEqual(initial);
  });

  test("N=1: single rotation", () => {
    const bytes = runJamBytes(JAM, i32LeHex(1));
    expect(decodeU64s(bytes, 6)).toEqual(rotateLeft(initial, 1));
  });

  test("N=3: half rotation", () => {
    const bytes = runJamBytes(JAM, i32LeHex(3));
    expect(decodeU64s(bytes, 6)).toEqual(rotateLeft(initial, 3));
  });

  // N=6 brings the values back to their original positions; this is the
  // strongest cycle-correctness check because every value must follow the
  // full cycle and land back where it started.
  test("N=6: full rotation returns to start", () => {
    const bytes = runJamBytes(JAM, i32LeHex(6));
    expect(decodeU64s(bytes, 6)).toEqual(initial);
  });

  test("N=13: 13 mod 6 == 1", () => {
    const bytes = runJamBytes(JAM, i32LeHex(13));
    expect(decodeU64s(bytes, 6)).toEqual(rotateLeft(initial, 13));
  });
});

// ----------------------------------------------------------------------------
// Fixture 2: three disjoint 2-cycles on the back-edge.
// Each iteration applies three independent swaps (a/b, c/d, e/f). The
// resolver has to extract all three cycles rather than collapsing them
// into a single chain.
// ----------------------------------------------------------------------------

describe("phi-cycle-swaps", () => {
  const JAM = path.join(JAM_DIR, "phi-cycle-swaps.jam");
  const identity = [1n, 2n, 3n, 4n, 5n, 6n];
  const swapped = [2n, 1n, 4n, 3n, 6n, 5n];

  test("N=0: no swap", () => {
    const bytes = runJamBytes(JAM, i32LeHex(0));
    expect(decodeU64s(bytes, 6)).toEqual(identity);
  });

  test("N=1: all three pairs swapped", () => {
    const bytes = runJamBytes(JAM, i32LeHex(1));
    expect(decodeU64s(bytes, 6)).toEqual(swapped);
  });

  test("N=2: swaps cancel", () => {
    const bytes = runJamBytes(JAM, i32LeHex(2));
    expect(decodeU64s(bytes, 6)).toEqual(identity);
  });

  test("N=7: parity wins", () => {
    const bytes = runJamBytes(JAM, i32LeHex(7));
    expect(decodeU64s(bytes, 6)).toEqual(swapped);
  });
});

// ----------------------------------------------------------------------------
// Fixture 3: 8 phis at an if/else merge. The "then" arm uses runtime-
// derived values (so LLVM cannot fold the merge) and the "else" arm uses
// constants (exercising the resolver's constant-copy phase).
// ----------------------------------------------------------------------------

describe("phi-if-else-merge", () => {
  const JAM = path.join(JAM_DIR, "phi-if-else-merge.jam");

  test("sel=0 picks the constant arm", () => {
    const bytes = runJamBytes(JAM, i32LeHex(0));
    expect(decodeU64s(bytes, 8)).toEqual([
      101n,
      102n,
      103n,
      104n,
      105n,
      106n,
      107n,
      108n,
    ]);
  });

  test("sel=5 picks the runtime arm", () => {
    const bytes = runJamBytes(JAM, i32LeHex(5));
    expect(decodeU64s(bytes, 8)).toEqual([
      6n, // sel + 1
      7n,
      8n,
      9n,
      10n,
      11n,
      12n,
      13n,
    ]);
  });

  test("sel=42 picks the runtime arm", () => {
    const bytes = runJamBytes(JAM, i32LeHex(42));
    expect(decodeU64s(bytes, 8)).toEqual([
      43n,
      44n,
      45n,
      46n,
      47n,
      48n,
      49n,
      50n,
    ]);
  });
});
