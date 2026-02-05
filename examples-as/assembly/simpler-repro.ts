/**
 * Simpler reproduction - focus on the exact pattern that triggers the bug.
 * Bug: accessing arr[1] after a ternary THEN branch that does a memory load
 * returns 0 instead of 1.
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
  const arr = createArray(10);

  // This ternary determines if we take THEN (memory load) or ELSE (constant)
  // When THEN is taken, subsequent arr[1] access returns 0 instead of 1
  const temp: i32 = args_len > 0 ? load<u8>(args_ptr) : 5;

  // These three different index accesses show the pattern:
  // - index 0: always works
  // - index 1: fails when ternary takes THEN branch
  // - index 2: always works
  const index: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 1;

  let result: u32;
  if (index == 0) {
    result = arr[0];  // Should return 0
  } else if (index == 1) {
    result = arr[1];  // Should return 1, but returns 0 after THEN branch
  } else {
    result = arr[2];  // Should return 2
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
