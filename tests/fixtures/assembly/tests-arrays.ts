// Memory addresses
let RESULT_HEAP: usize = 0;
let ARRAY_HEAP: usize = 0;

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}

// Simple array implementation
// [length: i32, item0: i32, item1: i32, ...]

function arraySet(arrPtr: i32, index: i32, value: i32): void {
  store<i32>(arrPtr + 4 + (index * 4), value);
}

function arrayGet(arrPtr: i32, index: i32): i32 {
  return load<i32>(arrPtr + 4 + (index * 4));
}

function arraySum(arrPtr: i32): i32 {
  const len = load<i32>(arrPtr);
  let sum = 0;
  for (let i = 0; i < len; i++) {
    sum += arrayGet(arrPtr, i);
  }
  return sum;
}

export function main(args_ptr: i32, args_len: i32): void {
  RESULT_HEAP = heap.alloc(256);
  ARRAY_HEAP = heap.alloc(32); // length + 5 ints = 24 bytes
  const arr = ARRAY_HEAP;
  const len = 5;
  store<i32>(arr, len);

  for (let i = 0; i < len; i++) {
    arraySet(arr, i, i * 10); // 0, 10, 20, 30, 40
  }

  // Sum = 100
  const sum = arraySum(arr);

  writeResult(sum);
}
