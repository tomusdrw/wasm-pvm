#!/usr/bin/env npx tsx
/**
 * Test PVM-in-PVM execution.
 * 
 * This runs a simple PVM program through the compiled anan-as interpreter
 * which is itself running in another PVM interpreter.
 * 
 * Usage: npx tsx scripts/test-pvm-in-pvm.ts
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

// The compiled anan-as with main() entry point
const OUTER_PVM_PATH = '/tmp/anan-pvm.jam';

// Simple test programs
const INNER_ADD_PATH = '/tmp/inner-add.jam';

async function main() {
  console.log('=== PVM-in-PVM Test ===\n');
  
  // First, compile the inner test program if needed
  const { execSync } = await import('node:child_process');
  
  console.log('Compiling inner test program (add.jam.wat)...');
  execSync('cargo run -p wasm-pvm-cli --quiet -- compile examples-wat/add.jam.wat -o /tmp/inner-add.jam', {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit'
  });
  
  console.log('Compiling outer PVM interpreter (anan-as)...');
  execSync('cargo run -p wasm-pvm-cli --quiet -- compile vendor/anan-as/build/release.wasm -o /tmp/anan-pvm.jam', {
    cwd: path.join(__dirname, '..'),
    stdio: 'inherit'
  });
  
  // Load the programs
  const outerPvm = fs.readFileSync(OUTER_PVM_PATH);
  const innerPvm = fs.readFileSync(INNER_ADD_PATH);
  
  console.log(`\nOuter PVM (anan-as): ${outerPvm.length} bytes`);
  console.log(`Inner PVM (add): ${innerPvm.length} bytes`);
  
  // The inner program expects args at 0xFEFF0000 with two u32 values
  // We need to create a "generic" PVM blob (not SPI) for resetGeneric
  // So we need to extract just the PVM blob from the inner SPI file
  
  // Parse inner SPI to get the PVM blob
  const innerBlob = extractPvmBlob(innerPvm);
  console.log(`Inner PVM blob: ${innerBlob.length} bytes`);
  
  // Build the input args for the outer PVM:
  // - program_len: u32 (4 bytes)
  // - program: u8[program_len] (PVM blob)
  // - registers: u8[104] (13 * 8 bytes)
  // - gas: u64 (8 bytes)
  // - steps: u32 (4 bytes)
  
  // For the add program, we need to set up:
  // - r7 = args pointer (0xFEFF0000)
  // - r8 = args length (8 bytes for two u32s)
  // - The args themselves need to be in memory at 0xFEFF0000
  
  // But wait - the inner interpreter (resetGeneric) expects a "generic" PVM blob,
  // not an SPI format. And it won't have access to the outer memory layout.
  
  // This is getting complex. Let me simplify by testing with a simpler inner program
  // that doesn't need external memory - just does some computation with registers.
  
  console.log('\n--- Testing with simple register-only computation ---');
  
  // Create a minimal PVM program that:
  // - Takes r9 and r10 as inputs
  // - Computes r9 + r10 -> r11
  // - Halts
  
  // PVM instructions:
  // ADD_64 r11, r9, r10  (opcode 200, ThreeReg encoding)
  // JUMP 0xFFFF0000      (to halt)
  
  const simplePvmCode = createSimpleAddProgram();
  console.log(`Simple add program: ${simplePvmCode.length} bytes`);
  
  // Create registers: r9=5, r10=7, others=0
  const registers = new ArrayBuffer(104); // 13 * 8 bytes
  const regView = new DataView(registers);
  regView.setBigUint64(9 * 8, 5n, true);  // r9 = 5
  regView.setBigUint64(10 * 8, 7n, true); // r10 = 7
  regView.setBigUint64(0 * 8, BigInt(0xFFFF0000), true); // r0 = EXIT address
  
  // Gas and steps
  const gas = 1000000n;
  const steps = 1000;
  
  // Pack the input
  const inputArgs = packInput(simplePvmCode, new Uint8Array(registers), gas, steps);
  console.log(`Input args for outer PVM: ${inputArgs.length} bytes`);
  
  // Load anan-as to run the outer PVM
  const ananAs = await import(ananAsPath);
  
  // Run the outer PVM (anan-as compiled to PVM)
  console.log('\nRunning outer PVM (anan-as interpreter)...');
  
  const outerProgram = ananAs.prepareProgram(
    ananAs.InputKind.SPI,
    ananAs.HasMetadata.No,
    Array.from(outerPvm),
    [],
    [],
    [],
    Array.from(inputArgs)
  );
  
  const outerGas = BigInt(100_000_000);
  const outerResult = ananAs.runProgram(outerProgram, outerGas, 0, false);
  
  console.log(`\nOuter PVM execution:`);
  console.log(`  Status: ${statusToString(outerResult.status)}`);
  console.log(`  Exit Code: ${outerResult.exitCode}`);
  console.log(`  PC: ${outerResult.pc}`);
  console.log(`  Gas Used: ${outerGas - outerResult.gas}`);
  
  if (outerResult.status !== 0) {
    console.log('\n❌ Outer PVM did not halt successfully');
    
    // Show some registers for debugging
    console.log('\nOuter registers:');
    for (let i = 0; i < 13; i++) {
      console.log(`  r${i}: ${outerResult.registers[i]} (0x${outerResult.registers[i].toString(16)})`);
    }
    
    process.exit(1);
  }
  
  // Read the result from outer PVM's output
  // result_ptr is in r7, result_len is in r8
  const resultAddr = Number(outerResult.registers[7]);
  const resultLen = Number(outerResult.registers[8] & 0xffffffffn);
  
  console.log(`\nOuter result pointer: 0x${resultAddr.toString(16)}, length: ${resultLen}`);
  
  if (resultLen > 0 && outerResult.memory) {
    const resultBytes = readMemoryFromChunks(outerResult.memory, resultAddr, resultLen);
    console.log(`Result bytes: ${Array.from(resultBytes).map(b => b.toString(16).padStart(2, '0')).join(' ')}`);
    
    // Parse the result:
    // - status: u8 (1 byte)
    // - pc: u32 (4 bytes)
    // - gas_left: u64 (8 bytes)
    // - registers: u8[104]
    
    if (resultLen >= 13) {
      const view = new DataView(new Uint8Array(resultBytes).buffer);
      const innerStatus = view.getUint8(0);
      const innerPc = view.getUint32(1, true);
      const innerGasLeft = view.getBigUint64(5, true);
      
      console.log(`\nInner PVM result:`);
      console.log(`  Status: ${statusToString(innerStatus)}`);
      console.log(`  PC: ${innerPc}`);
      console.log(`  Gas Left: ${innerGasLeft}`);
      
      if (resultLen >= 117) {
        // Read inner registers
        console.log(`  Inner registers:`);
        for (let i = 0; i < 13; i++) {
          const reg = view.getBigUint64(13 + i * 8, true);
          if (reg !== 0n) {
            console.log(`    r${i}: ${reg}`);
          }
        }
        
        // Check if r11 = 12 (5 + 7)
        const r11 = view.getBigUint64(13 + 11 * 8, true);
        if (r11 === 12n) {
          console.log('\n✅ PVM-in-PVM SUCCESS! r11 = 5 + 7 = 12');
        } else {
          console.log(`\n❌ Expected r11=12, got r11=${r11}`);
        }
      }
    }
  }
}

function extractPvmBlob(spiData: Buffer): Uint8Array {
  // Parse SPI header to find the PVM blob
  let offset = 0;
  
  // roLength (3 bytes)
  const roLength = spiData[offset] | (spiData[offset + 1] << 8) | (spiData[offset + 2] << 16);
  offset += 3;
  
  // rwLength (3 bytes)
  const rwLength = spiData[offset] | (spiData[offset + 1] << 8) | (spiData[offset + 2] << 16);
  offset += 3;
  
  // heapPages (2 bytes)
  offset += 2;
  
  // stackSize (3 bytes)
  offset += 3;
  
  // Skip RO data
  offset += roLength;
  
  // Skip RW data
  offset += rwLength;
  
  // codeLength (4 bytes)
  const codeLength = spiData[offset] | (spiData[offset + 1] << 8) | (spiData[offset + 2] << 16) | (spiData[offset + 3] << 24);
  offset += 4;
  
  // The PVM blob starts here
  return spiData.subarray(offset, offset + codeLength);
}

function createSimpleAddProgram(): Uint8Array {
  // Create a minimal PVM program:
  // 1. ADD_64 r11, r9, r10  - compute r11 = r9 + r10
  // 2. LOAD_IMM r2, 0xFFFF0000  - load exit address
  // 3. JUMP_IND r2, 0  - jump to exit
  
  // PVM blob format:
  // - jumpTableLength (varU32)
  // - jumpTableItemBytes (u8)
  // - codeLength (varU32)
  // - jumpTable (jumpTableLength * jumpTableItemBytes bytes)
  // - code (codeLength bytes)
  // - mask ((codeLength + 7) / 8 bytes)
  
  const code: number[] = [];
  const mask: number[] = [];
  
  // ADD_64: opcode 200, ThreeReg encoding
  // reg[c] = reg[b] + reg[a]
  // We want: r11 = r9 + r10
  // Encoding: [200] [a<<4 | b] [c]
  // Where a=src1, b=src2, c=dst
  // So: a=9, b=10, c=11 -> [200] [0x9A] [0x0B]
  code.push(200);  // ADD_64 opcode
  code.push((9 << 4) | 10);  // src1=r9, src2=r10
  code.push(11);  // dst=r11
  
  // LOAD_IMM r2, 0xFFFF0000: opcode 51, OneRegOneImm encoding
  // Encoding: [51] [reg_nibble] [imm bytes...]
  // For -65536 (0xFFFF0000 as i32), we need 4 bytes
  code.push(51);  // LOAD_IMM opcode
  code.push(2);   // r2
  code.push(0x00);  // imm byte 0
  code.push(0x00);  // imm byte 1
  code.push(0xFF);  // imm byte 2
  code.push(0xFF);  // imm byte 3
  
  // JUMP_IND r2, 0: opcode 50, OneRegOneImm encoding
  code.push(50);  // JUMP_IND opcode
  code.push(2);   // r2
  code.push(0);   // offset = 0
  
  // Build mask: 1 bit per byte, 1 = opcode start
  // Positions: 0 (ADD_64), 3 (LOAD_IMM), 9 (JUMP_IND)
  // Byte 0: positions 0-7 -> bits 0,3 set = 0b00001001 = 0x09
  // Byte 1: positions 8-15 -> bit 9-8=1 set = 0b00000010 = 0x02
  const maskLen = Math.ceil(code.length / 8);
  for (let i = 0; i < maskLen; i++) {
    mask.push(0);
  }
  // Set instruction start bits
  setBit(mask, 0);  // ADD_64 at position 0
  setBit(mask, 3);  // LOAD_IMM at position 3
  setBit(mask, 9);  // JUMP_IND at position 9
  
  // Build the blob
  const blob: number[] = [];
  
  // jumpTableLength = 0 (no jump table needed)
  blob.push(0);
  
  // jumpTableItemBytes = 0
  blob.push(0);
  
  // codeLength (varU32 encoding)
  if (code.length < 0x80) {
    blob.push(code.length);
  } else {
    // For larger code, use multi-byte encoding
    throw new Error('Code too long for simple varU32');
  }
  
  // No jump table entries
  
  // Code
  blob.push(...code);
  
  // Mask
  blob.push(...mask);
  
  return new Uint8Array(blob);
}

function setBit(mask: number[], bitIndex: number): void {
  const byteIndex = Math.floor(bitIndex / 8);
  const bitOffset = bitIndex % 8;
  mask[byteIndex] |= (1 << bitOffset);
}

function packInput(program: Uint8Array, registers: Uint8Array, gas: bigint, steps: number): Uint8Array {
  const totalLen = 4 + program.length + 104 + 8 + 4;
  const buffer = new ArrayBuffer(totalLen);
  const view = new DataView(buffer);
  const bytes = new Uint8Array(buffer);
  
  let offset = 0;
  
  // program_len (4 bytes)
  view.setUint32(offset, program.length, true);
  offset += 4;
  
  // program
  bytes.set(program, offset);
  offset += program.length;
  
  // registers (104 bytes)
  bytes.set(registers, offset);
  offset += 104;
  
  // gas (8 bytes)
  view.setBigUint64(offset, gas, true);
  offset += 8;
  
  // steps (4 bytes)
  view.setUint32(offset, steps, true);
  
  return bytes;
}

function statusToString(status: number): string {
  const names: Record<number, string> = {
    [-1]: 'OK (running)',
    0: 'HALT',
    1: 'PANIC',
    2: 'FAULT',
    3: 'HOST',
    4: 'OOG (out of gas)',
  };
  return names[status] ?? `UNKNOWN(${status})`;
}

interface MemoryChunk {
  address: number;
  data: number[];
}

function readMemoryFromChunks(chunks: MemoryChunk[], address: number, length: number): number[] {
  const result: number[] = new Array(length).fill(0);
  
  for (const chunk of chunks) {
    const chunkStart = chunk.address;
    const chunkEnd = chunkStart + chunk.data.length;
    
    const overlapStart = Math.max(address, chunkStart);
    const overlapEnd = Math.min(address + length, chunkEnd);
    
    if (overlapStart < overlapEnd) {
      for (let i = overlapStart; i < overlapEnd; i++) {
        result[i - address] = chunk.data[i - chunkStart];
      }
    }
  }
  
  return result;
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
