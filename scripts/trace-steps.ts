#!/usr/bin/env bun
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

async function main() {
  const jamFile = process.argv[2] || '/tmp/fact.jam';
  const argsHex = process.argv[3] || '05000000';
  const maxSteps = parseInt(process.argv[4] || '50', 10);
  
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
  
  // Run with VERY verbose output (step by step)
  const output = ananAs.runProgram(program, BigInt(maxSteps), 0, true);
}

function hexToBytes(hex: string): number[] {
  const result: number[] = [];
  for (let i = 0; i < hex.length; i += 2) {
    result.push(parseInt(hex.slice(i, i + 2), 16));
  }
  return result;
}

main().catch(console.error);
