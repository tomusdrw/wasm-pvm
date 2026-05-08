import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR } from "../helpers/paths";
import { runJamBytes } from "../helpers/run";

const JAM_FILE = path.join(JAM_DIR, "saturating_arith.jam");

// =============================================================================
// Reference implementations (BigInt-based to avoid JS sign issues).
// =============================================================================

const U8_MAX = 0xffn;
const U16_MAX = 0xffffn;
const U32_MAX = 0xffffffffn;
const U64_MAX = 0xffffffffffffffffn;

const I8_MIN = -0x80n, I8_MAX = 0x7fn;
const I16_MIN = -0x8000n, I16_MAX = 0x7fffn;
const I32_MIN = -0x80000000n, I32_MAX = 0x7fffffffn;
const I64_MIN = -0x8000000000000000n, I64_MAX = 0x7fffffffffffffffn;

function clamp(x: bigint, lo: bigint, hi: bigint): bigint {
  if (x < lo) return lo;
  if (x > hi) return hi;
  return x;
}

const uaddSat = (a: bigint, b: bigint, max: bigint) => (a + b > max ? max : a + b);
const usubSat = (a: bigint, b: bigint) => (a < b ? 0n : a - b);
const saddSat = (a: bigint, b: bigint, lo: bigint, hi: bigint) => clamp(a + b, lo, hi);
const ssubSat = (a: bigint, b: bigint, lo: bigint, hi: bigint) => clamp(a - b, lo, hi);

// =============================================================================
// Argument encoding: 4-byte op + 8-byte a + 8-byte b (little-endian).
// =============================================================================

function u32LeHex(x: number | bigint): string {
  const v = BigInt.asUintN(32, BigInt(x));
  let h = "";
  for (let i = 0; i < 4; i++) h += Number((v >> BigInt(i * 8)) & 0xffn).toString(16).padStart(2, "0");
  return h;
}

function u64LeHex(x: bigint): string {
  const v = BigInt.asUintN(64, x);
  let h = "";
  for (let i = 0; i < 8; i++) h += Number((v >> BigInt(i * 8)) & 0xffn).toString(16).padStart(2, "0");
  return h;
}

function args(op: number, a: bigint, b: bigint): string {
  return u32LeHex(op) + u64LeHex(a) + u64LeHex(b);
}

function bytesToBigIntLe(bytes: Uint8Array): bigint {
  let v = 0n;
  for (let i = bytes.length - 1; i >= 0; i--) v = (v << 8n) | BigInt(bytes[i]);
  return v;
}

function bytesToSignedBigIntLe(bytes: Uint8Array, bits: number): bigint {
  const u = bytesToBigIntLe(bytes);
  const sign = 1n << BigInt(bits - 1);
  return u >= sign ? u - (1n << BigInt(bits)) : u;
}

// =============================================================================
// Test case generators.
// =============================================================================

interface UCase { a: bigint; b: bigint; }
interface SCase { a: bigint; b: bigint; }

const u8Cases: UCase[] = [
  { a: 0n, b: 0n },
  { a: 0xffn, b: 0n },
  { a: 0xffn, b: 1n },
  { a: 0xfen, b: 5n },
  { a: 5n, b: 10n },
  { a: 100n, b: 50n },
  { a: 0n, b: 0xffn },
  { a: 0x80n, b: 0x80n },
];

const u16Cases: UCase[] = [
  { a: 0n, b: 0n },
  { a: 0xffffn, b: 0n },
  { a: 0xffffn, b: 1n },
  { a: 0xfffen, b: 5n },
  { a: 5n, b: 10n },
  { a: 1000n, b: 500n },
  { a: 0n, b: 0xffffn },
  { a: 0x8000n, b: 0x8000n },
];

const u32Cases: UCase[] = [
  { a: 0n, b: 0n },
  { a: U32_MAX, b: 0n },
  { a: U32_MAX, b: 1n },
  { a: 0xfffffffen, b: 5n },
  { a: 5n, b: 10n },
  { a: 1_000_000n, b: 500_000n },
  { a: 0n, b: U32_MAX },
  { a: 0x80000000n, b: 0x80000000n },
];

const u64Cases: UCase[] = [
  { a: 0n, b: 0n },
  { a: U64_MAX, b: 0n },
  { a: U64_MAX, b: 1n },
  { a: U64_MAX - 4n, b: 5n },
  { a: 5n, b: 10n },
  { a: 1n << 40n, b: 1n << 40n },
  { a: 0n, b: U64_MAX },
  { a: 1n << 63n, b: 1n << 63n },
];

const s8Cases: SCase[] = [
  { a: 0n, b: 0n },
  { a: I8_MAX, b: 1n },
  { a: I8_MIN, b: 1n },
  { a: I8_MIN, b: -1n },
  { a: I8_MAX, b: -1n },
  { a: 50n, b: 50n },
  { a: -50n, b: -50n },
  { a: 100n, b: -100n },
];

const s16Cases: SCase[] = [
  { a: 0n, b: 0n },
  { a: I16_MAX, b: 1n },
  { a: I16_MIN, b: 1n },
  { a: I16_MIN, b: -1n },
  { a: I16_MAX, b: -1n },
  { a: 1000n, b: 1000n },
  { a: -1000n, b: -1000n },
  { a: 30000n, b: -30000n },
];

const s32Cases: SCase[] = [
  { a: 0n, b: 0n },
  { a: I32_MAX, b: 1n },
  { a: I32_MIN, b: 1n },
  { a: I32_MIN, b: -1n },
  { a: I32_MAX, b: -1n },
  { a: 1_000_000n, b: 1_000_000n },
  { a: -1_000_000n, b: -1_000_000n },
  { a: 0x7fffffffn, b: -0x7fffffffn },
];

const s64Cases: SCase[] = [
  { a: 0n, b: 0n },
  { a: I64_MAX, b: 1n },
  { a: I64_MIN, b: 1n },
  { a: I64_MIN, b: -1n },
  { a: I64_MAX, b: -1n },
  { a: 1n << 50n, b: 1n << 50n },
  { a: -(1n << 50n), b: -(1n << 50n) },
  { a: I64_MAX, b: -I64_MAX },
];

// =============================================================================
// Tests.
// =============================================================================

describe("uadd.sat", () => {
  describe("i8", () => {
    for (const { a, b } of u8Cases) {
      const expected = uaddSat(a, b, U8_MAX);
      test(`uadd_sat_u8(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(0, a, b));
        expect(out.length).toBe(1);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i16", () => {
    for (const { a, b } of u16Cases) {
      const expected = uaddSat(a, b, U16_MAX);
      test(`uadd_sat_u16(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(1, a, b));
        expect(out.length).toBe(2);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i32", () => {
    for (const { a, b } of u32Cases) {
      const expected = uaddSat(a, b, U32_MAX);
      test(`uadd_sat_u32(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(2, a, b));
        expect(out.length).toBe(4);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i64", () => {
    for (const { a, b } of u64Cases) {
      const expected = uaddSat(a, b, U64_MAX);
      test(`uadd_sat_u64(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(3, a, b));
        expect(out.length).toBe(8);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
});

describe("usub.sat", () => {
  describe("i8", () => {
    for (const { a, b } of u8Cases) {
      const expected = usubSat(a, b);
      test(`usub_sat_u8(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(4, a, b));
        expect(out.length).toBe(1);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i16", () => {
    for (const { a, b } of u16Cases) {
      const expected = usubSat(a, b);
      test(`usub_sat_u16(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(5, a, b));
        expect(out.length).toBe(2);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i32", () => {
    for (const { a, b } of u32Cases) {
      const expected = usubSat(a, b);
      test(`usub_sat_u32(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(6, a, b));
        expect(out.length).toBe(4);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
  describe("i64", () => {
    for (const { a, b } of u64Cases) {
      const expected = usubSat(a, b);
      test(`usub_sat_u64(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(7, a, b));
        expect(out.length).toBe(8);
        expect(bytesToBigIntLe(out)).toBe(expected);
      });
    }
  });
});

describe("sadd.sat", () => {
  describe("i8", () => {
    for (const { a, b } of s8Cases) {
      const expected = saddSat(a, b, I8_MIN, I8_MAX);
      test(`sadd_sat_i8(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(8, a, b));
        expect(out.length).toBe(1);
        expect(bytesToSignedBigIntLe(out, 8)).toBe(expected);
      });
    }
  });
  describe("i16", () => {
    for (const { a, b } of s16Cases) {
      const expected = saddSat(a, b, I16_MIN, I16_MAX);
      test(`sadd_sat_i16(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(9, a, b));
        expect(out.length).toBe(2);
        expect(bytesToSignedBigIntLe(out, 16)).toBe(expected);
      });
    }
  });
  describe("i32", () => {
    for (const { a, b } of s32Cases) {
      const expected = saddSat(a, b, I32_MIN, I32_MAX);
      test(`sadd_sat_i32(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(10, a, b));
        expect(out.length).toBe(4);
        expect(bytesToSignedBigIntLe(out, 32)).toBe(expected);
      });
    }
  });
  describe("i64", () => {
    for (const { a, b } of s64Cases) {
      const expected = saddSat(a, b, I64_MIN, I64_MAX);
      test(`sadd_sat_i64(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(11, a, b));
        expect(out.length).toBe(8);
        expect(bytesToSignedBigIntLe(out, 64)).toBe(expected);
      });
    }
  });
});

describe("ssub.sat", () => {
  describe("i8", () => {
    for (const { a, b } of s8Cases) {
      const expected = ssubSat(a, b, I8_MIN, I8_MAX);
      test(`ssub_sat_i8(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(12, a, b));
        expect(out.length).toBe(1);
        expect(bytesToSignedBigIntLe(out, 8)).toBe(expected);
      });
    }
  });
  describe("i16", () => {
    for (const { a, b } of s16Cases) {
      const expected = ssubSat(a, b, I16_MIN, I16_MAX);
      test(`ssub_sat_i16(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(13, a, b));
        expect(out.length).toBe(2);
        expect(bytesToSignedBigIntLe(out, 16)).toBe(expected);
      });
    }
  });
  describe("i32", () => {
    for (const { a, b } of s32Cases) {
      const expected = ssubSat(a, b, I32_MIN, I32_MAX);
      test(`ssub_sat_i32(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(14, a, b));
        expect(out.length).toBe(4);
        expect(bytesToSignedBigIntLe(out, 32)).toBe(expected);
      });
    }
  });
  describe("i64", () => {
    for (const { a, b } of s64Cases) {
      const expected = ssubSat(a, b, I64_MIN, I64_MAX);
      test(`ssub_sat_i64(${a}, ${b}) = ${expected}`, () => {
        const out = runJamBytes(JAM_FILE, args(15, a, b));
        expect(out.length).toBe(8);
        expect(bytesToSignedBigIntLe(out, 64)).toBe(expected);
      });
    }
  });
});
