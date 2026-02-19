/**
 * Test to trace values at each loop iteration.
 * Returns a number encoding the values seen at each iteration.
 */


export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function createArray(len: i32): u8[] {
  const r = new Array<u8>(len);
  for (let i: i32 = 0; i < len; i++) {
    r[i] = <u8>i;
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr1 = createArray(10);
  const arr2 = createArray(10);  // Spilled local

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Return the number of iterations (hardcoded limit)
    let count: i32 = 0;
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      count++;
    }
    result = <u32>count;
    // Expected: 5
  } else if (step == 1) {
    // Test 1: Return the number of iterations (limit from args)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let count: i32 = 0;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      count++;
    }
    result = <u32>count;
    // With limit=5: Expected: 5
  } else if (step == 2) {
    // Test 2: Encode which iterations ran (hardcoded)
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      result |= (1 << i);
    }
    // Expected: 11111 binary = 31
  } else if (step == 3) {
    // Test 3: Encode which iterations ran (limit from args)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      result |= (1 << i);
    }
    // With limit=5: Expected: 11111 binary = 31
  } else if (step == 4) {
    // Test 4: Check condition values manually
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    // Check i=0
    let i: i32 = 0;
    const cond0 = (i < limit && i < arr1.length) ? 1 : 0;
    // Check i=1
    i = 1;
    const cond1 = (i < limit && i < arr1.length) ? 1 : 0;
    // Check i=2
    i = 2;
    const cond2 = (i < limit && i < arr1.length) ? 1 : 0;
    result = <u32>(cond0 * 100 + cond1 * 10 + cond2);
    // With limit=5: Expected: 111
  } else if (step == 5) {
    // Test 5: Just check the first condition (i < limit)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let i: i32 = 1;
    result = (i < limit) ? 1 : 0;
    // With limit=5: Expected: 1 (1 < 5 is true)
  } else if (step == 6) {
    // Test 6: Check limit value
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = <u32>limit;
    // With arg 05: Expected: 5
  } else if (step == 7) {
    // Test 7: Simplified loop - just count with limit from args
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let count: i32 = 0;
    for (let i: i32 = 0; i < limit; i++) {
      count++;
    }
    result = <u32>count;
    // With limit=5: Expected: 5
  } else {
    // Test 8: Simplified loop with arr.length only
    let count: i32 = 0;
    for (let i: i32 = 0; i < arr1.length; i++) {
      count++;
    }
    result = <u32>count;
    // Expected: 10
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
