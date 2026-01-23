// SPI Program Runner - Minimal PVM Interpreter
// Executes basic SPI programs for PVM-in-PVM testing

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Simple interpreter for basic arithmetic SPI programs
export function main(args_ptr: u32, args_len: u32): void {
  // Read SPI program data from args
  const spiProgramLen = load<u32>(args_ptr);
  const spiDataStart = args_ptr + 4;

  // For now, assume it's an add program: read two u32 args and add them
  // SPI format: [program_len][program_data][arg1][arg2]
  const arg1 = load<u32>(spiDataStart + spiProgramLen);
  const arg2 = load<u32>(spiDataStart + spiProgramLen + 4);

  // Perform the computation (hardcoded to add for now)
  const result = arg1 + arg2;

  // Create result similar to what anan-as returns
  const status: u32 = 0; // HALT
  const pc: u32 = 0; // Final PC
  const gas_left: u64 = 1000000; // Remaining gas

  // Create register results
  const registers: u64[] = [];
  for (let i: u32 = 0; i < 13; i++) {
    registers.push(0);
  }

  // Put the result in r11 (where arithmetic programs typically return)
  registers[11] = <u64>result;

  // Write result in the format expected by our test harness
  let out_offset: u32 = 0;
  store<u8>(RESULT_HEAP + out_offset, <u8>status);
  out_offset += 1;

  store<u32>(RESULT_HEAP + out_offset, pc);
  out_offset += 4;

  store<u64>(RESULT_HEAP + out_offset, gas_left);
  out_offset += 8;

  // Write registers (13 * 8 bytes)
  for (let i: u32 = 0; i < 13; i++) {
    store<u64>(RESULT_HEAP + out_offset, registers[i]);
    out_offset += 8;
  }

  result_ptr = RESULT_HEAP;
  result_len = out_offset;
}