// Memory test: Verify we can read arguments correctly and detect issues
// This test reads the first 8 bytes of args and echoes them back, plus some diagnostics


// Export mutable globals for result pointer and length
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  // Output format:
  // [0-3]: args_ptr value (what we received)
  // [4-7]: args_len value (what we received)
  // [8-11]: first u32 from args
  // [12-15]: second u32 from args
  // [16-19]: sum of first two u32s
  // [20-23]: args_ptr + 0 (should be args_ptr)
  // [24-27]: args_ptr + 4 (should be args_ptr + 4)
  
  let offset: u32 = 0;
  
  // Store args_ptr value
  store<u32>(RESULT_HEAP + offset, args_ptr as u32);
  offset += 4;
  
  // Store args_len value
  store<u32>(RESULT_HEAP + offset, args_len as u32);
  offset += 4;
  
  // Read and store first arg
  const a = load<u32>(args_ptr);
  store<u32>(RESULT_HEAP + offset, a);
  offset += 4;
  
  // Read and store second arg
  const b = load<u32>(args_ptr + 4);
  store<u32>(RESULT_HEAP + offset, b);
  offset += 4;
  
  // Store sum
  store<u32>(RESULT_HEAP + offset, a + b);
  offset += 4;
  
  // Store computed addresses (for debugging)
  store<u32>(RESULT_HEAP + offset, (args_ptr + 0) as u32);
  offset += 4;
  
  store<u32>(RESULT_HEAP + offset, (args_ptr + 4) as u32);
  offset += 4;
  
  result_ptr = RESULT_HEAP as i32;
  result_len = offset as i32;
}
