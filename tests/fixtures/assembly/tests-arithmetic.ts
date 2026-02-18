// Memory addresses (hardcoded per PVM spec)
let RESULT_HEAP: usize = 0;

// Globals required by SPI interface
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Helper to write result
function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}

export function main(args_ptr: i32, args_len: i32): void {
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

  writeResult(res);
}
