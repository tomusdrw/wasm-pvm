import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";

// -----------------------------------------------------------------------------
// End-to-end test for compiler-builtins libcall recognition (__multi3).
//
// The fixture WAT declares a stub `__multi3` (writes zeros). With
// libcall_recognition enabled (default), the compiler replaces the body with
// a hand-crafted PVM implementation using `MulUpperUU`. If the result we
// observe matches the expected 128-bit product, recognition fired AND the
// synthesized body is correct.
// -----------------------------------------------------------------------------

const JAM_FILE = path.join(JAM_DIR, "u128-mul.jam");

const MASK_128 = (1n << 128n) - 1n;
const MASK_64 = (1n << 64n) - 1n;

function encodeArgs(a: bigint, b: bigint): string {
  const buf = new Uint8Array(32);
  const view = new DataView(buf.buffer);
  view.setBigUint64(0, a & MASK_64, true);
  view.setBigUint64(8, (a >> 64n) & MASK_64, true);
  view.setBigUint64(16, b & MASK_64, true);
  view.setBigUint64(24, (b >> 64n) & MASK_64, true);
  return Array.from(buf)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function decodeResult(bytes: Uint8Array): bigint {
  expect(bytes.length).toBe(16);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const lo = view.getBigUint64(0, true);
  const hi = view.getBigUint64(8, true);
  return (hi << 64n) | lo;
}

function expectProduct(a: bigint, b: bigint) {
  const expected = (a * b) & MASK_128;
  const bytes = runJamBytes(JAM_FILE, encodeArgs(a, b));
  const actual = decodeResult(bytes);
  expect(actual).toBe(expected);
}

describe("__multi3 libcall recognition", () => {
  test("0 × anything = 0", () => {
    expectProduct(0n, 0n);
    expectProduct(0n, 12345n);
    expectProduct(0n, MASK_128);
  });

  test("u64 × u64 (low product, both small)", () => {
    expectProduct(7n, 11n);
    expectProduct(1234567n, 7654321n);
    expectProduct(0xdeadbeefn, 0xcafef00dn);
  });

  test("u64 × u64 (low product, both near u64::MAX)", () => {
    // Both fit in u64. Product is up to 2^128-2^65+1, exercising the high half.
    expectProduct(MASK_64, MASK_64);
    expectProduct(MASK_64 - 1n, MASK_64 - 1n);
  });

  test("mixed widths: u64 × u128", () => {
    const a = MASK_64;
    const b = (1n << 100n) | 12345n; // sets high half of b
    expectProduct(a, b);
  });

  test("u128 × u128 — high halves contribute to result", () => {
    const a = (1n << 65n) | 7n;
    const b = (1n << 70n) | 13n;
    expectProduct(a, b);
  });

  test("modular wrap: full u128 × u128 truncates", () => {
    // a * b overflows; ensure we get the low 128 bits.
    expectProduct(MASK_128, MASK_128);
    expectProduct(MASK_128, 3n);
  });

  test("specific bit patterns", () => {
    // Stress carries: lo×lo high half + a_hi×b_lo low half must collide.
    expectProduct(0x8000000000000001n, 2n);
    expectProduct(0x8000000000000001n, 0x8000000000000001n);
  });
});
