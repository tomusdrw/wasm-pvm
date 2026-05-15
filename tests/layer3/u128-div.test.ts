import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";

// -----------------------------------------------------------------------------
// End-to-end test for __udivti3 b_hi specialization.
//
// Fast path (a_hi == 0 && b_hi == 0): native u64/u64 divide, quotient_hi = 0.
// Slow path: forwards to a stub `specialized_div_rem` that writes sentinel
//            values (0xDEADBEF0 / 0xDEADBEF1) — proves the stack-frame dance
//            and 16-byte copy in the synthesized slow path work end-to-end.
// -----------------------------------------------------------------------------

const JAM_FILE = path.join(JAM_DIR, "u128-div.jam");

const MASK_64 = (1n << 64n) - 1n;

function encodeArgs(a_lo: bigint, a_hi: bigint, b_lo: bigint, b_hi: bigint): string {
  const buf = new Uint8Array(32);
  const view = new DataView(buf.buffer);
  view.setBigUint64(0, a_lo & MASK_64, true);
  view.setBigUint64(8, a_hi & MASK_64, true);
  view.setBigUint64(16, b_lo & MASK_64, true);
  view.setBigUint64(24, b_hi & MASK_64, true);
  return Array.from(buf)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function decodeResult(bytes: Uint8Array): { lo: bigint; hi: bigint } {
  expect(bytes.length).toBe(16);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return {
    lo: view.getBigUint64(0, true),
    hi: view.getBigUint64(8, true),
  };
}

// Sentinel values our stub specialized_div_rem writes; see u128-div.jam.wat.
const SLOW_PATH_QLO = 0xdeadbef0n;
const SLOW_PATH_QHI = 0xdeadbef1n;

describe("__udivti3 fast path (a_hi == 0 && b_hi == 0)", () => {
  // The synthesized fast path emits `udiv i64`, which our backend gates
  // with `emit_wasm_div_zero_trap` before the actual `DivU64`. anan-as
  // reports a trap as `Status: PANIC` with no result bytes (it still
  // exits the process with code 0, so `runJamBytes` does not throw —
  // hence the empty-result assertion). This guards against future
  // codegen changes silently turning div-by-zero into undefined data.
  test("u64 / 0 traps (no result returned)", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(17n, 0n, 0n, 0n));
    expect(bytes.length).toBe(0);
  });

  test("u64 / u64 exact division", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(15n, 0n, 3n, 0n));
    const { lo, hi } = decodeResult(bytes);
    expect(lo).toBe(5n);
    expect(hi).toBe(0n);
  });

  test("u64 / u64 truncation", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(17n, 0n, 5n, 0n));
    expect(decodeResult(bytes)).toEqual({ lo: 3n, hi: 0n });
  });

  test("u64 / u64 with large values", () => {
    const a = MASK_64 - 7n;
    const b = 1000003n;
    const bytes = runJamBytes(JAM_FILE, encodeArgs(a, 0n, b, 0n));
    expect(decodeResult(bytes)).toEqual({ lo: a / b, hi: 0n });
  });

  test("a < b yields zero quotient", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(7n, 0n, 100n, 0n));
    expect(decodeResult(bytes)).toEqual({ lo: 0n, hi: 0n });
  });

  test("a == b yields one", () => {
    const a = 0xcafef00dcafef00dn;
    const bytes = runJamBytes(JAM_FILE, encodeArgs(a, 0n, a, 0n));
    expect(decodeResult(bytes)).toEqual({ lo: 1n, hi: 0n });
  });
});

describe("__udivti3 slow path (high half non-zero)", () => {
  test("a_hi != 0 takes slow path", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(7n, 1n, 3n, 0n));
    expect(decodeResult(bytes)).toEqual({
      lo: SLOW_PATH_QLO,
      hi: SLOW_PATH_QHI,
    });
  });

  test("b_hi != 0 takes slow path", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(7n, 0n, 3n, 1n));
    expect(decodeResult(bytes)).toEqual({
      lo: SLOW_PATH_QLO,
      hi: SLOW_PATH_QHI,
    });
  });

  test("both high halves set take slow path", () => {
    const bytes = runJamBytes(JAM_FILE, encodeArgs(7n, 1n, 3n, 1n));
    expect(decodeResult(bytes)).toEqual({
      lo: SLOW_PATH_QLO,
      hi: SLOW_PATH_QHI,
    });
  });
});
