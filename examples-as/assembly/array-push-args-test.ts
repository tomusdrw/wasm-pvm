/**
 * Test Array.push() with bytes loaded from args memory
 *
 * This mimics the PVM-in-PVM pattern where we load bytes from
 * the SPI args segment (around 0xFEFA0000) and push to an array.
 *
 * Input: 8 bytes (args)
 * Expected: Returns sum of all 8 input bytes
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Create an empty array
  const arr: u8[] = [];

  // Push bytes loaded from args memory (like in PVM-in-PVM)
  for (let i: i32 = 0; i < args_len && i < 8; i++) {
    const byte = load<u8>(args_ptr + i);
    arr.push(byte);
  }

  // Read them back and compute sum to verify correctness
  let sum: i32 = 0;
  for (let i = 0; i < arr.length; i++) {
    sum += arr[i];
  }

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
