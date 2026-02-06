/**
 * Test to isolate the issue with accessing array.length in loop conditions.
 *
 * The minimal-fail test shows Pattern A fails (using arr.length in loop condition)
 * but Pattern D works (storing length in a local first).
 *
 * This test focuses on that specific difference.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Simulates lowerBytes - creates an Array<u8> with values 0..len-1
function createArray(len: i32): u8[] {
  const r = new Array<u8>(len);
  for (let i: i32 = 0; i < len; i++) {
    r[i] = <u8>i;
  }
  return r;
}

// Simple function that returns its input (for testing call interference)
function getValue(arr: u8[], idx: i32): u8 {
  return arr[idx];
}

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Single array, access .length in loop condition
    // arr1 is in local $2 (r11)
    const arr1 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: 0+1+2+...+9 = 45
  } else if (step == 1) {
    // Test 1: Two arrays, access first .length in loop, sum first only
    const arr1 = createArray(10);  // $2 (r11)
    const arr2 = createArray(10);  // $3 (r12)

    // Only iterate over arr1
    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // Expected: 0+1+2+...+9 = 45
    // But also verify arr2 is intact by adding its first element
    result += arr2[0];
    // Expected: 45 + 0 = 45
  } else if (step == 2) {
    // Test 2: Two arrays, sum both using .length in loop conditions
    // This is the failing pattern!
    const arr1 = createArray(10);  // $2 (r11)
    const arr2 = createArray(10);  // $3 (r12), values are also 0-9

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
    }
    // Expected: 45 + 45 = 90
  } else if (step == 3) {
    // Test 3: Two arrays, sum both using local variables for length
    const arr1 = createArray(10);
    const arr2 = createArray(10);
    const len1 = arr1.length;
    const len2 = arr2.length;

    for (let i: i32 = 0; i < len1; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < len2; i++) {
      result += arr2[i];
    }
    // Expected: 45 + 45 = 90
  } else if (step == 4) {
    // Test 4: Check arr2 after first loop
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // First loop done, now check arr2's length
    result = <u32>arr2.length;
    // Expected: 10
  } else if (step == 5) {
    // Test 5: Check arr2[0] after first loop
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // First loop done, check arr2[0]
    result = arr2[0];
    // Expected: 0
  } else if (step == 6) {
    // Test 6: Return arr2 pointer as result (to see if it's valid)
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // Return arr2 as a number (it's a pointer)
    result = <u32>(changetype<usize>(arr2) & 0xFFFF);
    // Expected: some non-zero value
  } else if (step == 7) {
    // Test 7: Two loops but use getValue function instead of direct access
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += getValue(arr1, i);
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += getValue(arr2, i);
    }
    // Expected: 45 + 45 = 90
  } else {
    // Test 8: Check what i32.load offset=12 returns for arr2 after loop 1
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    // Run first loop
    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }

    // Now manually do what arr2.length does
    // Array layout: [mmInfo(4)][gcInfo(4)][length(4)][...data]
    // offset 12 is where length is stored
    const arr2Ptr = changetype<usize>(arr2);
    const loadedLen = load<i32>(arr2Ptr, 12);
    result = <u32>loadedLen;
    // Expected: 10
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
