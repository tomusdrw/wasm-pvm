import { test, expect, describe } from "bun:test";
import path from "node:path";
import { JAM_DIR } from "../helpers/paths";
import { runJam, runJamBytes } from "../helpers/run";

const JAM_FILE = path.join(JAM_DIR, "bitreverse.jam");

// Reference bit-reverse implementations using BigInt to avoid JS sign issues.
function bitreverse32(x: number): number {
  let v = BigInt.asUintN(32, BigInt(x >>> 0));
  let r = 0n;
  for (let i = 0; i < 32; i++) {
    r = (r << 1n) | (v & 1n);
    v >>= 1n;
  }
  return Number(r);
}

function bitreverse64(x: bigint): bigint {
  let v = BigInt.asUintN(64, x);
  let r = 0n;
  for (let i = 0; i < 64; i++) {
    r = (r << 1n) | (v & 1n);
    v >>= 1n;
  }
  return r;
}

function u32ToHexLe(x: number): string {
  const v = x >>> 0;
  const bytes = [v & 0xff, (v >>> 8) & 0xff, (v >>> 16) & 0xff, (v >>> 24) & 0xff];
  return bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function u64ToHexLe(x: bigint): string {
  const v = BigInt.asUintN(64, x);
  let hex = "";
  for (let i = 0; i < 8; i++) {
    hex += Number((v >> BigInt(i * 8)) & 0xffn)
      .toString(16)
      .padStart(2, "0");
  }
  return hex;
}

function u64BytesLe(x: bigint): Uint8Array {
  const v = BigInt.asUintN(64, x);
  const out = new Uint8Array(8);
  for (let i = 0; i < 8; i++) {
    out[i] = Number((v >> BigInt(i * 8)) & 0xffn);
  }
  return out;
}

function bytesToHex(b: Uint8Array): string {
  return Array.from(b)
    .map((x) => x.toString(16).padStart(2, "0"))
    .join("");
}

// op=0 (i32 bitreverse): args = 4-byte op=0 + 4-byte little-endian value
function argsI32(value: number): string {
  return "00000000" + u32ToHexLe(value);
}

// op=1 (i64 bitreverse): args = 4-byte op=1 + 8-byte little-endian value
function argsI64(value: bigint): string {
  return "01000000" + u64ToHexLe(value);
}

// op=2 (i8 bitreverse): args = 4-byte op=2 + 1-byte value
function argsI8(value: number): string {
  return "02000000" + (value & 0xff).toString(16).padStart(2, "0");
}

// op=3 (i16 bitreverse): args = 4-byte op=3 + 2-byte little-endian value
function argsI16(value: number): string {
  const v = value & 0xffff;
  const lo = v & 0xff;
  const hi = (v >>> 8) & 0xff;
  return "03000000" + lo.toString(16).padStart(2, "0") + hi.toString(16).padStart(2, "0");
}

function bitreverse8(x: number): number {
  let v = x & 0xff;
  let r = 0;
  for (let i = 0; i < 8; i++) {
    r = ((r << 1) | (v & 1)) & 0xff;
    v >>>= 1;
  }
  return r;
}

function bitreverse16(x: number): number {
  let v = x & 0xffff;
  let r = 0;
  for (let i = 0; i < 16; i++) {
    r = ((r << 1) | (v & 1)) & 0xffff;
    v >>>= 1;
  }
  return r;
}

describe("bitreverse", () => {
  describe("i32", () => {
    const cases: Array<[number, number]> = [
      [0x00000000, 0x00000000],
      [0xffffffff | 0, 0xffffffff | 0],
      [0x12345678, 0x1e6a2c48],
      [0x00000001, 0x80000000 | 0],
      [0x80000000 | 0, 0x00000001],
      [0xaaaaaaaa | 0, 0x55555555],
    ];
    for (const [input, expected] of cases) {
      test(`bitreverse(0x${(input >>> 0).toString(16).padStart(8, "0")}) = 0x${(expected >>> 0).toString(16).padStart(8, "0")}`, () => {
        const actual = runJam(JAM_FILE, argsI32(input));
        // runJam returns the u32 result interpreted as a JS number.
        expect(actual >>> 0).toBe(expected >>> 0);
        expect(actual >>> 0).toBe(bitreverse32(input));
      });
    }
  });

  describe("i64", () => {
    const cases: Array<[bigint, bigint]> = [
      [0n, 0n],
      [0xffffffffffffffffn, 0xffffffffffffffffn],
      [0x0123456789abcdefn, 0xf7b3d591e6a2c480n],
      [1n, 0x8000000000000000n],
      [0x8000000000000000n, 1n],
      [0xaaaaaaaaaaaaaaaan, 0x5555555555555555n],
    ];
    for (const [input, expected] of cases) {
      test(`bitreverse(0x${input.toString(16).padStart(16, "0")}) = 0x${expected.toString(16).padStart(16, "0")}`, () => {
        const actual = runJamBytes(JAM_FILE, argsI64(input));
        const expectedBytes = u64BytesLe(expected);
        expect(bytesToHex(actual)).toBe(bytesToHex(expectedBytes));
        expect(bytesToHex(actual)).toBe(bytesToHex(u64BytesLe(bitreverse64(input))));
      });
    }
  });

  describe("i8", () => {
    const cases: Array<[number, number]> = [
      [0x00, 0x00],
      [0xff, 0xff],
      [0x12, 0x48],
      [0x01, 0x80],
      [0x80, 0x01],
      [0xaa, 0x55],
      [0x55, 0xaa],
      [0xf0, 0x0f],
      [0x0f, 0xf0],
    ];
    for (const [input, expected] of cases) {
      test(`bitreverse(0x${input.toString(16).padStart(2, "0")}) = 0x${expected.toString(16).padStart(2, "0")}`, () => {
        const actual = runJamBytes(JAM_FILE, argsI8(input));
        // Output is a single byte.
        expect(actual.length).toBe(1);
        expect(actual[0]).toBe(expected);
        expect(actual[0]).toBe(bitreverse8(input));
      });
    }
  });

  describe("i16", () => {
    const cases: Array<[number, number]> = [
      [0x0000, 0x0000],
      [0xffff, 0xffff],
      [0x1234, 0x2c48],
      [0x0001, 0x8000],
      [0x8000, 0x0001],
      [0xaaaa, 0x5555],
      [0x5555, 0xaaaa],
      [0xff00, 0x00ff],
    ];
    for (const [input, expected] of cases) {
      test(`bitreverse(0x${input.toString(16).padStart(4, "0")}) = 0x${expected.toString(16).padStart(4, "0")}`, () => {
        const actual = runJamBytes(JAM_FILE, argsI16(input));
        // Output is two little-endian bytes.
        expect(actual.length).toBe(2);
        const got = actual[0] | (actual[1] << 8);
        expect(got).toBe(expected);
        expect(got).toBe(bitreverse16(input));
      });
    }
  });
});
