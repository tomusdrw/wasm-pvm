const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  let a = load<i32>(args_ptr);
  let b = load<i32>(args_ptr + 4);
  
  while (b != 0) {
    const temp = b;
    b = a % b;
    a = temp;
  }
  
  store<i32>(RESULT_HEAP, a);
  
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
