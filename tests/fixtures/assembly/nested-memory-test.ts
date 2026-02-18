// Nested memory test: Simulate what happens in PVM-in-PVM
// This tests reading from computed addresses and writing to buffers

const BUFFER: u32 = 0x100;  // Same as RESULT_BUFFER in index-compiler.ts

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  // This simulates what index-compiler.ts does:
  // 1. Read header fields from args
  // 2. Copy data to a buffer
  // 3. Do some processing
  // 4. Write results
  
  let out_offset: u32 = 0;
  
  // === Phase 1: Read header (like index-compiler reads gas, pc, lens) ===
  // Store raw args_ptr and args_len for debugging
  store<u32>(RESULT_HEAP + out_offset, args_ptr as u32);
  out_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, args_len as u32);
  out_offset += 4;
  
  // Input format (simulating index-compiler):
  // [0-3]: u32 program_len
  // [4-7]: u32 data_len
  // [8..8+program_len]: program bytes
  // [8+program_len..]: data bytes
  
  let read_offset: u32 = 0;
  
  const program_len = load<u32>(args_ptr + read_offset);
  read_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, program_len);
  out_offset += 4;
  
  const data_len = load<u32>(args_ptr + read_offset);
  read_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, data_len);
  out_offset += 4;
  
  // === Phase 2: Copy program bytes to buffer (like prepareProgram) ===
  // This simulates the byte-by-byte copying that index-compiler does
  
  let bytes_copied: u32 = 0;
  for (let i: u32 = 0; i < program_len; i++) {
    const byte_val = load<u8>(args_ptr + read_offset + i);
    store<u8>(BUFFER + i, byte_val);
    bytes_copied++;
  }
  read_offset += program_len;
  
  store<u32>(RESULT_HEAP + out_offset, bytes_copied);
  out_offset += 4;
  
  // === Phase 3: Read back first 4 bytes from buffer ===
  const copied_first = load<u32>(BUFFER);
  store<u32>(RESULT_HEAP + out_offset, copied_first);
  out_offset += 4;
  
  // === Phase 4: Read data bytes ===
  let data_sum: u32 = 0;
  for (let i: u32 = 0; i < data_len && i < 8; i++) {
    data_sum += load<u8>(args_ptr + read_offset + i) as u32;
  }
  store<u32>(RESULT_HEAP + out_offset, data_sum);
  out_offset += 4;
  
  result_ptr = RESULT_HEAP as i32;
  result_len = out_offset as i32;
}
