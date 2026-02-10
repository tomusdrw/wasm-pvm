/**
 * Test Uint8Array with 2 elements - isolate the two-element issue
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Two elements, both < 128 - expected: 10+20=30
    const arr = new Uint8Array(2);
    arr[0] = 10;
    arr[1] = 20;
    result = <u32>arr[0] + <u32>arr[1];
  } else if (step == 1) {
    // Two elements: 127, 1 - expected: 128
    const arr = new Uint8Array(2);
    arr[0] = 127;
    arr[1] = 1;
    result = <u32>arr[0] + <u32>arr[1];
  } else if (step == 2) {
    // Two elements: 1, 128 - expected: 129
    const arr = new Uint8Array(2);
    arr[0] = 1;
    arr[1] = 128;
    result = <u32>arr[0] + <u32>arr[1];
  } else if (step == 3) {
    // Two elements: 128, 1 - expected: 129
    const arr = new Uint8Array(2);
    arr[0] = 128;
    arr[1] = 1;
    result = <u32>arr[0] + <u32>arr[1];
  } else if (step == 4) {
    // Two elements: 128, 128 - expected: 256
    const arr = new Uint8Array(2);
    arr[0] = 128;
    arr[1] = 128;
    result = <u32>arr[0] + <u32>arr[1];
  } else if (step == 5) {
    // Return arr[0] only - expected: 128
    const arr = new Uint8Array(2);
    arr[0] = 128;
    arr[1] = 159;
    result = arr[0];
  } else if (step == 6) {
    // Return arr[1] only - expected: 159
    const arr = new Uint8Array(2);
    arr[0] = 128;
    arr[1] = 159;
    result = arr[1];
  } else if (step == 7) {
    // Encode both values: arr[0]<<8 | arr[1] - expected: 128<<8|159 = 32927
    const arr = new Uint8Array(2);
    arr[0] = 128;
    arr[1] = 159;
    result = (<u32>arr[0] << 8) | <u32>arr[1];
  } else if (step == 8) {
    // Three elements: 128, 159, 200 - sum expected: 487
    const arr = new Uint8Array(3);
    arr[0] = 128;
    arr[1] = 159;
    arr[2] = 200;
    result = <u32>arr[0] + <u32>arr[1] + <u32>arr[2];
  } else if (step == 9) {
    // Store in reverse order: arr[1] first, then arr[0]
    const arr = new Uint8Array(2);
    arr[1] = 159;
    arr[0] = 128;
    result = <u32>arr[0] + <u32>arr[1];  // Expected: 287
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
