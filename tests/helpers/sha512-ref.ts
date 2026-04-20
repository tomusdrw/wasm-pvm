import { sha512 } from "@noble/hashes/sha2";

export interface Sha512Args {
  /** Input to hash. */
  input: Uint8Array;
}

/** Reference SHA-512 via `@noble/hashes`. Output is always 64 bytes. */
export function sha512Ref(args: Sha512Args): Uint8Array {
  return sha512(args.input);
}

/**
 * Encode args for the WAT entry-point: args = [input]. No prefix — SHA-512
 * has a fixed 64-byte output, so there is nothing to parameterize.
 */
export function encodeSha512Args(args: Sha512Args): Uint8Array {
  // Return a copy so the caller can mutate the encoded buffer without affecting args.input.
  return new Uint8Array(args.input);
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}
