/**
 * Test lowerBytes pattern - copy Uint8Array to Array<u8>
 *
 * This is the pattern used in anan-as program.ts
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

  // Create source array [0,1,2,3,4,5,6,7,8,9]
  const source = new Uint8Array(10);
  for (let i: i32 = 0; i < 10; i++) {
    source[i] = <u8>i;
  }

  let result: u32 = 0;

  if (step == 0) {
    // Step 0: Just check source array is correct
    // Expected: 0+1+2+...+9 = 45
    for (let i: i32 = 0; i < source.length; i++) {
      result += source[i];
    }
  } else if (step == 1) {
    // Step 1: Check subarray
    const slice = source.subarray(0, 10);
    // Expected: 0+1+2+...+9 = 45
    for (let i: i32 = 0; i < slice.length; i++) {
      result += slice[i];
    }
  } else if (step == 2) {
    // Step 2: Check lowerBytes result
    const arr = lowerBytes(source);
    // Check length: should be 10
    result = <u32>arr.length * 1000;  // Expected: 10000
    // Check sum: should be 45
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
    // Expected: 10045
  } else if (step == 3) {
    // Step 3: Check lowerBytes on subarray
    const slice = source.subarray(0, 10);
    const arr = lowerBytes(slice);
    // Check length
    result = <u32>arr.length * 1000;  // Expected: 10000
    // Check sum
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
    // Expected: 10045
  } else if (step == 4) {
    // Step 4: Check each element individually after lowerBytes
    const arr = lowerBytes(source);
    // Return as concatenated: arr[0]*1000000 + arr[1]*100000 + arr[2]*10000 + arr[3]*1000 + arr[4]*100 + sum(rest)
    // Expected: 0*1M + 1*100k + 2*10k + 3*1k + 4*100 + (5+6+7+8+9)
    // = 0 + 100000 + 20000 + 3000 + 400 + 35 = 123435
    result = <u32>arr[0] * 1000000;
    result += <u32>arr[1] * 100000;
    result += <u32>arr[2] * 10000;
    result += <u32>arr[3] * 1000;
    result += <u32>arr[4] * 100;
    for (let i: i32 = 5; i < arr.length; i++) {
      result += arr[i];
    }
  } else {
    // Step 5+: Direct comparison of indices
    const arr = lowerBytes(source);
    // Return index where arr[i] != i (or 999 if all match)
    for (let i: i32 = 0; i < arr.length; i++) {
      if (arr[i] != <u8>i) {
        result = <u32>i;
        // Also encode what value we got
        result += <u32>arr[i] * 1000;
        store<i32>(RESULT_HEAP, result);
        result_ptr = RESULT_HEAP;
        result_len = 4;
        return;
      }
    }
    result = 999;  // All match
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
