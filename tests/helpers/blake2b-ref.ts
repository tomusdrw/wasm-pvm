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
