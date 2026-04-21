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

/**
 * Encode `(outLen, input)` into the WAT entry-point's args bytes:
 * `[outLen:u8][7 bytes zero pad][input]`.
 *
 * The 7-byte pad keeps the input portion 8-byte-aligned from `args_ptr`, which
 * lets the WAT's bulk memory.copy and per-block stream reads stay word-aligned
 * (avoiding cross-PVM-page u64 loads that made the unpadded format blow up on
 * inputs above ~4 KB). Matches the alignment pattern used by the SHA-512
 * fixture.
 */
export function encodeBlake2bArgs(args: Blake2bArgs): Uint8Array {
  const out = new Uint8Array(8 + args.input.length);
  out[0] = args.outLen;
  // bytes 1..7 stay zero (implicit with Uint8Array default).
  out.set(args.input, 8);
  return out;
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}
