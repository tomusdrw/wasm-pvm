// Memory addresses
let RESULT_HEAP: usize = 0;

function writeResult(val: i32): i64 {
  store<i32>(RESULT_HEAP, val);
  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}

export function main(args_ptr: i32, args_len: i32): i64 {
  RESULT_HEAP = heap.alloc(256);
  const input = load<i32>(args_ptr);
  let result = 0;

  // If/Else
  if (input > 10) {
    result = 1;
  } else {
    result = 2;
  }

  // Loop
  let i = 0;
  while (i < input) {
    result += 1;
    i++;
  }

  // Nested Loop with break
  for (let j = 0; j < 5; j++) {
    for (let k = 0; k < 5; k++) {
      if (k > 2) break;
      result++;
    }
  }

  return writeResult(result);
}
