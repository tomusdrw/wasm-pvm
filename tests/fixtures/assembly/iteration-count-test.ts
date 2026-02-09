/**
 * Test to find at what iteration count the second loop stops working.
 */

const RESULT_HEAP: u32 = 0x30100;

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
  // Use args to control number of iterations in loop 1
  // args[0] = step (0-4 for different tests)
  // args[1] = iteration count for loop 1 (if applicable)
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;
  const iterCount: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 1;

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Variable iteration count, check if loop 2 runs
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    // Loop 1: run exactly iterCount iterations
    for (let i: i32 = 0; i < iterCount && i < arr1.length; i++) {
      result += arr1[i];
    }

    // Loop 2: just check if it runs at all
    let loop2Ran: bool = false;
    for (let i: i32 = 0; i < arr2.length; i++) {
      loop2Ran = true;
      break;
    }

    result = result * 100 + (loop2Ran ? 1 : 0);
    // Expected: sum(0..iterCount-1) * 100 + 1
  } else if (step == 1) {
    // Test 1: Exactly N iterations in loop 1, then full loop 2
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < iterCount && i < arr1.length; i++) {
      result += arr1[i];
    }

    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
    }
    // Expected: sum(0..iterCount-1) + sum(0..9)
  } else if (step == 2) {
    // Test 2: Simple counter in loop 1, check loop 2 entry
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    let count1: i32 = 0;
    for (let i: i32 = 0; i < iterCount && i < arr1.length; i++) {
      count1++;
    }

    let count2: i32 = 0;
    for (let i: i32 = 0; i < arr2.length; i++) {
      count2++;
      if (count2 == 1) break;  // Only count first entry
    }

    result = <u32>(count1 * 10 + count2);
    // Expected: iterCount * 10 + 1
  } else if (step == 3) {
    // Test 3: Without early termination condition
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    // Loop 1: always 10 iterations (no iterCount limit)
    for (let i: i32 = 0; i < 10; i++) {
      result += arr1[i];
    }

    // Loop 2: just once
    for (let i: i32 = 0; i < arr2.length; i++) {
      result += arr2[i];
      break;
    }
    // Expected: 45 + 0 = 45
  } else {
    // Test 4: Full 10 iterations in both loops
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < 10; i++) {
      result += arr1[i];
    }

    for (let i: i32 = 0; i < 10; i++) {
      result += arr2[i];
    }
    // Expected: 45 + 45 = 90
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
