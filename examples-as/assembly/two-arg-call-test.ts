/**
 * Minimal test for two-argument function calls after ternary+drop in nested if-result.
 * The function uses memory access to prevent inlining/constant-folding.
 */

const RESULT_HEAP: u32 = 0x30100;
const DATA_HEAP: u32 = 0x30200;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// This function MUST have two runtime arguments that can't be constant-folded.
// It loads from base+index, which prevents optimization.
function loadAt(base: i32, index: i32): i32 {
  return load<u8>(base + index);
}

// Simple adder that can't be inlined (uses memory to break optimization)
function addWithMem(a: i32, b: i32): i32 {
  // Store and reload to break constant folding
  store<i32>(DATA_HEAP + 100, a);
  store<i32>(DATA_HEAP + 104, b);
  return load<i32>(DATA_HEAP + 100) + load<i32>(DATA_HEAP + 104);
}

export function main(args_ptr: i32, args_len: i32): void {
  // Initialize data: DATA_HEAP[0]=10, DATA_HEAP[1]=20, DATA_HEAP[2]=30
  store<u8>(DATA_HEAP, 10);
  store<u8>(DATA_HEAP + 1, 20);
  store<u8>(DATA_HEAP + 2, 30);

  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: i32;

  // Outer if-result
  if (step != 0) {
    // Nested if-result
    if (step == 1) {
      // Ternary that produces a value, then dropped
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      // 100 + 42 = 142
      result = addWithMem(100, 42);
    } else if (step == 2) {
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      // 200 + 42 = 242
      result = addWithMem(200, 42);
    } else if (step == 3) {
      // No ternary, direct call
      result = addWithMem(100, 42);
      // Expected: 142
    } else if (step == 4) {
      // Use step in computation to ensure runtime values
      result = addWithMem(step * 25, 42); // 4*25 + 42 = 142
    } else if (step == 5) {
      // loadAt with runtime base (DATA_HEAP from local)
      const base: i32 = DATA_HEAP;
      const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = loadAt(base, 1);
      // Expected: 20
    } else {
      result = 99;
    }
  } else {
    // Outside any nesting
    result = addWithMem(100, 42);
    // Expected: 142
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
