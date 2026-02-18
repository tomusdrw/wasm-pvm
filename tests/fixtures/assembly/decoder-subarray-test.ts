/**
 * Test Uint8Array.subarray() in a Decoder-like pattern
 *
 * This mimics the varU32() pattern in the Decoder class that
 * calls subarray() to get a view starting at the current offset.
 *
 * Input: at least 5 bytes
 * Expected: Returns first byte (simulating varU32 for small values)
 */


export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  // Create a Uint8Array from args (like liftBytes does)
  const data = new Uint8Array(args_len);
  for (let i: i32 = 0; i < args_len; i++) {
    data[i] = load<u8>(args_ptr + i);
  }

  // Simulate Decoder pattern: subarray from offset
  // Start with offset 0, call subarray, read first byte
  let offset: i32 = 0;

  // First "varU32" read at offset 0
  const slice1 = data.subarray(offset);
  const val1: i32 = slice1[0];
  offset += 1;

  // Second "varU32" read at offset 1
  const slice2 = data.subarray(offset);
  const val2: i32 = slice2[0];
  offset += 1;

  // Third read at offset 2
  const slice3 = data.subarray(offset);
  const val3: i32 = slice3[0];

  // Return sum of first 3 values
  const sum = val1 + val2 + val3;

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
