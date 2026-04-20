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
