
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  let a = load<i32>(args_ptr);
  let b = load<i32>(args_ptr + 4);
  
  while (b != 0) {
    const temp = b;
    b = a % b;
    a = temp;
  }
  
  store<i32>(RESULT_HEAP, a);
  
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
