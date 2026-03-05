export function main(args_ptr: i32, args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
  let n = load<i32>(args_ptr);
  let a: i32 = 0;
  let b: i32 = 1;

  while (n > 0) {
    b = a + b;
    a = b - a;
    n = n - 1;
  }

  store<i32>(RESULT_HEAP, a);

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}
