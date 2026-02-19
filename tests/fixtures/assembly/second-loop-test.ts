/**
 * Ultra-minimal test to check if the second loop ever executes
 * when arrays are involved.
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

  // Markers to track execution flow
  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Set marker bits to track which loops executed
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    // Marker bit 1: before loop 1
    result |= 1;

    for (let i: i32 = 0; i < arr1.length; i++) {
      // Marker bit 2: inside loop 1
      result |= 2;
      break; // Just one iteration
    }

    // Marker bit 4: after loop 1
    result |= 4;

    for (let i: i32 = 0; i < arr2.length; i++) {
      // Marker bit 8: inside loop 2
      result |= 8;
      break; // Just one iteration
    }

    // Marker bit 16: after loop 2
    result |= 16;
    // Expected: 1 | 2 | 4 | 8 | 16 = 31
  } else if (step == 1) {
    // Test 1: Same but with explicit length variables
    const arr1 = createArray(10);
    const arr2 = createArray(10);
    const len1 = arr1.length;
    const len2 = arr2.length;

    result |= 1;  // before loop 1

    for (let i: i32 = 0; i < len1; i++) {
      result |= 2;  // inside loop 1
      break;
    }

    result |= 4;  // after loop 1

    for (let i: i32 = 0; i < len2; i++) {
      result |= 8;  // inside loop 2
      break;
    }

    result |= 16;  // after loop 2
    // Expected: 31
  } else if (step == 2) {
    // Test 2: Check loop 2 condition evaluation
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }
    // result should be 45

    // Now check: is 0 < arr2.length?
    const loopCond = 0 < arr2.length;
    result = loopCond ? 1 : 0;
    // Expected: 1
  } else if (step == 3) {
    // Test 3: Manual check of what loop 2 would see
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      result += arr1[i];
    }

    // What value is at arr2 + 12 (the length field)?
    const arr2Ptr = changetype<usize>(arr2);
    result = <u32>load<i32>(arr2Ptr, 12);
    // Expected: 10
  } else if (step == 4) {
    // Test 4: Check loop counter state after loop 1
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    let i: i32 = 0;
    for (; i < arr1.length; i++) {
      // loop 1
    }
    // i should be 10 after loop 1

    // Return i to verify
    result = <u32>i;
    // Expected: 10
  } else if (step == 5) {
    // Test 5: Check if i=0 assignment works
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    let i: i32 = 0;
    for (; i < arr1.length; i++) {
      // loop 1
    }
    // i = 10

    i = 0;  // Reset
    result = <u32>i;
    // Expected: 0
  } else if (step == 6) {
    // Test 6: Very explicit second loop check
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    for (let i: i32 = 0; i < arr1.length; i++) {
      // loop 1 - just iterate
    }

    // Now explicit: set i=0, then check condition, then try to enter loop
    let i: i32 = 0;
    if (i < arr2.length) {
      result = 1;  // We can enter the condition
    } else {
      result = 2;  // Condition failed
    }
    // Expected: 1
  } else {
    // Test 7: Manual unrolled version
    const arr1 = createArray(10);
    const arr2 = createArray(10);

    // Loop 1 - unrolled first iteration only
    result += arr1[0];
    // result = 0

    // Loop 2 - check first iteration
    const val = arr2[0];
    result = <u32>val * 1000 + result;
    // Expected: 0 * 1000 + 0 = 0
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
