/**
 * Minimal test for Array.push() bug in PVM
 *
 * Test case: Push bytes 0,1,2,3,4,5,6,7 to an array, then verify they're correct
 * Expected: Returns the sum (0+1+2+3+4+5+6+7 = 28)
 * Bug: Values appear at wrong indices after push()
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Create an empty array
  const arr: u8[] = [];

  // Push 8 bytes
  arr.push(0);
  arr.push(1);
  arr.push(2);
  arr.push(3);
  arr.push(4);
  arr.push(5);
  arr.push(6);
  arr.push(7);

  // Read them back and compute sum to verify correctness
  // Expected sum: 0+1+2+3+4+5+6+7 = 28
  let sum: i32 = 0;
  for (let i = 0; i < arr.length; i++) {
    // Each arr[i] should equal i
    // If it doesn't, the sum will be wrong
    sum += arr[i];
  }

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
