// Globals at indices 0,1 are result_ptr, result_len


// Export mutable globals for result pointer and length
// These get stored at 0x20000 + idx*4 by wasm-pvm compiler
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);
  const sum = a + b;
  
  store<i32>(RESULT_HEAP, sum);
  
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
