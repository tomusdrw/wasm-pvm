// Mini PVM Runner: Simplified version of index-compiler.ts to debug the issue
// This mimics the exact input parsing that index-compiler does

const RESULT_BUFFER: u32 = 0x100;
const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Status codes
const STATUS_OK: u8 = 0;
const STATUS_PANIC: u8 = 1;

function writeStatus(status: u8): void {
  store<u8>(RESULT_BUFFER, status);
  result_ptr = RESULT_BUFFER;
  result_len = 1;
}

function writeDiagnostics(offset: u32, msg_code: u32, val1: u32, val2: u32, val3: u32): u32 {
  store<u32>(RESULT_BUFFER + offset, msg_code);
  store<u32>(RESULT_BUFFER + offset + 4, val1);
  store<u32>(RESULT_BUFFER + offset + 8, val2);
  store<u32>(RESULT_BUFFER + offset + 12, val3);
  return offset + 16;
}

export function main(argsPtr: i32, argsLen: i32): void {
  // Input format from index-compiler.ts:
  // 8 (gas) + 4 (pc) + 4 (spi-program-len) + 4 (inner-args-len) + ? (spi-program) + ? (inner-args)
  
  let out_offset: u32 = 0;
  
  // Write args info for debugging
  out_offset = writeDiagnostics(out_offset, 0x11111111, argsPtr as u32, argsLen as u32, 0);
  
  if (argsLen < 20) {
    out_offset = writeDiagnostics(out_offset, 0xDEAD0001, argsLen as u32, 20, 0);
    result_ptr = RESULT_BUFFER;
    result_len = out_offset as i32;
    return;
  }
  
  let read_offset: u32 = 0;
  
  // Read gas (8 bytes, little-endian u64)
  const gasLow = load<u32>(argsPtr + read_offset);
  const gasHigh = load<u32>(argsPtr + read_offset + 4);
  read_offset += 8;
  out_offset = writeDiagnostics(out_offset, 0x22222222, gasLow, gasHigh, read_offset);
  
  // Read pc (4 bytes)
  const pc = load<u32>(argsPtr + read_offset);
  read_offset += 4;
  out_offset = writeDiagnostics(out_offset, 0x33333333, pc, read_offset, 0);
  
  // Read program_len (4 bytes)
  const programLen = load<u32>(argsPtr + read_offset);
  read_offset += 4;
  out_offset = writeDiagnostics(out_offset, 0x44444444, programLen, read_offset, 0);
  
  // Read inner_args_len (4 bytes)
  const innerArgsLen = load<u32>(argsPtr + read_offset);
  read_offset += 4;
  out_offset = writeDiagnostics(out_offset, 0x55555555, innerArgsLen, read_offset, 0);
  
  // Validate lengths
  const expectedLen = read_offset + programLen + innerArgsLen;
  if (argsLen as u32 != expectedLen) {
    out_offset = writeDiagnostics(out_offset, 0xDEAD0002, argsLen as u32, expectedLen, read_offset);
    result_ptr = RESULT_BUFFER;
    result_len = out_offset as i32;
    return;
  }
  
  // Copy program bytes to array (like index-compiler)
  const spiProgram: u8[] = [];
  for (let i: u32 = 0; i < programLen; i++) {
    spiProgram.push(load<u8>(argsPtr + read_offset + i));
  }
  read_offset += programLen;
  out_offset = writeDiagnostics(out_offset, 0x66666666, spiProgram.length as u32, read_offset, 0);
  
  // Store first bytes of program
  for (let i = 0; i < spiProgram.length && i < 8; i++) {
    store<u8>(RESULT_BUFFER + out_offset + i, spiProgram[i]);
  }
  out_offset += 8;
  
  // Copy inner args
  const innerArgs: u8[] = [];
  for (let i: u32 = 0; i < innerArgsLen; i++) {
    innerArgs.push(load<u8>(argsPtr + read_offset + i));
  }
  read_offset += innerArgsLen;
  out_offset = writeDiagnostics(out_offset, 0x77777777, innerArgs.length as u32, read_offset, 0);
  
  // Store first bytes of inner args  
  for (let i = 0; i < innerArgs.length && i < 8; i++) {
    store<u8>(RESULT_BUFFER + out_offset + i, innerArgs[i]);
  }
  out_offset += 8;
  
  // Success marker
  out_offset = writeDiagnostics(out_offset, 0xAAAAAAAA, 0, 0, 0);
  
  result_ptr = RESULT_BUFFER;
  result_len = out_offset as i32;
}
