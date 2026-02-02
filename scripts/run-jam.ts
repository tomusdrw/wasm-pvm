#!/usr/bin/env bun
/**
 * Run JAM files with hex arguments using anan-as library
 * Usage: bun scripts/run-jam.ts <jam-file> --args=<hex-args> [--pc=<pc>] [--gas=<gas>]
 */

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Import anan-as library
// We need to use dynamic import or require because it might use top-level await
// and we want to handle path resolution
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/dist/build/release.js');

function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new Error('Hex string must have even length');
  }

  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.slice(i, i + 2), 16);
  }
  return bytes;
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length < 1) {
    console.error('Usage: bun scripts/run-jam.ts <jam-file> --args=<hex-args> [--pc=<pc>] [--gas=<gas>]');
    process.exit(1);
  }

  const jamFile = args[0];
  let hexArgs = '';
  let pc = 0;
  let gas = 100_000_000n; // Default 100M gas

  for (let i = 1; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith('--args=')) {
      hexArgs = arg.slice(7);
    } else if (arg.startsWith('--pc=')) {
      pc = parseInt(arg.slice(5), 10);
    } else if (arg.startsWith('--gas=')) {
      gas = BigInt(arg.slice(6));
    } else {
      console.error(`Unknown argument: ${arg}`);
      process.exit(1);
    }
  }

  // Read JAM file
  const jamData = readFileSync(jamFile);

  // Convert hex args to binary
  const argsData = hexArgs ? hexToBytes(hexArgs) : new Uint8Array(0);

  try {
    // Import anan-as
    const ananAs = await import(ananAsPath);
    
    // anan-as expects Array<number>, not Uint8Array
    const program = Array.from(jamData);
    const programArgs = Array.from(argsData);

    console.log(`🚀 Running ${jamFile} (as JAM SPI)`);
    
    const { InputKind, HasMetadata, prepareProgram, runProgram } = ananAs;
    
    const prog = prepareProgram(
        InputKind.SPI, 
        HasMetadata.No, 
        program, 
        [], [], [], 
        programArgs
    );
    
    console.time('runProgram');
    const result = runProgram(prog, gas, pc, false, true);
    console.timeEnd('runProgram');

    console.log(`Status: ${result.status}`);
    console.log(`Exit code: ${result.exitCode}`);
    console.log(`Program counter: ${result.pc}`);
    console.log(`Gas remaining: ${result.gas}`);
    console.log(`Registers: [${result.registers.join(', ')}]`);
    
    if (result.status === 0) { // HALT
        // Manually read result from memory based on r7/r8
        const r7 = Number(result.registers[7]);
        const r8 = Number(result.registers[8]);
        
        console.log(`Result Ptr (r7): 0x${r7.toString(16)}`);
        console.log(`Result End (r8): 0x${r8.toString(16)}`);
        
        // Wait, anan-as expects r8 to be END POINTER?
        // "const ptr_end = output.registers[8];"
        // "if (ptr_start >= ptr_end)"
        
        // If I changed my compiler to set r8 = start + len.
        // Then r8 IS end pointer.
        
        const ptr_start = r7;
        const ptr_end = r8;
        
         if (ptr_start >= ptr_end) {
              console.log("Result range empty or invalid");
         } else {
              const resultLength = ptr_end - ptr_start;
              const resultBytes = new Uint8Array(resultLength);
              console.log(`Memory chunks: ${result.memory.length}`);

              for (const chunk of result.memory) {
                  const start = chunk.address;
                  const end = start + chunk.data.length;
                  console.log(`Chunk: [0x${start.toString(16)}, 0x${end.toString(16)}]`);

                  const overlapStart = Math.max(ptr_start, start);
                  const overlapEnd = Math.min(ptr_end, end);
                  if (overlapStart >= overlapEnd) {
                      continue;
                  }

                  const chunkBytes = Uint8Array.from(chunk.data);
                  const chunkOffsetStart = overlapStart - start;
                  const chunkOffsetEnd = overlapEnd - start;
                  const resultOffset = overlapStart - ptr_start;

                  resultBytes.set(
                      chunkBytes.subarray(chunkOffsetStart, chunkOffsetEnd),
                      resultOffset,
                  );
              }

              if (resultBytes.length === 4) {
                  const view = new DataView(resultBytes.buffer);
                  console.log(`Result: ${view.getUint32(0, true)}`);
              } else {
                  console.log(`Result (hex): ${Buffer.from(resultBytes).toString('hex')}`);
              }
         }
    } else {
        console.log(`Execution failed with status ${result.status}`);
    }

  } catch (error: any) {
    console.error('Failed to run JAM file:', error);
    process.exit(1);
  }
}

main();
