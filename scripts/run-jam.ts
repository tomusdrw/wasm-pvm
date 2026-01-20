#!/usr/bin/env npx tsx
/**
 * Usage: npx tsx scripts/run-jam.ts <jam-file> [--pc=0] [--args=hex] [--gas=1000000] [--verbose]
 * Example: npx tsx scripts/run-jam.ts output.jam --args=0102030405 --gas=100000
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

async function main() {
  const args = process.argv.slice(2);
  
  let spiFile: string | null = null;
  let pc = 0;
  let inputArgs: number[] = [];
  let gas = BigInt(10_000_000);
  let verbose = false;
  
  for (const arg of args) {
    if (arg.startsWith('--pc=')) {
      pc = parseInt(arg.slice(5), 10);
    } else if (arg.startsWith('--args=')) {
      inputArgs = hexToBytes(arg.slice(7));
    } else if (arg.startsWith('--gas=')) {
      gas = BigInt(arg.slice(6));
    } else if (arg === '--verbose' || arg === '-v') {
      verbose = true;
    } else if (!arg.startsWith('-')) {
      spiFile = arg;
    }
  }
  
  if (!spiFile) {
    console.error('Usage: run-jam.ts <spi-file> [--pc=0] [--args=hex] [--gas=1000000] [--verbose]');
    process.exit(1);
  }
  
  if (!fs.existsSync(ananAsPath)) {
    console.error('Error: anan-as not built. Run: cd vendor/anan-as && npm ci && npm run build');
    process.exit(1);
  }
  
  const spiData = fs.readFileSync(spiFile);
  const ananAs = await import(ananAsPath);
  
  const program = ananAs.prepareProgram(
    ananAs.InputKind.SPI,
    ananAs.HasMetadata.No,
    Array.from(spiData),
    [],
    [],
    [],
    inputArgs
  );
  
  if (verbose) {
    console.log('=== Program Info ===');
    console.log(`PC: ${pc}`);
    console.log(`Gas: ${gas}`);
    console.log(`Args: [${inputArgs.map(b => '0x' + b.toString(16).padStart(2, '0')).join(', ')}]`);
    console.log();
    console.log('=== Disassembly ===');
    console.log(ananAs.disassemble(Array.from(spiData), ananAs.InputKind.SPI, ananAs.HasMetadata.No));
    console.log();
  }
  
  const output = ananAs.runProgram(program, gas, pc, verbose);
  
  console.log('=== Execution Result ===');
  console.log(`Status: ${statusToString(output.status)}`);
  console.log(`Exit Code: ${output.exitCode}`);
  console.log(`PC: ${output.pc}`);
  console.log(`Gas Remaining: ${output.gas}`);
  console.log();
  console.log('=== Registers ===');
  for (let i = 0; i < output.registers.length; i++) {
    const val = output.registers[i];
    console.log(`  r${i}: ${val} (0x${val.toString(16)})`);
  }
  
  // Show return value if present (r7=address, r8=length)
  const resultAddr = Number(output.registers[7]);
  const resultLen = Number(output.registers[8] & 0xffffffffn);
  if (output.status === 0 && resultLen > 0 && resultLen <= 1024) {
    console.log();
    console.log('=== Return Value ===');
    console.log(`  Address: 0x${resultAddr.toString(16)}`);
    console.log(`  Length: ${resultLen} bytes`);
    
    const resultBytes = readMemoryFromChunks(output.memory || [], resultAddr, resultLen);
    const hexStr = Array.from(resultBytes).map(b => b.toString(16).padStart(2, '0')).join(' ');
    console.log(`  Bytes: ${hexStr}`);
    if (resultLen === 4) {
      const value = new DataView(new Uint8Array(resultBytes).buffer).getUint32(0, true);
      console.log(`  As U32: ${value}`);
    } else if (resultLen === 8) {
      const value = new DataView(new Uint8Array(resultBytes).buffer).getBigUint64(0, true);
      console.log(`  As U64: ${value}`);
    }
  }
  
  process.exit(output.status === 0 ? 0 : 1);
}

function hexToBytes(hex: string): number[] {
  const bytes: number[] = [];
  const normalized = hex.startsWith('0x') || hex.startsWith('0X') ? hex.slice(2) : hex;
  for (let i = 0; i < normalized.length; i += 2) {
    bytes.push(parseInt(normalized.slice(i, i + 2), 16));
  }
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
