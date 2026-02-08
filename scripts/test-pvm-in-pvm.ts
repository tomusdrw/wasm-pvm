#!/usr/bin/env bun
/**
 * PVM-in-PVM Test Harness
 *
 * Compiles anan-as compiler to PVM, then runs test cases through the compiled anan-as-in-pvm.
 *
 * Usage: bun scripts/test-pvm-in-pvm.ts [--filter=pattern] [--verbose]
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'child_process';
import { testCases } from './test-cases.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');

// Paths for compiled artifacts
const ANAN_AS_COMPILER_WASM = path.join(projectRoot, 'vendor/anan-as/build/compiler.wasm');
const ANAN_AS_COMPILER_JAM = '/tmp/anan-as-compiler.jam';

function compileAnanAsToPvm(): void {
  console.log('Compiling anan-as compiler from WASM to PVM...');

  if (!fs.existsSync(ANAN_AS_COMPILER_WASM)) {
    throw new Error(`anan-as compiler WASM not found at ${ANAN_AS_COMPILER_WASM}. Run: cd vendor/anan-as && npm ci && npm run build`);
  }

  // Compile WASM to PVM using our wasm-pvm compiler
  const cmd = `cargo run -p wasm-pvm-cli -- compile ${ANAN_AS_COMPILER_WASM} --output ${ANAN_AS_COMPILER_JAM}`;
  execSync(cmd, {
    cwd: projectRoot,
    stdio: 'inherit'
  });

  console.log(`✅ Compiled anan-as compiler to: ${ANAN_AS_COMPILER_JAM}`);
}

function runTestThroughAnanAsInPvm(testName: string, args: string, innerPc?: number, verbose = false): number {
  const jamFile = path.join(projectRoot, 'dist', `${testName}.jam`);

  if (!fs.existsSync(jamFile)) {
    throw new Error(`Test JAM file not found: ${jamFile}`);
  }

  // Read the inner program (SPI format)
  const innerProgram = fs.readFileSync(jamFile);

  // Decode inner args from hex string
  const innerArgs = args ? Buffer.from(args, 'hex') : Buffer.alloc(0);

  // Build the input buffer for the outer interpreter:
  // 8 (gas) + 4 (pc) + 4 (spi-program-len) + 4 (inner-args-len) + program + args
  const gas = BigInt(100_000_000); // 100M gas for inner program
  const pc = innerPc ?? 0;

  const inputBuffer = Buffer.alloc(8 + 4 + 4 + 4 + innerProgram.length + innerArgs.length);
  let offset = 0;

  // Gas (8 bytes, little-endian u64)
  inputBuffer.writeBigUInt64LE(gas, offset);
  offset += 8;

  // PC (4 bytes, little-endian u32)
  inputBuffer.writeUInt32LE(pc, offset);
  offset += 4;

  // Program length (4 bytes)
  inputBuffer.writeUInt32LE(innerProgram.length, offset);
  offset += 4;

  // Inner args length (4 bytes)
  inputBuffer.writeUInt32LE(innerArgs.length, offset);
  offset += 4;

  // Program bytes
  innerProgram.copy(inputBuffer, offset);
  offset += innerProgram.length;

  // Inner args bytes
  innerArgs.copy(inputBuffer, offset);

  // Convert to hex string for CLI
  const inputHex = inputBuffer.toString('hex');

  // Run the outer PVM (compiled anan-as interpreter) with this input
  const ananAsCli = path.join(projectRoot, 'vendor/anan-as/dist/bin/index.js');
  const cmd = `node ${ananAsCli} run --spi --no-metadata --no-logs --gas=10000000000 ${ANAN_AS_COMPILER_JAM} 0x${inputHex}`;

  if (verbose) {
    console.log(`    Input: gas=${gas}, pc=${pc}, prog_len=${innerProgram.length}, args_len=${innerArgs.length}`);
  }

  try {
    if (verbose) {
      console.log(`    Command: ${cmd.substring(0, 200)}...`);
    }
    const output = execSync(cmd, {
      cwd: projectRoot,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer for large outputs
    });

    if (verbose) {
      console.log(`    Raw output: ${output.substring(0, 500)}`);
    }

    // Parse the hex result: Result: [0x...]
    const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]+)\]/);
    if (!resultMatch) {
      throw new Error(`Could not parse result from output: ${output}`);
    }

    const resultHex = resultMatch[1];
    const resultBuffer = Buffer.from(resultHex, 'hex');

    // Parse the output format:
    // 1 (status) + 4 (exitCode) + 8 (gas) + 4 (pc) + ? (result)
    if (resultBuffer.length < 17) {
      throw new Error(`Result too short: ${resultBuffer.length} bytes`);
    }

    const status = resultBuffer.readUInt8(0);
    const exitCode = resultBuffer.readUInt32LE(1);
    const gasLeft = resultBuffer.readBigUInt64LE(5);
    const finalPc = resultBuffer.readUInt32LE(13);
    const innerResult = resultBuffer.subarray(17);

    if (verbose) {
      console.log(`    Output: status=${status}, exitCode=${exitCode}, gasLeft=${gasLeft}, pc=${finalPc}`);
      console.log(`    Inner result (${innerResult.length} bytes): ${innerResult.toString('hex')}`);
    }

    // Check status - 0 = HALT (success)
    if (status !== 0) {
      throw new Error(`Inner program failed with status ${status}, exitCode ${exitCode}`);
    }

    // Parse the inner result as little-endian u32 (first 4 bytes)
    if (innerResult.length < 4) {
      throw new Error(`Inner result too short: ${innerResult.length} bytes`);
    }

    return innerResult.readUInt32LE(0);
  } catch (error: any) {
    if (verbose) {
      if (error.stdout) console.log('    stdout:', error.stdout.toString().substring(0, 500));
      if (error.stderr) console.log('    stderr:', error.stderr.toString().substring(0, 500));
    }
    throw new Error(`Execution failed: ${error.message.split('\n')[0]}`);
  }
}

async function runAllTests(filter?: string, verbose = false): Promise<void> {
  // Filter test cases if requested
  const filteredTestCases = filter ? testCases.filter(tc => tc.name.includes(filter)) : testCases;

  console.log(`Running ${filteredTestCases.length} test cases through PVM-in-PVM execution...`);
  console.log();

  let passed = 0;
  let failed = 0;

  for (const testCase of testCases) {
    if (filter && !testCase.name.includes(filter)) {
      continue;
    }

    console.log(`Testing ${testCase.name}...`);

    for (const test of testCase.tests) {
      try {
        if (verbose) {
          console.log(`  Running: ${test.description}`);
        }

        // Run the test through the compiled anan-as-in-pvm
        const actual = runTestThroughAnanAsInPvm(testCase.name, test.args, test.pc, verbose);

        if (actual === test.expected) {
          console.log(`  ✅ ${test.description}`);
          passed++;
        } else {
          console.log(`  ❌ ${test.description} - expected ${test.expected}, got ${actual}`);
          failed++;
        }

      } catch (error) {
        console.log(`  ❌ ${test.description} - ERROR: ${error.message}`);
        failed++;
      }
    }
    console.log();
  }

  console.log(`Results: ${passed} passed, ${failed} failed`);

  if (passed > 0) {
    console.log();
    console.log('🎉 PVM-in-PVM testing infrastructure is working!');
  }
}

async function main() {
  const args = process.argv.slice(2);
  let filter: string | undefined;
  let verbose = false;

  for (const arg of args) {
    if (arg.startsWith('--filter=')) {
      filter = arg.slice(9);
    } else if (arg === '--verbose' || arg === '-v') {
      verbose = true;
    }
  }

  console.log('=== PVM-in-PVM Test Suite ===\n');

  // Compile anan-as compiler to PVM first
  try {
    compileAnanAsToPvm();
  } catch (error) {
    console.error(`Failed to compile anan-as to PVM: ${error.message}`);
    process.exit(1);
  }

  console.log();

  await runAllTests(filter, verbose);
}

main().catch(console.error);
