/**
 * Test to verify that WASM locals are preserved across function calls.
 *
 * This tests the critical issue where local $3 (r12) appears to get
 * corrupted after function calls in certain patterns.
 */


export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// A simple function that just returns its input
// This will use r9-r10 for its locals, potentially clobbering caller's r9-r10
function identity(x: i32): i32 {
  return x;
}

// A function that uses more registers
// This will use r9-r12, potentially clobbering all caller locals
function useFourLocals(a: i32, b: i32, c: i32, d: i32): i32 {
  return a + b + c + d;
}

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Test 1: Simple - store two values in locals, call function, check both
    const val1: i32 = 42;
    const val2: i32 = 99;

    // val1 is in $2 (r11), val2 is in $3 (r12)
    // Call identity which may clobber caller's r9-r10
    const returned = identity(7);

    // After the call, val1 and val2 should still be correct
    result = <u32>(val1 + val2 + returned);
    // Expected: 42 + 99 + 7 = 148
  } else if (step == 1) {
    // Test 2: Multiple calls - call identity multiple times, check locals
    const val1: i32 = 10;
    const val2: i32 = 20;
    const val3: i32 = 30;

    let sum: i32 = 0;
    sum += identity(1);  // Call 1
    sum += identity(2);  // Call 2
    sum += identity(3);  // Call 3

    // All locals should still be correct
    result = <u32>(val1 + val2 + val3 + sum);
    // Expected: 10 + 20 + 30 + 1 + 2 + 3 = 66
  } else if (step == 2) {
    // Test 3: Call function that uses 4 locals
    const val1: i32 = 100;
    const val2: i32 = 200;

    // This call will use r9-r12, potentially clobbering val1 and val2
    const returned = useFourLocals(1, 2, 3, 4);

    // val1 and val2 should still be correct
    result = <u32>(val1 + val2 + returned);
    // Expected: 100 + 200 + 10 = 310
  } else if (step == 3) {
    // Test 4: Loop with call - similar to the failing pattern
    const val1: i32 = 5;
    const val2: i32 = 15;

    let sum: i32 = 0;
    for (let i: i32 = 0; i < 3; i++) {
      sum += identity(i);
    }

    // After the loop, val1 and val2 should still be correct
    result = <u32>(val1 + val2 + sum);
    // Expected: 5 + 15 + 0 + 1 + 2 = 23
  } else if (step == 4) {
    // Test 5: Two loops with calls
    const val1: i32 = 100;
    const val2: i32 = 200;

    let sum1: i32 = 0;
    for (let i: i32 = 0; i < 3; i++) {
      sum1 += identity(i);
    }

    let sum2: i32 = 0;
    for (let i: i32 = 0; i < 3; i++) {
      sum2 += identity(i + 10);
    }

    // val1 and val2 should still be correct
    result = <u32>(val1 + val2 + sum1 + sum2);
    // Expected: 100 + 200 + (0+1+2) + (10+11+12) = 100 + 200 + 3 + 33 = 336
  } else {
    // Test 6: Direct local check after call
    const a: i32 = 11;
    const b: i32 = 22;
    const c: i32 = 33;
    const d: i32 = 44;

    // a=$2(r11), b=$3(r12), c=$4(spilled), d=$5(spilled)
    identity(0);  // Call that may clobber registers

    // Return just b to see if $3 (r12) is preserved
    result = <u32>b;
    // Expected: 22
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
