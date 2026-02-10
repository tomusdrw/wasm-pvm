/**
 * Test for && conditions that include memory loads.
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

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Simple loop with arr.length in condition
    const arr = createArray(10);
    for (let i: i32 = 0; i < arr.length; i++) {
      result++;
    }
    // Expected: 10
  } else if (step == 1) {
    // Test 1: Loop with && and arr.length
    const arr = createArray(10);
    const limit: i32 = 5;
    for (let i: i32 = 0; i < limit && i < arr.length; i++) {
      result++;
    }
    // Expected: 5
  } else if (step == 2) {
    // Test 2: Loop with && where arr.length is the limiting factor
    const arr = createArray(5);
    const limit: i32 = 10;
    for (let i: i32 = 0; i < limit && i < arr.length; i++) {
      result++;
    }
    // Expected: 5
  } else if (step == 3) {
    // Test 3: Loop with && and array access in body
    const arr = createArray(10);
    const limit: i32 = 5;
    for (let i: i32 = 0; i < limit && i < arr.length; i++) {
      result += arr[i];
    }
    // Expected: 0 + 1 + 2 + 3 + 4 = 10
  } else if (step == 4) {
    // Test 4: Two arrays, loop with && on first, then access second
    const arr1 = createArray(10);
    const arr2 = createArray(10);
    const limit: i32 = 5;

    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result += arr1[i];
    }
    // After loop, check arr2[0]
    result = result * 10 + arr2[0];
    // Expected: 10 * 10 + 0 = 100
  } else if (step == 5) {
    // Test 5: Check specific iteration - what value does i have?
    const arr = createArray(10);
    const limit: i32 = 5;
    let lastI: i32 = -1;
    for (let i: i32 = 0; i < limit && i < arr.length; i++) {
      lastI = i;
    }
    result = <u32>lastI;
    // Expected: 4 (last value of i before exit)
  } else if (step == 6) {
    // Test 6: Manual version without &&
    const arr = createArray(10);
    const limit: i32 = 5;
    let i: i32 = 0;
    while (i < limit) {
      if (i >= arr.length) break;
      result++;
      i++;
    }
    // Expected: 5
  } else if (step == 7) {
    // Test 7: Loop with explicit length variable
    const arr = createArray(10);
    const limit: i32 = 5;
    const len: i32 = arr.length;  // Cache the length
    for (let i: i32 = 0; i < limit && i < len; i++) {
      result++;
    }
    // Expected: 5
  } else {
    // Test 8: Two arrays with two loops, each using arr.length
    const arr1 = createArray(10);
    const arr2 = createArray(10);
    const limit: i32 = 5;

    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < limit && i < arr2.length; i++) {
      result += arr2[i];
    }
    // Expected: 10 + 10 = 20
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
