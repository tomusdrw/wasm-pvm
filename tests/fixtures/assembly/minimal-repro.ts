/**
 * Minimal reproduction of the bug:
 * - Nested if-result blocks
 * - Ternary that takes THEN branch (does memory load)
 * - Drop the ternary result
 * - Two-argument function call
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function createArray(len: i32): u8[] {
  const r = new Array<u8>(len);
  for (let i: i32 = 0; i < len; i++) {
    r[i] = <u8>i;
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  // Load step FIRST (before any arrays) - mimics simple-call-test ordering
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr = createArray(10);
  const arr2 = createArray(10);  // Second array (dropped)

  let result: u32;

  // Outer if-result (mimic deep nesting like simple-call-test)
  if (step != 0) {
    if (step == 1) {
      // Shallow nesting: arr[1] after ternary
      const temp: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = arr[1];  // Should return 1
    } else if (step == 2) {
      const temp: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = arr[0];  // Should return 0
    } else if (step == 3) {
      const temp: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = arr[2];  // Should return 2
    } else if (step == 4) {
      result = arr[3];  // No ternary
    } else if (step == 5) {
      // Dummy branch for deeper nesting
      result = 55;
    } else if (step == 6) {
      // Deep nesting: create new array and access it
      // This mimics simple-call-test step==6
      const arr_only = createArray(10);
      const temp: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = arr_only[1];  // Should return 1
    } else if (step == 7) {
      result = arr[0];  // Even deeper
    } else {
      result = 99;
    }
  } else {
    result = arr[1];  // No ternary, should return 1
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
