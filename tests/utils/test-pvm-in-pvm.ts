#!/usr/bin/env bun
/**
 * PVM-in-PVM Test Harness
 *
 * Compiles anan-as compiler to PVM, then runs test cases through the compiled anan-as-in-pvm.
 *
 * Discovers test suites by dynamically importing all *.test.ts files from the
 * layer directories and collecting suites registered via defineSuite().
 *
 * Usage: bun tests/utils/test-pvm-in-pvm.ts [--filter=pattern] [--verbose]
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'child_process';
import { Glob } from 'bun';
import type { SuiteSpec } from '../helpers/suite';
import { getRegisteredSuites } from '../helpers/suite';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '../..');
const testsDir = path.join(__dirname, '..');

// Paths for compiled artifacts
const ANAN_AS_COMPILER_WASM = path.join(projectRoot, 'vendor/anan-as/dist/build/compiler.wasm');
const ANAN_AS_COMPILER_JAM = '/tmp/anan-as-compiler.jam';

function compileAnanAsToPvm(): void {
  console.log('Compiling anan-as compiler from WASM to PVM...');

  if (!fs.existsSync(ANAN_AS_COMPILER_WASM)) {
    throw new Error(`anan-as compiler WASM not found at ${ANAN_AS_COMPILER_WASM}. Run: cd vendor/anan-as && npm ci && npm run build`);
  }

  const cmd = `cargo run -p wasm-pvm-cli -- compile ${ANAN_AS_COMPILER_WASM} --output ${ANAN_AS_COMPILER_JAM}`;
  execSync(cmd, {
    cwd: projectRoot,
    stdio: 'inherit'
  });

  console.log(`Compiled anan-as compiler to: ${ANAN_AS_COMPILER_JAM}`);
}

async function discoverSuites(): Promise<SuiteSpec[]> {
  const glob = new Glob('layer*/*.test.ts');
  const testFiles: string[] = [];
  for await (const file of glob.scan({ cwd: testsDir })) {
    testFiles.push(path.join(testsDir, file));
  }
  testFiles.sort();

  // Import all test files to trigger defineSuite() registration
  for (const file of testFiles) {
    await import(file);
  }

  return getRegisteredSuites();
}

/**
 * Strip the metadata prefix from a JAM/SPI buffer.
 * Format: varint(metadata_len) + metadata_bytes + raw_spi
 */
function stripMetadata(buf: Buffer): Buffer {
  const firstByte = buf[0];
  let offset: number;
  let metadataLen: number;
  if (firstByte < 128) {
    metadataLen = firstByte;
    offset = 1;
  } else {
    const leadingOnes = Math.clz32(~(firstByte << 24));
    const mask = (1 << (8 - leadingOnes)) - 1;
    metadataLen = firstByte & mask;
    for (let i = 1; i <= leadingOnes; i++) {
      metadataLen = metadataLen * 256 + buf[i];
    }
    offset = 1 + leadingOnes;
  }
  return buf.subarray(offset + metadataLen);
}

function runTestThroughAnanAsInPvm(testName: string, args: string, innerPc?: number, verbose = false): number {
  const jamDir = path.join(__dirname, '../build/jam');
  const jamFile = path.join(jamDir, `${testName}.jam`);

  if (!fs.existsSync(jamFile)) {
    throw new Error(`Test JAM file not found: ${jamFile}`);
  }

  // Strip metadata prefix: inner programs are passed as raw SPI to the anan-as
  // interpreter running inside PVM, which expects raw SPI format.
  const innerProgram = stripMetadata(fs.readFileSync(jamFile));
  const innerArgs = args ? Buffer.from(args, 'hex') : Buffer.alloc(0);

  const gas = BigInt(100_000_000);
  const pc = innerPc ?? 0;

  const inputBuffer = Buffer.alloc(8 + 4 + 4 + 4 + innerProgram.length + innerArgs.length);
  let offset = 0;

  inputBuffer.writeBigUInt64LE(gas, offset);
  offset += 8;

  inputBuffer.writeUInt32LE(pc, offset);
  offset += 4;

  inputBuffer.writeUInt32LE(innerProgram.length, offset);
  offset += 4;

  inputBuffer.writeUInt32LE(innerArgs.length, offset);
  offset += 4;

  innerProgram.copy(inputBuffer, offset);
  offset += innerProgram.length;

  innerArgs.copy(inputBuffer, offset);

  const inputHex = inputBuffer.toString('hex');

  const ananAsCli = path.join(projectRoot, 'vendor/anan-as/dist/bin/index.js');
  const cmd = `node ${ananAsCli} run --spi --no-logs --gas=10000000000 ${ANAN_AS_COMPILER_JAM} 0x${inputHex}`;

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
      maxBuffer: 10 * 1024 * 1024,
    });

    if (verbose) {
      console.log(`    Raw output: ${output.substring(0, 500)}`);
    }

    const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]+)\]/);
    if (!resultMatch) {
      throw new Error(`Could not parse result from output: ${output}`);
    }

    const resultHex = resultMatch[1];
    const resultBuffer = Buffer.from(resultHex, 'hex');

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

    if (status !== 0) {
      throw new Error(`Inner program failed with status ${status}, exitCode ${exitCode}`);
    }

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
  const testCases = await discoverSuites();
  const filteredTestCases = filter ? testCases.filter(tc => tc.name.includes(filter)) : testCases;

  console.log(`Running ${filteredTestCases.length} test cases through PVM-in-PVM execution...`);
  console.log();

  let passed = 0;
  let failed = 0;

  for (const testCase of filteredTestCases) {
    console.log(`Testing ${testCase.name}...`);

    for (const t of testCase.tests) {
      try {
        if (verbose) {
          console.log(`  Running: ${t.description}`);
        }

        const actual = runTestThroughAnanAsInPvm(testCase.name, t.args, t.pc, verbose);

        if (actual === t.expected) {
          console.log(`  PASS ${t.description}`);
          passed++;
        } else {
          console.log(`  FAIL ${t.description} - expected ${t.expected}, got ${actual}`);
          failed++;
        }

      } catch (error: any) {
        console.log(`  FAIL ${t.description} - ERROR: ${error.message}`);
        failed++;
      }
    }
    console.log();
  }

  console.log(`Results: ${passed} passed, ${failed} failed`);

  if (passed > 0) {
    console.log();
    console.log('PVM-in-PVM testing infrastructure is working!');
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

  try {
    compileAnanAsToPvm();
  } catch (error: any) {
    console.error(`Failed to compile anan-as to PVM: ${error.message}`);
    process.exit(1);
  }

  console.log();

  await runAllTests(filter, verbose);
}

main().catch(console.error);
