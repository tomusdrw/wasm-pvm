// Test that console.log import can be mapped via --imports.
// The console.log call will be mapped to an ecalli with ptr_params
// so that the WASM pointer argument is converted to a PVM address.

@external("env", "console.log")
declare function consoleLog(msg: string): void;

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Store result first — the PVM runner may halt at the ecalli.
  store<i32>(RESULT_HEAP, 42);
  result_ptr = RESULT_HEAP;
  result_len = 4;

  consoleLog("Hello from PVM!");
}
