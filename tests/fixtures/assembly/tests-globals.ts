// Memory addresses
let RESULT_HEAP: usize = 0;

// AS globals map to WASM globals
let globalVar: i32 = 10;
let globalCounter: i32 = 0;

function writeResult(val: i32): i64 {
  store<i32>(RESULT_HEAP, val);
  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}

function incrementGlobal(): void {
  globalCounter++;
}

function modifyGlobal(val: i32): void {
  globalVar = val;
}

export function main(args_ptr: i32, args_len: i32): i64 {
  RESULT_HEAP = heap.alloc(256);
  incrementGlobal();
  incrementGlobal();

  // globalCounter should be 2

  modifyGlobal(globalVar + 5); // 10 + 5 = 15

  const res = globalVar + globalCounter; // 15 + 2 = 17

  return writeResult(res);
}
