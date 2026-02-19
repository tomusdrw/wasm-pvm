/**
 * Test ternary + drop + 1-arg call WITHOUT nested if-result blocks.
 * This isolates whether the bug is in nested if-result or just the
 * ternary+drop+call pattern.
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
  const arr = createArray(10);
  const arr2 = createArray(10);  // Second array (dropped)

  // The ternary - THEN branch does a memory load
  // Note: temp is never used, just like in nested-repro
  const temp: i32 = args_len > 0 ? load<u8>(args_ptr) : 5;

  // Access array - NO nested if-result blocks here
  let result: u32 = arr[1];  // Should always return 1

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
