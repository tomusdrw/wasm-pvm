/**
 * Minimal test to reproduce the nested if + drop + call pattern.
 * This isolates the exact failing pattern.
 */


export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Two-arg function that returns second arg (to test arg passing)
function getSecondArg(a: i32, b: i32): i32 {
  return b;
}

// Two-arg function that returns first arg
function getFirstArg(a: i32, b: i32): i32 {
  return a;
}

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32;

  // Outer if to create nesting
  if (step != 0) {
    // Inner if
    if (step == 1) {
      // Ternary then drop, then 2-arg call
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      // Don't use limit - just drop it
      result = getSecondArg(100, 42);
      // Expected: 42
    } else if (step == 2) {
      // Same but check first arg
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = getFirstArg(100, 42);
      // Expected: 100
    } else if (step == 3) {
      // Direct call without ternary
      result = getSecondArg(100, 42);
      // Expected: 42
    } else {
      // Return args passed: first*1000 + second
      const a: i32 = 100;
      const b: i32 = 42;
      result = <u32>(getFirstArg(a, b) * 1000 + getSecondArg(a, b));
      // Expected: 100042
    }
  } else {
    // Direct call without any nesting
    result = getSecondArg(100, 42);
    // Expected: 42
  }

  store<u32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
