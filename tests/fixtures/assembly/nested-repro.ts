/**
 * Nested if-result reproduction - matches minimal-repro nesting structure.
 *
 * The key pattern that triggers the bug:
 * 1. Outer if (result i32)
 * 2. Inner if (result i32) - else-if chain
 * 3. Ternary inside inner if
 * 4. Drop the ternary result
 * 5. Access array with index
 */


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
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr = createArray(10);
  const arr2 = createArray(10);  // Second array (dropped)

  let result: u32;

  // Outer if-result block (mimic step != 0)
  if (step != 0) {
    // Inner if-result block (mimic step == 1)
    if (step == 1) {
      // The ternary - THEN branch does a memory load
      const temp: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      // Access array - should return 1 but returns 0 when THEN branch taken
      result = arr[1];
    } else {
      // This branch should not be taken for step=1
      result = 99;
    }
  } else {
    // step == 0: no ternary, just array access
    result = arr[1];
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
