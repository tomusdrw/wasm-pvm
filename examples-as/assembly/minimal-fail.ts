/**
 * Minimal reproduction of the failing multi-slice test
 *
 * This is the exact pattern from largebuf-subarray-test step 5 that fails.
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
    // Pattern A: direct accumulation (failing pattern)
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);

    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
    }
    // Expected: 45 + 145 = 190
  } else if (step == 1) {
    // Pattern B: separate sums (working pattern)
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

    result = sum1 + sum2;
    // Expected: 45 + 145 = 190
  } else if (step == 2) {
    // Pattern C: Pattern A but without lowerBytes
    const slice1 = largeBuffer.subarray(0, 10);
    const slice2 = largeBuffer.subarray(10, 20);

    for (let i: i32 = 0; i < slice1.length; i++) {
      result += slice1[i];
    }
    for (let i: i32 = 0; i < slice2.length; i++) {
      result += slice2[i];
    }
    // Expected: 45 + 145 = 190
  } else {
    // Pattern D: Pattern A with explicit length tracking
    const slice1 = largeBuffer.subarray(0, 10);
    const arr1 = lowerBytes(slice1);
    const len1 = arr1.length;

    const slice2 = largeBuffer.subarray(10, 20);
    const arr2 = lowerBytes(slice2);
    const len2 = arr2.length;

    for (let i: i32 = 0; i < len1; i++) {
      result += arr1[i];
    }
    for (let i: i32 = 0; i < len2; i++) {
      result += arr2[i];
    }
    // Expected: 45 + 145 = 190
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
