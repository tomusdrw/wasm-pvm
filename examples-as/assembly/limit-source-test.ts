/**
 * Test to check if the source of the limit value affects behavior.
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
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  // Create two arrays BEFORE setting up the limit
  const arr1 = createArray(10);
  const arr2 = createArray(10);  // This becomes a spilled local!

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Hardcoded limit (5)
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: 0+1+2+3+4 = 10
  } else if (step == 1) {
    // Test 1: Limit from args
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: sum(0..limit-1) or sum(0..9) if limit > 10
  } else if (step == 2) {
    // Test 2: Limit from local variable set to constant
    let limit: i32 = 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: 10
  } else if (step == 3) {
    // Test 3: Without arr2 (fewer locals)
    const arr1_only = createArray(10);
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1_only.length; i++) {
      result += arr1_only[i];
    }
    // Expected: 10 (with limit=5)
  } else if (step == 4) {
    // Test 4: Explicit individual element check
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      const val = arr1[i];
      result += val;
    }
    // Expected: 10
  } else if (step == 5) {
    // Test 5: Check what arr1[1] returns
    const v1 = arr1[1];
    result = v1;
    // Expected: 1
  } else if (step == 6) {
    // Test 6: Sum just first two elements
    result = arr1[0] + arr1[1];
    // Expected: 0 + 1 = 1
  } else if (step == 7) {
    // Test 7: Two iterations only (hardcoded)
    for (let i: i32 = 0; i < 2 && i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: 0 + 1 = 1
  } else {
    // Test 8: Two iterations from args
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 2;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result += arr1[i];
    }
    // With limit=2: Expected: 0 + 1 = 1
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
