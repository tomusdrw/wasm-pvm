export function main(args_ptr: i32, args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);
  const sum = a + b;

  store<i32>(RESULT_HEAP, sum);

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}
