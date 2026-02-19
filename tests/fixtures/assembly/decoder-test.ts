// Decoder test: Simulate what index-compiler.ts and decodeSpi do
// This tests the exact pattern that fails in PVM-in-PVM

let RESULT_HEAP: usize = 0;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Simplified Decoder (same pattern as codec.ts)
class Decoder {
  private offset: i32 = 0;
  
  constructor(private readonly source: Uint8Array) {}
  
  private ensureBytes(need: u32): void {
    if (this.offset + need > this.source.length) {
      // Signal error by storing 0xDEADBEEF
      store<u32>(RESULT_HEAP, 0xDEADBEEF);
      unreachable();
    }
  }
  
  u8(): u8 {
    this.ensureBytes(1);
    const v = this.source[this.offset];
    this.offset += 1;
    return v;
  }
  
  u32(): u32 {
    this.ensureBytes(4);
    let v: u32 = this.source[this.offset];
    v |= u32(this.source[this.offset + 1]) << 8;
    v |= u32(this.source[this.offset + 2]) << 16;
    v |= u32(this.source[this.offset + 3]) << 24;
    this.offset += 4;
    return v;
  }
  
  bytes(len: u32): Uint8Array {
    this.ensureBytes(len);
    const result = this.source.subarray(this.offset, this.offset + len);
    this.offset += len;
    return result;
  }
  
  getOffset(): i32 {
    return this.offset;
  }
  
  getLength(): i32 {
    return this.source.length;
  }
}

// Simplified liftBytes
function liftBytes(data: u8[]): Uint8Array {
  const p = new Uint8Array(data.length);
  p.set(data, 0);
  return p;
}

export function main(args_ptr: i32, args_len: i32): void {
  RESULT_HEAP = heap.alloc(256);
  let out_offset: u32 = 0;
  
  // Store input info
  store<u32>(RESULT_HEAP + out_offset, args_ptr as u32);
  out_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, args_len as u32);
  out_offset += 4;
  
  // Read header: program_len (4 bytes) + data_len (4 bytes)
  const program_len = load<u32>(args_ptr);
  const data_len = load<u32>(args_ptr + 4);
  
  store<u32>(RESULT_HEAP + out_offset, program_len);
  out_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, data_len);
  out_offset += 4;
  
  // === Step 1: Copy program bytes to u8[] array (like index-compiler) ===
  const programArray: u8[] = [];
  for (let i: u32 = 0; i < program_len; i++) {
    programArray.push(load<u8>(args_ptr + 8 + i));
  }
  
  store<u32>(RESULT_HEAP + out_offset, programArray.length as u32);
  out_offset += 4;
  
  // === Step 2: Convert to Uint8Array (like liftBytes) ===
  const program = liftBytes(programArray);
  
  store<u32>(RESULT_HEAP + out_offset, program.length as u32);
  out_offset += 4;
  
  // Store first few bytes of program for verification
  for (let i = 0; i < program.length && i < 4; i++) {
    store<u8>(RESULT_HEAP + out_offset, program[i]);
    out_offset += 1;
  }
  while (out_offset % 4 != 0) {
    store<u8>(RESULT_HEAP + out_offset, 0);
    out_offset += 1;
  }
  
  // === Step 3: Create Decoder and read values (like decodeSpi) ===
  const decoder = new Decoder(program);
  
  // Read first u8
  const first_u8 = decoder.u8();
  store<u8>(RESULT_HEAP + out_offset, first_u8);
  out_offset += 1;
  while (out_offset % 4 != 0) {
    store<u8>(RESULT_HEAP + out_offset, 0);
    out_offset += 1;
  }
  
  // Read remaining bytes
  const remaining = decoder.bytes(program_len - 1);
  store<u32>(RESULT_HEAP + out_offset, remaining.length as u32);
  out_offset += 4;
  
  for (let i = 0; i < remaining.length && i < 4; i++) {
    store<u8>(RESULT_HEAP + out_offset, remaining[i]);
    out_offset += 1;
  }
  while (out_offset % 4 != 0) {
    store<u8>(RESULT_HEAP + out_offset, 0);
    out_offset += 1;
  }
  
  // Store decoder state
  store<u32>(RESULT_HEAP + out_offset, decoder.getOffset() as u32);
  out_offset += 4;
  store<u32>(RESULT_HEAP + out_offset, decoder.getLength() as u32);
  out_offset += 4;
  
  result_ptr = RESULT_HEAP as i32;
  result_len = out_offset as i32;
}
