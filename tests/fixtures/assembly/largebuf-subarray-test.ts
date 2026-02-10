/**
 * Test subarray and lowerBytes on a larger buffer
 *
 * This tests if slicing a 256-byte buffer works correctly.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function lowerBytes(data: Uint8Array): u8[] {
  const r = new Array<u8>(data.length);
  for (let i: i32 = 0; i < data.length; i++) {
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
    // Step 0: Direct sum of first 10 elements
    // Expected: 0+1+2+...+9 = 45
    for (let i: i32 = 0; i < 10; i++) {
      result += largeBuffer[i];
    }
  } else if (step == 1) {
    // Step 1: Subarray of first 10 elements
    const slice = largeBuffer.subarray(0, 10);
    // Expected: 0+1+2+...+9 = 45
    for (let i: i32 = 0; i < slice.length; i++) {
      result += slice[i];
    }
  } else if (step == 2) {
    // Step 2: lowerBytes on subarray
    const slice = largeBuffer.subarray(0, 10);
    const arr = lowerBytes(slice);
    // Expected: 0+1+2+...+9 = 45
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
  } else if (step == 3) {
    // Step 3: lowerBytes on subarray + check length
    const slice = largeBuffer.subarray(0, 10);
    const arr = lowerBytes(slice);
    // Return length * 1000 + sum
    // Expected: 10000 + 45 = 10045
    result = <u32>arr.length * 1000;
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
  } else if (step == 4) {
    // Step 4: Slice middle of buffer [100-109]
    const slice = largeBuffer.subarray(100, 110);
    const arr = lowerBytes(slice);
    // Expected: 100+101+...+109 = 1045
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
  } else {
    // Step 5: Multiple slices and lowerBytes
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);

    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);

    // Sum both: (0+...+9) + (10+...+19) = 45 + 145 = 190
    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
    }
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
