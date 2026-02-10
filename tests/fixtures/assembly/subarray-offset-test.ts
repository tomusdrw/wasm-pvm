/**
 * Subarray offset test - test if subarray returns correct view
 *
 * Tests that subarray(start, end) returns the correct slice
 * by creating a buffer [0,1,2,...,9] and checking slice values.
 *
 * Input: ignored
 * Expected: Sum of specific slice values
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Create buffer [0,1,2,3,4,5,6,7,8,9]
  const buffer = new Uint8Array(10);
  for (let i: i32 = 0; i < 10; i++) {
    buffer[i] = <u8>i;
  }

  // Test 1: subarray(0, 5) should give [0,1,2,3,4]
  const slice1 = buffer.subarray(0, 5);
  let sum: u32 = 0;

  // Check slice1 length and values
  const len1 = slice1.length;  // Should be 5
  sum += len1 * 100;  // +500
  sum += slice1[0];   // +0
  sum += slice1[1];   // +1
  sum += slice1[2];   // +2
  sum += slice1[3];   // +3
  sum += slice1[4];   // +4
  // Total so far: 500 + 0+1+2+3+4 = 510

  // Test 2: subarray(5) should give [5,6,7,8,9]
  const slice2 = buffer.subarray(5);
  const len2 = slice2.length;  // Should be 5
  sum += len2 * 100;  // +500
  sum += slice2[0];   // +5
  sum += slice2[1];   // +6
  sum += slice2[2];   // +7
  sum += slice2[3];   // +8
  sum += slice2[4];   // +9
  // Total so far: 510 + 500 + 5+6+7+8+9 = 1045

  // Test 3: subarray(3, 7) should give [3,4,5,6]
  const slice3 = buffer.subarray(3, 7);
  const len3 = slice3.length;  // Should be 4
  sum += len3 * 100;  // +400
  sum += slice3[0];   // +3
  sum += slice3[1];   // +4
  sum += slice3[2];   // +5
  sum += slice3[3];   // +6
  // Total: 1045 + 400 + 3+4+5+6 = 1463

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
