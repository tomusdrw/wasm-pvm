export function main(args_ptr: i32, args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
  let a = load<i32>(args_ptr);
  let b = load<i32>(args_ptr + 4);

  while (b != 0) {
    const temp = b;
    b = a % b;
    a = temp;
  }

  store<i32>(RESULT_HEAP, a);

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}
