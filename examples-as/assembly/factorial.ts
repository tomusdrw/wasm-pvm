// SPI Convention: args_ptr=0xFEFF0000, result heap=0x30100
// Globals at indices 0,1 are result_ptr, result_len

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const n = load<i32>(args_ptr);
  
  // Iterative factorial using loop+br (avoid if/else which needs more ops)
  let result: i32 = 1;
  let i: i32 = 1;
  
  // Use while loop pattern: block { loop { br_if exit; ...; br loop; } }
  while (i <= n) {
    result = result * i;
    i = i + 1;
  }
  
  store<i32>(RESULT_HEAP, result);
  
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
