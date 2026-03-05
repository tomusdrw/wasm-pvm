// Memory addresses (hardcoded per PVM spec)
let RESULT_HEAP: usize = 0;

// Helper to write result and return packed i64
function writeResult(val: i32): i64 {
  store<i32>(RESULT_HEAP, val);
  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}

export function main(args_ptr: i32, args_len: i32): i64 {
  RESULT_HEAP = heap.alloc(256);
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);
  const c = load<i32>(args_ptr + 8);

  // Basic arithmetic
  let res = (a + b) * c; // (5 + 7) * 2 = 24

  // Bitwise
  res = res | 1; // 25
  res = res & ~2; // 25 (binary 11001 & 11101 = 11001)

  // Shifts
  res = res << 1; // 50
  res = res >> 1; // 25

  return writeResult(res);
}
