// Memory addresses
const RESULT_HEAP: u32 = 0x30100;
const STRUCT_HEAP: u32 = 0x40000;

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}

// Emulate a struct:
// struct Point {
//   x: i32; (offset 0)
//   y: i32; (offset 4)
//   z: i32; (offset 8)
// }

function setPoint(ptr: i32, x: i32, y: i32, z: i32): void {
  store<i32>(ptr, x);
  store<i32>(ptr + 4, y);
  store<i32>(ptr + 8, z);
}

function dotProduct(ptr1: i32, ptr2: i32): i32 {
  const x1 = load<i32>(ptr1);
  const y1 = load<i32>(ptr1 + 4);
  const z1 = load<i32>(ptr1 + 8);

  const x2 = load<i32>(ptr2);
  const y2 = load<i32>(ptr2 + 4);
  const z2 = load<i32>(ptr2 + 8);

  return x1 * x2 + y1 * y2 + z1 * z2;
}

export function main(args_ptr: i32, args_len: i32): void {
  const p1 = STRUCT_HEAP;
  const p2 = STRUCT_HEAP + 16;

  setPoint(p1, 1, 2, 3);
  setPoint(p2, 4, 5, 6);

  // Dot product: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
  const dp = dotProduct(p1, p2);

  writeResult(dp);
}
