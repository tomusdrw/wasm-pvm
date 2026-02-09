/**
 * Flat test without nested if-else structures.
 * Uses global result instead of step-based branching.
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

  // Evaluate ternary (will be dropped)
  const limit: i32 = args_len > 0 ? load<u8>(args_ptr) : 5;

  // Access arr[1] after ternary
  const v1 = arr[1];

  // Return both: limit * 100 + v1
  const result = limit * 100 + v1;

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
