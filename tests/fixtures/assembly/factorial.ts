export function main(args_ptr: i32, args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
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

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}
