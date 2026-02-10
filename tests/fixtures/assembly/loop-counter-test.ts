/**
 * Test to verify loop counter behavior between multiple loops.
 *
 * Hypothesis: The loop counter ($0/r9) may not be correctly reset
 * between loops, causing the second loop to not execute.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Creates an array of specified length with values 0..len-1
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
    // Test 0: Two simple loops, no function calls
    let sum: i32 = 0;
    for (let i: i32 = 0; i < 10; i++) {
      sum += 1;
    }
    for (let i: i32 = 0; i < 10; i++) {
      sum += 1;
    }
    result = <u32>sum;
    // Expected: 10 + 10 = 20
  } else if (step == 1) {
    // Test 1: Check loop counter value after first loop
    let i: i32 = 0;
    for (; i < 10; i++) {
      // Just count
    }
    // i should be 10 after the loop
    result = <u32>i;
    // Expected: 10
  } else if (step == 2) {
    // Test 2: Check if we can reset loop counter and run second loop
    let i: i32 = 0;
    for (; i < 10; i++) {
      // First loop
    }
    i = 0;  // Reset counter
    for (; i < 10; i++) {
      result += 1;
    }
    // Expected: 10
  } else if (step == 3) {
    // Test 3: Two loops with arrays but no function calls in loop body
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      // Just access, no call
      result += <u32>unchecked(arr1[i]);
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      // Just access, no call
      result += <u32>unchecked(arr2[i]);
    }
    // Expected: 45 + 45 = 90
  } else if (step == 4) {
    // Test 4: Two loops with manual length check in condition
    const arr1 = createArray(10);
    const arr2 = createArray(10);
    const len = 10;

    for (let i: i32 = 0; i < len; i++) {
      result += <u32>unchecked(arr1[i]);
    }
    for (let i: i32 = 0; i < len; i++) {
      result += <u32>unchecked(arr2[i]);
    }
    // Expected: 45 + 45 = 90
  } else if (step == 5) {
    // Test 5: Two loops with arrays AND function calls
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];  // Uses checked access (function call)
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];  // Uses checked access (function call)
    }
    // Expected: 45 + 45 = 90
  } else if (step == 6) {
    // Test 6: Return i value AFTER second loop starts but before body
    // This tests if the second loop condition check runs
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // After loop 1, result should be 45

    // Now return arr2.length to see if we can access it
    // (This should work based on test 4 in previous test file)
    result = <u32>(arr2.length * 1000 + (<i32>result));
    // Expected: 10 * 1000 + 45 = 10045
  } else if (step == 7) {
    // Test 7: Check what arr2.length comparison returns
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }

    // i is 0 here (new scope)
    // Check if 0 < arr2.length
    const i: i32 = 0;
    const cmp = i < arr2.length ? 1 : 0;
    result = <u32>cmp;
    // Expected: 1 (0 < 10)
  } else {
    // Test 8: Single loop, many iterations
    for (let i: i32 = 0; i < 100; i++) {
      result += 1;
    }
    // Expected: 100
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
