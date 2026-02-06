/**
 * Test storing values >= 128 to Uint8Array.
 * The complex-alloc bug shows values >= 128 might be problematic.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Store 127 (max positive i8) - expected: 127
    const arr = new Uint8Array(1);
    arr[0] = 127;
    result = arr[0];
  } else if (step == 1) {
    // Store 128 (first "negative" i8 if signed) - expected: 128
    const arr = new Uint8Array(1);
    arr[0] = 128;
    result = arr[0];
  } else if (step == 2) {
    // Store 255 (max u8) - expected: 255
    const arr = new Uint8Array(1);
    arr[0] = 255;
    result = arr[0];
  } else if (step == 3) {
    // Store via direct memory (bypass array) - expected: 128
    const ptr: u32 = 0x30200;
    store<u8>(ptr, 128);
    result = load<u8>(ptr);
  } else if (step == 4) {
    // Uint8Array with literal 128 via <u8> cast - expected: 128
    const arr = new Uint8Array(2);
    arr[0] = <u8>128;
    arr[1] = <u8>159;
    result = <u32>arr[0] + <u32>arr[1];  // Expected: 287
  } else if (step == 5) {
    // Store computed value (4*32 = 128) - expected: 128
    const arr = new Uint8Array(1);
    const val: i32 = 4 * 32;
    arr[0] = <u8>val;
    result = arr[0];
  } else if (step == 6) {
    // Store computed (4*32+31 = 159) - expected: 159
    const arr = new Uint8Array(1);
    const val: i32 = 4 * 32 + 31;
    arr[0] = <u8>val;
    result = arr[0];
  } else if (step == 7) {
    // Return the computed value itself (no array) - expected: 159
    result = <u32>((4 * 32 + 31) & 0xFF);
  } else if (step == 8) {
    // Sum 127 + 128 from array - expected: 255
    const arr = new Uint8Array(2);
    arr[0] = 127;
    arr[1] = 128;
    result = arr[0] + arr[1];
  } else if (step == 9) {
    // Direct memory store then array overlay read - expected: 128
    const ptr: u32 = 0x30200;
    store<u8>(ptr, 128);
    // Create Uint8Array pointing to this memory... not directly possible
    // Instead, just test direct memory
    store<u8>(ptr + 1, 159);
    result = <u32>load<u8>(ptr) + <u32>load<u8>(ptr + 1);  // Expected: 287
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
