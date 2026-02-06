/**
 * Test with non-inlinable function to force actual call.
 * Uses memory loads which prevent inlining.
 */

const RESULT_HEAP: u32 = 0x30100;
const DATA_HEAP: u32 = 0x30200;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Function that loads from memory - cannot be inlined to constant
function loadFromMemory(base: i32, index: i32): i32 {
  return load<u8>(base + index);
}

export function main(args_ptr: i32, args_len: i32): void {
  // Initialize some data in memory
  store<u8>(DATA_HEAP + 0, 10);
  store<u8>(DATA_HEAP + 1, 20);
  store<u8>(DATA_HEAP + 2, 30);

  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32;

  if (step != 0) {
    if (step == 1) {
      // Ternary then drop, then call with memory load
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = loadFromMemory(DATA_HEAP, 1);
      // Expected: 20 (DATA_HEAP + 1 = 20)
    } else if (step == 2) {
      // Same with index 0
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = loadFromMemory(DATA_HEAP, 0);
      // Expected: 10
    } else if (step == 3) {
      // Direct call without ternary
      result = loadFromMemory(DATA_HEAP, 1);
      // Expected: 20
    } else {
      result = 99;
    }
  } else {
    // Direct call without nesting
    result = loadFromMemory(DATA_HEAP, 1);
    // Expected: 20
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
