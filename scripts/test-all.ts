#!/usr/bin/env bun
/**
 * Automated test suite for wasm-pvm examples using anan-as CLI
 * Usage: bun scripts/test-all.ts [--filter=pattern] [--verbose]
 */

import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { testCases } from './test-cases.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');
const examplesWatDir = path.join(projectRoot, 'examples-wat');
const examplesAsBuildDir = path.join(projectRoot, 'examples-as', 'build');

function compileWatIfAvailable(testName: string, jamFile: string): void {
  const watFile = path.join(examplesWatDir, `${testName}.jam.wat`);
  if (!fs.existsSync(watFile)) {
    return;
  }

  console.time('compile');
  const cmd = `cargo run -p wasm-pvm-cli --release -- compile ${watFile} -o ${jamFile}`;
  execSync(cmd, {
    cwd: projectRoot,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe']
  });
  console.timeEnd('compile');
}

function compileAsIfAvailable(testName: string, jamFile: string): void {
  // AssemblyScript tests are prefixed with 'as-'
  if (!testName.startsWith('as-')) {
    return;
  }
  
  // Extract the base name (e.g., 'as-add' -> 'add')
  const baseName = testName.slice(3);
  const wasmFile = path.join(examplesAsBuildDir, `${baseName}.wasm`);
  
  if (!fs.existsSync(wasmFile)) {
    console.log(`  ⚠️  WASM file not found: ${wasmFile}`);
    return;
  }

  const cmd = `cargo run -p wasm-pvm-cli -- compile ${wasmFile} -o ${jamFile}`;
  execSync(cmd, {
    cwd: projectRoot,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe']
  });
}

function runJamFile(jamFile: string, args: string, pc?: number): number {
  // Build anan-as CLI command
  // Format: anan-as run --spi --no-metadata --no-logs [--pc <n>] <file.jam> 0x<hex-args>
  const ananAsCli = path.join(projectRoot, 'vendor/anan-as/dist/bin/index.js');
  let cmd = `node ${ananAsCli} run --spi --no-metadata --no-logs`;
  
  if (pc !== undefined) {
    cmd += ` --pc=${pc}`;
  }
  
  // Add gas (default 100M to match previous behavior)
  cmd += ` --gas=100000000`;
  
  // Add file and hex args (with 0x prefix)
  cmd += ` ${jamFile} 0x${args}`;

  try {
    console.time('run');
    const output = execSync(cmd, {
      cwd: projectRoot,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    console.timeEnd('run');

    // Parse the hex result format: Result: [0x0c000000]
    // The result is a little-endian hex array, extract and convert to decimal
    const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]+)\]/);

    if (resultMatch) {
      let hexResult = resultMatch[1];
      
      // Normalize to exactly 8 hex chars
      if (hexResult.length < 8) {
        hexResult = hexResult.padEnd(8, '0');
      } else if (hexResult.length > 8) {
        hexResult = hexResult.slice(0, 8);
      }

      const bytes = hexResult.match(/.{2}/g) || [];
      // Little-endian: bytes are in reverse order
      const value = parseInt(bytes.reverse().join(''), 16);
      return value;
    }

    throw new Error(`Could not parse result from output: ${output}`);
  } catch (error: any) {
    if (error.stdout) console.log(error.stdout.toString());
    if (error.stderr) console.error(error.stderr.toString());
    throw new Error(`Execution failed: ${error.message.split('\n')[0]}`, { cause: error });
  }
}

async function main() {
  const args = process.argv.slice(2);
  let filter: string | undefined;

  for (const arg of args) {
    if (arg.startsWith('--filter=')) {
      filter = arg.slice(9);
    }
  }

  console.log('=== WASM-PVM Test Suite ===\n');
  console.log('Testing JAM-SPI programs using anan-as CLI...\n');

  let totalTests = 0;
  let passedTests = 0;
  let failedTests = 0;
  const failures: string[] = [];

  for (const testCase of testCases) {
    if (filter && !testCase.name.includes(filter)) {
      continue;
    }

    console.log(`Testing ${testCase.name}...`);
    const jamFile = path.join(projectRoot, 'dist', `${testCase.name}.jam`);

    compileWatIfAvailable(testCase.name, jamFile);
    compileAsIfAvailable(testCase.name, jamFile);

    // Check if JAM file exists
    if (!fs.existsSync(jamFile)) {
      console.log(`  ❌ JAM file not found: ${jamFile}`);
      failures.push(`${testCase.name}: JAM file not found`);
      failedTests += testCase.tests.length;
      totalTests += testCase.tests.length;
      continue;
    }

    for (const test of testCase.tests) {
      totalTests++;

      try {
        const actual = runJamFile(jamFile, test.args, test.pc);

        if (actual === test.expected) {
          console.log(`  ✓ ${test.description}`);
          passedTests++;
        } else {
          console.log(`  ❌ ${test.description} - expected ${test.expected}, got ${actual}`);
          failures.push(`${testCase.name}: ${test.description} - expected ${test.expected}, got ${actual}`);
          failedTests++;
        }
      } catch (err: any) {
        console.log(`  ❌ ${test.description} - execution failed: ${err.message}`);
        failures.push(`${testCase.name}: ${test.description} - execution failed`);
        failedTests++;
      }
    }
    console.log();
  }

  console.log('=== Summary ===');
  console.log(`Total: ${totalTests}, Passed: ${passedTests}, Failed: ${failedTests}`);

  if (failures.length > 0) {
    console.log('\nFailures:');
    for (const failure of failures) {
      console.log(`  - ${failure}`);
    }
    process.exit(1);
  } else {
    console.log('\n✓ All tests passed!');
    process.exit(0);
  }
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
