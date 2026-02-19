/**
 * Simplest possible test to check if arr[1] returns correct value after ternary.
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

  const arr1 = createArray(10);
  const arr2 = createArray(10);  // Spilled local to trigger the bug

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: arr[1] without ternary before it
    result = arr1[1];
    // Expected: 1
  } else if (step == 1) {
    // Test 1: arr[1] after ternary that uses args
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    // Now access arr[1] - this is where it might fail
    result = arr1[1];
    // Expected: 1
  } else if (step == 2) {
    // Test 2: Just the ternary value (to confirm ternary works)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = <u32>limit;
    // With args "0205": Expected: 5
  } else if (step == 3) {
    // Test 3: Check if arr[0] works after ternary
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr1[0];
    // Expected: 0
  } else if (step == 4) {
    // Test 4: Check if arr[2] works after ternary
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr1[2];
    // Expected: 2
  } else if (step == 5) {
    // Test 5: Ternary result in variable, then use it + arr[1]
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = <u32>(limit * 10 + arr1[1]);
    // With limit=5: Expected: 5*10 + 1 = 51
  } else if (step == 6) {
    // Test 6: Without arr2 (fewer locals)
    const arr_only = createArray(10);
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr_only[1];
    // Expected: 1
  } else if (step == 7) {
    // Test 7: arr[1] first, then ternary
    const v1 = arr1[1];
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = <u32>(v1 * 10 + limit);
    // With limit=5: Expected: 1*10 + 5 = 15
  } else {
    // Test 8: Direct memory access instead of arr[1]
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    // Get arr1's data pointer and read directly
    const dataPtr = load<u32>(changetype<usize>(arr1) + 4);
    result = load<u8>(dataPtr + 1);
    // Expected: 1
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
