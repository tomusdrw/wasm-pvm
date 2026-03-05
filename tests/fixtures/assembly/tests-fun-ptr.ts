// Memory addresses
let RESULT_HEAP: usize = 0;

function writeResult(val: i32): i64 {
  store<i32>(RESULT_HEAP, val);
  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}

type BinOp = (a: i32, b: i32) => i32;

function add(a: i32, b: i32): i32 { return a + b; }
function sub(a: i32, b: i32): i32 { return a - b; }
function mul(a: i32, b: i32): i32 { return a * b; }

export function main(args_ptr: i32, args_len: i32): i64 {
  RESULT_HEAP = heap.alloc(4);
  let op: BinOp;
  let res = 0;

  // 1. Add
  op = add;
  res += op(10, 2); // 12

  // 2. Sub
  op = sub;
  res += op(10, 2); // 12 + 8 = 20

  // 3. Mul
  op = mul;
  res += op(10, 2); // 20 + 20 = 40

  // 4. Conditional dispatch
  const condition = 1;
  if (condition > 0) {
    op = add;
  } else {
    op = sub;
  }
  res += op(5, 5); // 40 + 10 = 50

  return writeResult(res);
}
