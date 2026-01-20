#!/usr/bin/env npx tsx
/**
 * PVM-in-PVM Test Harness
 *
 * Runs all existing test cases through compiled anan-as running inside regular anan-as.
 *
 * Usage: npx tsx scripts/test-pvm-in-pvm.ts [--filter=pattern] [--verbose]
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../vendor/anan-as/build/release.js');

// The compiled anan-as with main() entry point
const OUTER_PVM_PATH = '/tmp/anan-as-pvm.jam';
if (!fs.existsSync(OUTER_PVM_PATH)) {
  console.error(`Error: Compiled anan-as PVM not found at ${OUTER_PVM_PATH}`);
  console.error('Run: cargo run -p wasm-pvm-cli -- compile vendor/anan-as/build/release-stub.wasm -o /tmp/anan-as-pvm.jam');
  process.exit(1);
}

const ananAsPvmData = fs.readFileSync(OUTER_PVM_PATH);

interface TestCase {
  name: string;
  spiFile: string;
  inputArgs: number[];
  expectedOutput?: number;
  description?: string;
}

interface PvmInPvmResult {
  status: number;
  pc: number;
  gasLeft: bigint;
  registers: bigint[];
  resultAddr: number;
  resultLen: number;
  resultBytes: number[];
  resultValue?: number;
}



async function runPvmInPvm(testSpiFile: string, inputArgs: number[] = [], gas: bigint = BigInt(10_000_000)): Promise<PvmInPvmResult> {
  const testSpiData = fs.readFileSync(testSpiFile);

  // Prepare input for compiled anan-as main() function
  // Format: program_len (4) + program (SPI) + gas (8) + steps (4) + inner_args
  const programLen = testSpiData.length;
  const steps = 1000000; // Max steps

  // Build input buffer
  const inputBuffer = new ArrayBuffer(4 + programLen + 8 + 4 + inputArgs.length);
  const view = new DataView(inputBuffer);
  const bytes = new Uint8Array(inputBuffer);
  let offset = 0;

  // program_len (u32)
  view.setUint32(offset, programLen, true);
  offset += 4;

  // program bytes (SPI format)
  bytes.set(testSpiData, offset);
  offset += programLen;

  // gas (u64)
  view.setBigUint64(offset, gas, true);
  offset += 8;

  // steps (u32)
  view.setUint32(offset, steps, true);
  offset += 4;

  // inner program args
  for (let i = 0; i < inputArgs.length; i++) {
    bytes[offset + i] = inputArgs[i];
  }

  // Convert to byte array
  const inputBytes = Array.from(bytes);

  // Import anan-as
  const ananAs = await import(ananAsPath);

  // Prepare the compiled anan-as PVM program
  const outerProgram = ananAs.prepareProgram(
    ananAs.InputKind.SPI,
    ananAs.HasMetadata.No,
    Array.from(ananAsPvmData),
    [],
    [],
    [],
    inputBytes // Pass input data as SPI args
  );

  // Run the compiled anan-as inside regular anan-as
  const output = ananAs.runProgram(outerProgram, BigInt(100_000_000), 0, false);

  if (output.status !== 0) {
    throw new Error(`Compiled anan-as failed with status ${output.status}, exit code ${output.exitCode}`);
  }

  // Extract results from compiled anan-as output
  // Format: status(1) + pc(4) + gas_left(8) + registers(104) = 117 bytes
  const resultAddr = Number(output.registers[7]); // result_ptr from SPI
  const resultLen = Number(output.registers[8]); // result_len from SPI

  if (resultLen !== 117) {
    throw new Error(`Unexpected result length: ${resultLen}, expected 117`);
  }

  const resultBytes = readMemoryFromChunks(output.memory || [], resultAddr, resultLen);
  const resultView = new DataView(new Uint8Array(resultBytes).buffer);

  let resultOffset = 0;
  const status = resultView.getUint8(resultOffset);
  resultOffset += 1;

  const pc = resultView.getUint32(resultOffset, true);
  resultOffset += 4;

  const gasLeft = resultView.getBigUint64(resultOffset, true);
  resultOffset += 8;

  const finalRegisters: bigint[] = [];
  for (let i = 0; i < 13; i++) {
    finalRegisters.push(resultView.getBigUint64(resultOffset, true));
    resultOffset += 8;
  }

  return {
    status,
    pc,
    gasLeft,
    registers: finalRegisters,
    resultAddr,
    resultLen: resultLen,
    resultBytes,
    resultValue: resultLen === 4 ? new DataView(new Uint8Array(resultBytes.slice(0, 4)).buffer).getUint32(0, true) : undefined
  };
}

async function runAllTests(filter?: string, verbose = false): Promise<void> {
  // Define all test cases based on existing examples
  const testCases: TestCase[] = [
    { name: 'add', spiFile: 'examples-as/build/add.wasm', inputArgs: [5, 7], expectedOutput: 12 },
    { name: 'factorial', spiFile: 'examples-as/build/factorial.wasm', inputArgs: [5], expectedOutput: 120 },
    { name: 'fibonacci', spiFile: 'examples-as/build/fibonacci.wasm', inputArgs: [10], expectedOutput: 55 },
    { name: 'gcd', spiFile: 'examples-as/build/gcd.wasm', inputArgs: [48, 18], expectedOutput: 6 },
    { name: 'life', spiFile: 'examples-as/build/life.wasm', inputArgs: [5] }, // Game of Life with 5 steps
  ];

  // Add WAT examples
  const watExamples = [
    'add.jam', 'factorial.jam', 'fibonacci.jam', 'gcd.jam', 'is-prime.jam',
    'div.jam', 'call.jam', 'br-table.jam', 'bit-ops.jam', 'rotate.jam',
    'entry-points.jam', 'recursive.jam', 'nested-calls.jam', 'call-indirect.jam',
    'i64-ops.jam', 'many-locals.jam', 'block-result.jam', 'block-br-test.jam'
  ];

  for (const watFile of watExamples) {
    const spiFile = `examples-wat/build/${watFile}.spi`;
    if (fs.existsSync(spiFile)) {
      // Most tests take small inputs, add specific cases as needed
      let inputArgs: number[] = [];
      let expectedOutput: number | undefined;

      if (watFile === 'add.jam') {
        inputArgs = [5, 7];
        expectedOutput = 12;
      } else if (watFile === 'factorial.jam') {
        inputArgs = [5];
        expectedOutput = 120;
      } else if (watFile === 'fibonacci.jam') {
        inputArgs = [10];
        expectedOutput = 55;
      } else if (watFile === 'gcd.jam') {
        inputArgs = [48, 18];
        expectedOutput = 6;
      }

      testCases.push({
        name: watFile.replace('.jam', ''),
        spiFile,
        inputArgs,
        expectedOutput
      });
    }
  }

  // Filter tests if requested
  const filteredTests = filter ? testCases.filter(tc => tc.name.includes(filter)) : testCases;

  console.log(`Running ${filteredTests.length} PVM-in-PVM tests...`);
  console.log();

  let passed = 0;
  let failed = 0;

  for (const testCase of filteredTests) {
    try {
      if (verbose) {
        console.log(`Running ${testCase.name}...`);
      }

      const result = await runPvmInPvm(testCase.spiFile, testCase.inputArgs);

      if (verbose) {
        console.log(`  Status: ${statusToString(result.status)}`);
        console.log(`  PC: ${result.pc}`);
        console.log(`  Gas Left: ${result.gasLeft}`);
        if (result.resultValue !== undefined) {
          console.log(`  Result: ${result.resultValue}`);
        }
      }

      let testPassed = true;
      if (result.status !== 0) {
        console.log(`❌ ${testCase.name}: Failed with status ${statusToString(result.status)}`);
        testPassed = false;
      } else if (testCase.expectedOutput !== undefined && result.resultValue !== testCase.expectedOutput) {
        console.log(`❌ ${testCase.name}: Expected ${testCase.expectedOutput}, got ${result.resultValue}`);
        testPassed = false;
      } else {
        console.log(`✅ ${testCase.name}: PASSED`);
      }

      if (testPassed) {
        passed++;
      } else {
        failed++;
      }

    } catch (error) {
      console.log(`❌ ${testCase.name}: ERROR - ${error.message}`);
      failed++;
    }
  }

  console.log();
  console.log(`Results: ${passed} passed, ${failed} failed`);
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

  if (!fs.existsSync(ananAsPath)) {
    console.error('Error: anan-as not built. Run: cd vendor/anan-as && npm ci && npm run build');
    process.exit(1);
  }

  await runAllTests(filter, verbose);
}

function readMemoryFromChunks(chunks: any[], addr: number, len: number): number[] {
  const result: number[] = [];
  for (let i = 0; i < len; i++) {
    const byteAddr = addr + i;
    const chunkIndex = Math.floor(byteAddr / 65536);
    const chunkOffset = byteAddr % 65536;

    if (chunkIndex >= chunks.length || !chunks[chunkIndex]) {
      result.push(0);
    } else {
      result.push(chunks[chunkIndex][chunkOffset] || 0);
    }
  }
  return result;
}

function statusToString(status: number): string {
  switch (status) {
    case 0: return 'HALT';
    case 1: return 'PANIC';
    case 2: return 'OUT_OF_GAS';
    case 3: return 'PAGE_FAULT';
    case 4: return 'HOST_CALL';
    default: return `UNKNOWN(${status})`;
  }
}

main().catch(console.error);
