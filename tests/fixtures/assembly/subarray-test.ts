/**
 * Minimal test for Uint8Array.subarray() bug in PVM
 *
 * Test case: Create Uint8Array [10,20,30,40,50], subarray from offset 2, read element 0
 * Expected: Returns 30 (element at original index 2)
 * Bug: subarray() doesn't work correctly, causes panic or wrong values
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Create a Uint8Array with known values
  const arr = new Uint8Array(5);
  arr[0] = 10;
  arr[1] = 20;
  arr[2] = 30;
  arr[3] = 40;
  arr[4] = 50;

  // Create a subarray starting at index 2
  const sub = arr.subarray(2);

  // Read element 0 of the subarray (should be 30)
  const value: i32 = sub[0];

  store<i32>(RESULT_HEAP, value);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
