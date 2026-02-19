/**
 * Test to isolate whether locals are clobbered after function calls
 * and ternary expressions.
 */

const DATA_HEAP: u32 = 0x30200;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Force two-argument function with memory access
function loadAt(base: i32, index: i32): i32 {
  return load<u8>(base + index);
}

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  // Set up data: [10, 20, 30]
  store<u8>(DATA_HEAP, 10);
  store<u8>(DATA_HEAP + 1, 20);
  store<u8>(DATA_HEAP + 2, 30);

  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: i32 = 0;
  let saved: i32 = 0;  // This will use another local register

  if (step == 0) {
    // Test 0: Call, save result to local, return local
    const val = loadAt(DATA_HEAP, 1);  // Should be 20
    saved = val;
    result = saved;
    // Expected: 20
  } else if (step == 1) {
    // Test 1: Call, save result*10, then simple if-else, return local
    const val = loadAt(DATA_HEAP, 1);  // Should be 20
    saved = val * 10;  // saved = 200
    if (args_len > 1) {
      result = saved + 1;  // 201
    } else {
      result = saved + 2;  // 202
    }
    // With args_len=1: Expected: 202
  } else if (step == 2) {
    // Test 2: Call, save result*10, then TERNARY, return local + ternary
    const val = loadAt(DATA_HEAP, 1);  // Should be 20
    saved = val * 10;  // saved = 200
    const extra: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = saved + extra;
    // With args_len=1: Expected: 200 + 5 = 205
  } else if (step == 3) {
    // Test 3: Nested if, call, save, ternary, return
    if (step == 3) {  // Extra nesting
      const val = loadAt(DATA_HEAP, 1);  // Should be 20
      saved = val * 10;  // saved = 200
      const extra: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
      result = saved + extra;
    } else {
      result = 999;
    }
    // With args_len=1: Expected: 205
  } else if (step == 4) {
    // Test 4: Just return a constant to verify basic operation
    result = 4444;
    // Expected: 4444
  } else if (step == 5) {
    // Test 5: Save step*100, ternary, return saved
    saved = step * 100;  // 500
    const extra: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = saved + extra;
    // Expected: 500 + 5 = 505
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
