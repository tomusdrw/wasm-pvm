// Memory addresses
let RESULT_HEAP: usize = 0;

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}

type BinOp = (a: i32, b: i32) => i32;

function add(a: i32, b: i32): i32 { return a + b; }
function sub(a: i32, b: i32): i32 { return a - b; }
function mul(a: i32, b: i32): i32 { return a * b; }

export function main(args_ptr: i32, args_len: i32): void {
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

  writeResult(res);
}
