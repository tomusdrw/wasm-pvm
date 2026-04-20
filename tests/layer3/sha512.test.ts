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
