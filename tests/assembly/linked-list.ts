// Memory addresses
const RESULT_HEAP: u32 = 0x30100;
const NODE_HEAP: u32 = 0x40000;

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}

// Node structure: [value: i32, next: i32] (8 bytes)

function createNode(ptr: i32, val: i32, next: i32): void {
  store<i32>(ptr, val);
  store<i32>(ptr + 4, next);
}

function sumList(head: i32): i32 {
  if (head == 0) return 0;
  
  const val = load<i32>(head);
  const next = load<i32>(head + 4);
  
  // Recursive sum
  return val + sumList(next);
}

export function main(args_ptr: i32, args_len: i32): void {
  // Create list: 10 -> 20 -> 30 -> null
  // Node 1 at 0x40000
  // Node 2 at 0x40008
  // Node 3 at 0x40010
  
  createNode(NODE_HEAP, 10, NODE_HEAP + 8);
  createNode(NODE_HEAP + 8, 20, NODE_HEAP + 16);
  createNode(NODE_HEAP + 16, 30, 0);
  
  const sum = sumList(NODE_HEAP); // 60
  
  writeResult(sum);
}
