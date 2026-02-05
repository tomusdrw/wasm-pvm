#!/usr/bin/env bun
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

async function main() {
  const jamFile = process.argv[2];
  const argsHex = process.argv[3] || '';
  
  if (!jamFile) {
    console.error('Usage: debug-dump.ts <jam-file> [args-hex]');
    process.exit(1);
  }

  const ananAs = await import(ananAsPath);
  const spiData = fs.readFileSync(jamFile);
  const argsBytes = [];
  for (let i = 0; i < argsHex.length; i += 2) {
    argsBytes.push(parseInt(argsHex.slice(i, i + 2), 16));
  }
  
  console.log(`Loading ${jamFile} (${spiData.length} bytes)`);
  
  const program = ananAs.prepareProgram(
    ananAs.InputKind.SPI,
    ananAs.HasMetadata.No,
    Array.from(spiData),
    [],
    [],
    [],
    argsBytes
  );
  
  console.log('Running...');
  // Run with plenty of gas
  const output = ananAs.runProgram(program, 200_000_000n, 0, false);
  
  console.log(`Status: ${output.status}`);
  console.log(`Exit Code: ${output.exitCode}`);
  console.log(`Gas Remaining: ${output.gas}`);
  console.log(`PC: ${output.pc}`);
  console.log('Output object:', output);
  
  // Reconstruct memory from sparse chunks
  // The anan-as/interpreter output returns modified memory chunks
  const memMap = new Map<number, number>(); // byte address -> value
  
  for (const chunk of output.memory) {
      for (let i = 0; i < chunk.data.length; i++) {
          memMap.set(chunk.address + i, chunk.data[i]);
      }
  }
  
  console.log(`\nReconstructed memory map with ${memMap.size} bytes`);
  
  console.log('\nDebug Dump (0x30100):');
  let offset = 0;
  let buffer = [];
  
  // Read 4KB of debug data
  for (let i = 0; i < 4096; i += 4) {
    const addr = 0x30100 + i;
    
    // Read 4 bytes
    let val = 0;
    let hasData = false;
    
    if (memMap.has(addr)) {
        val |= memMap.get(addr)!;
        hasData = true;
    }
    if (memMap.has(addr+1)) {
        val |= memMap.get(addr+1)! << 8;
        hasData = true;
    }
    if (memMap.has(addr+2)) {
        val |= memMap.get(addr+2)! << 16;
        hasData = true;
    }
    if (memMap.has(addr+3)) {
        val |= memMap.get(addr+3)! << 24;
        hasData = true;
    }
    
    if (hasData) {
        if (val != 0) {
            console.log(`0x${addr.toString(16)}: 0x${val.toString(16).padStart(8, '0')} (${val})`);
            buffer.push(val);
        } else if (buffer.length > 0 && buffer[buffer.length-1] != 0) {
             // stop after seeing zeros following data
             if (i > 1000) break; 
        }
    }
  }
  
  // Also print raw hex string for python script
  const hexStr = buffer.map(v => v.toString(16).padStart(8, '0')).map(s => s.match(/../g).reverse().join('')).join('');
  console.log('\nRaw Hex:');
  console.log(hexStr);
}

main().catch(console.error);
