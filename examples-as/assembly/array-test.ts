// Array test: Test creating arrays from args and reading from them
// This simulates what index-compiler.ts does

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  let out_offset: u32 = 0;
  
  // Store args info
  store<u32>(RESULT_HEAP + out_offset, args_ptr as u32);
  out_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, args_len as u32);
  out_offset += 4;
  
  // Input format:
  // [0-3]: count (u32) - number of bytes to copy
  // [4..4+count]: bytes to copy
  
  const count = load<u32>(args_ptr);
  store<u32>(RESULT_HEAP + out_offset, count);
  out_offset += 4;
  
  // Copy bytes to array (like index-compiler does)
  const arr: u8[] = [];
  for (let i: u32 = 0; i < count; i++) {
    const byte_val = load<u8>(args_ptr + 4 + i);
    arr.push(byte_val);
  }
  
  // Store array length
  store<u32>(RESULT_HEAP + out_offset, arr.length as u32);
  out_offset += 4;
  
  // Read bytes back from array
  for (let i = 0; i < arr.length && i < 8; i++) {
    store<u8>(RESULT_HEAP + out_offset, arr[i]);
    out_offset += 1;
  }
  
  // Pad to u32 boundary
  while (out_offset % 4 != 0) {
    store<u8>(RESULT_HEAP + out_offset, 0);
    out_offset += 1;
  }
  
  // Now test Uint8Array from array
  const uint8arr = new Uint8Array(arr.length);
  for (let i = 0; i < arr.length; i++) {
    uint8arr[i] = arr[i];
  }
  
  // Store uint8arr length
  store<u32>(RESULT_HEAP + out_offset, uint8arr.length as u32);
  out_offset += 4;
  
  // Read bytes back from uint8arr
  for (let i = 0; i < uint8arr.length && i < 8; i++) {
    store<u8>(RESULT_HEAP + out_offset, uint8arr[i]);
    out_offset += 1;
  }
  
  // Pad to u32 boundary
  while (out_offset % 4 != 0) {
    store<u8>(RESULT_HEAP + out_offset, 0);
    out_offset += 1;
  }
  
  // Test subarray
  if (uint8arr.length >= 2) {
    const sub = uint8arr.subarray(1);
    store<u32>(RESULT_HEAP + out_offset, sub.length as u32);
    out_offset += 4;
    
    for (let i = 0; i < sub.length && i < 4; i++) {
      store<u8>(RESULT_HEAP + out_offset, sub[i]);
      out_offset += 1;
    }
    
    while (out_offset % 4 != 0) {
      store<u8>(RESULT_HEAP + out_offset, 0);
      out_offset += 1;
    }
  }
  
  result_ptr = RESULT_HEAP;
  result_len = out_offset as i32;
}
