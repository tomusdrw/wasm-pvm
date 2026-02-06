/**
 * Test varU32 encoding/decoding pattern from codec.ts
 *
 * This tests the specific pattern that deblob() uses:
 * - Create Uint8Array
 * - Call subarray() to get a slice
 * - Read bytes from the slice
 *
 * Input: varU32-encoded values (e.g., simple bytes < 0x80 are 1-byte encoded)
 * Expected: Decoded value
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Masks for varU32 length detection (from codec.ts)
const MASKS: u8[] = [0xff, 0xfe, 0xfc, 0xf8, 0xf0, 0xe0, 0xc0, 0x80];

function variableLength(firstByte: u8): u8 {
  const len = <u8>MASKS.length;
  for (let i: u8 = 0; i < len; i++) {
    if (firstByte >= MASKS[i]) {
      return 8 - i;
    }
  }
  return 0;
}

// Simplified varU32 decoder (from codec.ts decodeVarU32)
function decodeVarU32(data: Uint8Array): u32 {
  const length = variableLength(data[0]);
  const first = u32(data[0]);

  if (length === 0) {
    // Single byte value
    return first;
  }

  // Multi-byte value
  const msb = (first + (1 << (8 - length)) - 256) << (length * 8);
  let number: u32 = 0;
  for (let i: i32 = length - 1; i >= 0; i--) {
    number = (number << 8) + data[1 + i];
  }
  number += msb;

  return number;
}

export function main(args_ptr: i32, args_len: i32): void {
  // Create a Uint8Array from args
  const data = new Uint8Array(args_len);
  for (let i: i32 = 0; i < args_len; i++) {
    data[i] = load<u8>(args_ptr + i);
  }

  // Test 1: Simple single-byte value (< 0x80)
  // The subarray call is what we're really testing
  const slice1 = data.subarray(0);
  const val1 = decodeVarU32(slice1);

  // Test 2: Read second varU32 at offset 1
  const slice2 = data.subarray(1);
  const val2 = decodeVarU32(slice2);

  // Test 3: Read third varU32 at offset 2
  const slice3 = data.subarray(2);
  const val3 = decodeVarU32(slice3);

  // Return sum of decoded values
  const sum = val1 + val2 + val3;

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
