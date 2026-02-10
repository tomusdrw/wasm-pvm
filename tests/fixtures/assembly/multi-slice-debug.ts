/**
 * Debug multiple slices issue
 *
 * Tests what happens when we create two subarrays and lowerBytes them.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function lowerBytes(data: Uint8Array): u8[] {
  const r = new Array<u8>(data.length);
  for (let i = 0; i < data.length; i++) {
    r[i] = data[i];
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  // Create large buffer [0,1,2,...,255]
  const largeBuffer = new Uint8Array(256);
  for (let i: i32 = 0; i < 256; i++) {
    largeBuffer[i] = <u8>(i & 0xFF);
  }

  let result: u32 = 0;

  if (step == 0) {
    // Step 0: First slice only
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);
    // Expected: 45
    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
  } else if (step == 1) {
    // Step 1: Second slice only
    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);
    // Expected: 10+11+...+19 = 145
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
    }
  } else if (step == 2) {
    // Step 2: Both slices, sum separately
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);

    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);

    let sum1: u32 = 0;
    for (let i: i32 = 0; i < arr1.length; i++) {
      sum1 += arr1[i];
    }

    let sum2: u32 = 0;
    for (let i: i32 = 0; i < arr2.length; i++) {
      sum2 += arr2[i];
    }

    // Return sum1 * 1000 + sum2
    // Expected: 45 * 1000 + 145 = 45145
    result = sum1 * 1000 + sum2;
  } else if (step == 3) {
    // Step 3: Check arr2 first element
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);

    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);

    // Return arr2[0] * 10000 + arr2[1] * 100 + arr2.length
    // Expected: 10 * 10000 + 11 * 100 + 10 = 101110
    result = <u32>arr2[0] * 10000 + <u32>arr2[1] * 100 + <u32>arr2.length;
  } else if (step == 4) {
    // Step 4: Check slice2 directly (before lowerBytes)
    const slice1 = largeBuffer.subarray(0, 10);
    const slice2 = largeBuffer.subarray(10, 20);

    // Return slice2[0] * 10000 + slice2[1] * 100 + slice2.length
    // Expected: 10 * 10000 + 11 * 100 + 10 = 101110
    result = <u32>slice2[0] * 10000 + <u32>slice2[1] * 100 + <u32>slice2.length;
  } else if (step == 5) {
    // Step 5: Check if slice1 still has correct data after slice2 created
    const slice1 = largeBuffer.subarray(0, 10);
    const slice2 = largeBuffer.subarray(10, 20);

    // Return slice1[0] * 10000 + slice1[9] * 100 + slice1.length
    // Expected: 0 * 10000 + 9 * 100 + 10 = 910
    result = <u32>slice1[0] * 10000 + <u32>slice1[9] * 100 + <u32>slice1.length;
  } else {
    // Step 6: Both slices created, then both lowerBytes, check arr1
    const slice1 = largeBuffer.subarray(0, 10);
    const slice2 = largeBuffer.subarray(10, 20);

    const arr1 = lowerBytes(slice1);
    const arr2 = lowerBytes(slice2);

    // Return arr1[0] * 10000 + arr1[9] * 100 + arr1.length
    // Expected: 0 * 10000 + 9 * 100 + 10 = 910
    result = <u32>arr1[0] * 10000 + <u32>arr1[9] * 100 + <u32>arr1.length;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
