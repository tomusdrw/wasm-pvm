/**
 * Test for the if (result i32) construct used in short-circuit evaluation.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;
  const a: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 0;
  const b: i32 = args_len > 2 ? load<u8>(args_ptr + 2) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Simple && with two comparisons
    // a && b where a and b are the input values
    result = (a != 0 && b != 0) ? 1 : 0;
    // Expected: 1 if both non-zero, 0 otherwise
  } else if (step == 1) {
    // Test 1: Check if && short-circuits correctly
    // a < 5 && b < 10
    result = (a < 5 && b < 10) ? 1 : 0;
    // Expected: 1 if a<5 and b<10
  } else if (step == 2) {
    // Test 2: Nested conditions in loop-like pattern
    // Simulate: i < iterCount && i < arr.length where i=a, iterCount=b, arr.length=10
    const i: i32 = a;
    const iterCount: i32 = b;
    const arrLength: i32 = 10;
    result = (i < iterCount && i < arrLength) ? 1 : 0;
    // Expected: 1 if a < b and a < 10
  } else if (step == 3) {
    // Test 3: Loop with && condition - count iterations
    const limit: i32 = a;
    const maxIter: i32 = 10;
    let count: i32 = 0;
    for (let i: i32 = 0; i < limit && i < maxIter; i++) {
      count++;
    }
    result = <u32>count;
    // Expected: min(a, 10)
  } else if (step == 4) {
    // Test 4: Check specific iteration values
    const limit: i32 = 5;
    let iterations: i32 = 0;
    for (let i: i32 = 0; i < limit && i < 10; i++) {
      iterations = iterations * 10 + i;
    }
    result = <u32>iterations;
    // Expected: 01234 but encoded as 0*10+1=1, 1*10+2=12, 12*10+3=123, 123*10+4=1234
    // So: 0, 1, 12, 123, 1234 -> 1234
  } else if (step == 5) {
    // Test 5: Manual && simulation
    // Manually check both conditions without using &&
    const i: i32 = a;
    const iterCount: i32 = b;
    const arrLength: i32 = 10;
    let cond1: i32 = i < iterCount ? 1 : 0;
    let cond2: i32 = i < arrLength ? 1 : 0;
    result = (cond1 != 0 && cond2 != 0) ? 1 : 0;
    // Expected: 1 if a < b and a < 10
  } else if (step == 6) {
    // Test 6: Return value of first condition when it fails
    const i: i32 = a;
    const iterCount: i32 = b;
    // If i >= iterCount, short-circuit should return 0
    result = (i < iterCount) ? 1 : 0;
    // Expected: 1 if a < b
  } else if (step == 7) {
    // Test 7: Using || instead (to compare)
    result = (a != 0 || b != 0) ? 1 : 0;
    // Expected: 1 if either non-zero
  } else {
    // Test 8: Check if specific values work
    // With a=1, b=2: should be 1 < 2 && 1 < 10 = true
    const i: i32 = 1;
    const iterCount: i32 = 2;
    result = (i < iterCount && i < 10) ? 1 : 0;
    // Expected: 1
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
