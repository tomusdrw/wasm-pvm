// Test that console.log import can be mapped via --imports.
// The console.log call will be mapped to an ecalli with ptr_params
// so that the WASM pointer argument is converted to a PVM address.


export function main(_args_ptr: i32, _args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
  // Store result first — the PVM runner may halt at the ecalli.
  store<i32>(RESULT_HEAP, 42);

  console.log("Hello from PVM!");

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}
