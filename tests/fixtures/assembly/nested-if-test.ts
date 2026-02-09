/**
 * Test to isolate nested if (result i32) behavior.
 * This reproduces the pattern from step-based tests but minimal.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Create array
  const arr = new Array<u8>(10);
  for (let i: i32 = 0; i < 10; i++) {
    arr[i] = <u8>i;
  }

  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32;

  // This creates nested if (result i32) blocks
  if (step == 0) {
    // No ternary, just arr[1]
    result = arr[1];
    // Expected: 1
  } else if (step == 1) {
    // Ternary then arr[1]
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = <u32>(limit * 100 + arr[1]);
    // With limit=5: Expected: 501
  } else if (step == 2) {
    // Test with explicit nesting level check
    // Return which nesting level we're in
    result = 2;
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
