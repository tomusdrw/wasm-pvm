/**
 * Test to check what values are returned from array accesses.
 */


export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Allocate once at module scope to avoid leaking on repeated main() calls.
const RESULT_HEAP = heap.alloc(256);

function createArray(len: i32): u8[] {
  const r = new Array<u8>(len);
  for (let i: i32 = 0; i < len; i++) {
    r[i] = <u8>i;
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr1 = createArray(10);

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Return arr1[0]
    result = arr1[0];
    // Expected: 0
  } else if (step == 1) {
    // Test 1: Return arr1[1]
    result = arr1[1];
    // Expected: 1
  } else if (step == 2) {
    // Test 2: Return arr1[2]
    result = arr1[2];
    // Expected: 2
  } else if (step == 3) {
    // Test 3: Return arr1[0] + arr1[1] * 10
    result = <u32>arr1[0] + <u32>arr1[1] * 10;
    // Expected: 0 + 1*10 = 10
  } else if (step == 4) {
    // Test 4: Loop and check each value
    for (let i: i32 = 0; i < 5; i++) {
      result = result * 10 + arr1[i];
    }
    // Expected: 0*10+0 = 0, 0*10+1 = 1, 1*10+2 = 12, 12*10+3 = 123, 123*10+4 = 1234
  } else if (step == 5) {
    // Test 5: Check arr1[1] after accessing arr1[0]
    const v0 = arr1[0];
    const v1 = arr1[1];
    result = <u32>(v0 * 10 + v1);
    // Expected: 0*10 + 1 = 1
  } else if (step == 6) {
    // Test 6: Sum first 3 elements
    result = arr1[0] + arr1[1] + arr1[2];
    // Expected: 0 + 1 + 2 = 3
  } else if (step == 7) {
    // Test 7: Sum in a loop
    for (let i: i32 = 0; i < 3; i++) {
      result += arr1[i];
    }
    // Expected: 0 + 1 + 2 = 3
  } else {
    // Test 8: Direct memory read of arr1[1]
    const dataStart = load<i32>(changetype<usize>(arr1), 4);
    result = load<u8>(dataStart, 1);
    // Expected: 1
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
