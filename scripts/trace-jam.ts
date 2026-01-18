#!/usr/bin/env npx tsx
/**
 * Detailed tracing for debugging infinite loops
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

async function main() {
  const jamFile = process.argv[2] || '/tmp/life.jam';
  const argsHex = process.argv[3] || '01000000';
  const maxSteps = parseInt(process.argv[4] || '50000', 10);
  
  const ananAs = await import(ananAsPath);
  const spiData = fs.readFileSync(jamFile);
  const argsBytes = hexToBytes(argsHex);
  
  const program = ananAs.prepareProgram(
    ananAs.InputKind.SPI,
    ananAs.HasMetadata.No,
    Array.from(spiData),
    [],
    [],
    [],
    argsBytes
  );
  
  // Run with tracing enabled (using verbose mode of existing infra)
  // Note: this doesn't give us per-step control, but we can use runProgram with logging
  const output = ananAs.runProgram(program, BigInt(maxSteps), 0, true);
  
  console.log('\n=== Final State ===');
  console.log(`Status: ${output.status}`);
  console.log(`Exit Code: ${output.exitCode}`);
  console.log(`PC: ${output.pc}`);
  console.log(`Gas Remaining: ${output.gas}`);
  console.log('Registers:', output.registers.map((v, i) => `r${i}=${v}`).join(', '));
}

function hexToBytes(hex: string): number[] {
  const result: number[] = [];
  for (let i = 0; i < hex.length; i += 2) {
    result.push(parseInt(hex.slice(i, i + 2), 16));
  }
  return result;
}

main().catch(console.error);
