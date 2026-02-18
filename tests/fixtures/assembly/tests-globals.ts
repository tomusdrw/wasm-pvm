// Memory addresses
let RESULT_HEAP: usize = 0;

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// AS globals map to WASM globals
let globalVar: i32 = 10;
let globalCounter: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}

function incrementGlobal(): void {
  globalCounter++;
}

function modifyGlobal(val: i32): void {
  globalVar = val;
}

export function main(args_ptr: i32, args_len: i32): void {
  RESULT_HEAP = heap.alloc(256);
  incrementGlobal();
  incrementGlobal();

  // globalCounter should be 2

  modifyGlobal(globalVar + 5); // 10 + 5 = 15

  const res = globalVar + globalCounter; // 15 + 2 = 17

  writeResult(res);
}
