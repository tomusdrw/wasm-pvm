// Memory addresses
let RESULT_HEAP: usize = 0;
const DATA_HEAP: u32 = 0x40000; // Arbitrary safe location for data

// Globals
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}

export function main(args_ptr: i32, args_len: i32): void {
  RESULT_HEAP = heap.alloc(256);
  // Store 8-bit values
  store<u8>(DATA_HEAP, 0xAA);
  store<u8>(DATA_HEAP + 1, 0xBB);
  store<u8>(DATA_HEAP + 2, 0xCC);
  store<u8>(DATA_HEAP + 3, 0xDD);

  // Read back as 32-bit (little endian)
  // Expect: 0xDDCCBBAA
  let val32 = load<i32>(DATA_HEAP);

  // Modify one byte
  store<u8>(DATA_HEAP + 1, 0xFF);

  // Read back individual bytes
  let b0: i32 = load<u8>(DATA_HEAP);     // 0xAA
  let b1: i32 = load<u8>(DATA_HEAP + 1); // 0xFF
  let b2: i32 = load<u8>(DATA_HEAP + 2); // 0xCC
  let b3: i32 = load<u8>(DATA_HEAP + 3); // 0xDD

  // Verify (sum of bytes)
  let sum: i32 = b0 + b1 + b2 + b3; // 170 + 255 + 204 + 221 = 850

  writeResult(sum);
}
