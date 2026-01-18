#!/usr/bin/env npx tsx
/**
 * Automated test suite for wasm-pvm examples
 * Usage: npx tsx scripts/test-all.ts
 */

import { execSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');

interface TestCase {
  name: string;
  sourceWatOrWasm: string;
  tests: Array<{
    args: string;
    expected: number;
    description: string;
    pc?: number;
  }>;
}

const testCases: TestCase[] = [
  {
    name: 'add',
    sourceWatOrWasm: 'examples-wat/add.jam.wat',
    tests: [
      { args: '0500000007000000', expected: 12, description: '5 + 7 = 12' },
      { args: '00000000ffffffff', expected: 0xffffffff, description: '0 + MAX = MAX' },
      { args: '01000000ffffffff', expected: 0, description: '1 + MAX = 0 (overflow)' },
    ],
  },
  {
    name: 'factorial',
    sourceWatOrWasm: 'examples-wat/factorial.jam.wat',
    tests: [
      { args: '00000000', expected: 1, description: '0! = 1' },
      { args: '01000000', expected: 1, description: '1! = 1' },
      { args: '05000000', expected: 120, description: '5! = 120' },
      { args: '0a000000', expected: 3628800, description: '10! = 3628800' },
    ],
  },
  {
    name: 'fibonacci',
    sourceWatOrWasm: 'examples-wat/fibonacci.jam.wat',
    tests: [
      { args: '00000000', expected: 0, description: 'fib(0) = 0' },
      { args: '01000000', expected: 1, description: 'fib(1) = 1' },
      { args: '02000000', expected: 1, description: 'fib(2) = 1' },
      { args: '0a000000', expected: 55, description: 'fib(10) = 55' },
      { args: '14000000', expected: 6765, description: 'fib(20) = 6765' },
    ],
  },
  {
    name: 'gcd',
    sourceWatOrWasm: 'examples-wat/gcd.jam.wat',
    tests: [
      { args: '3000000012000000', expected: 6, description: 'gcd(48, 18) = 6' },
      { args: '6400000038000000', expected: 4, description: 'gcd(100, 56) = 4' },
      { args: '1100000011000000', expected: 17, description: 'gcd(17, 17) = 17' },
      { args: '01000000ff000000', expected: 1, description: 'gcd(1, 255) = 1' },
    ],
  },
  {
    name: 'is-prime',
    sourceWatOrWasm: 'examples-wat/is-prime.jam.wat',
    tests: [
      { args: '00000000', expected: 0, description: 'is_prime(0) = 0' },
      { args: '01000000', expected: 0, description: 'is_prime(1) = 0' },
      { args: '02000000', expected: 1, description: 'is_prime(2) = 1' },
      { args: '03000000', expected: 1, description: 'is_prime(3) = 1' },
      { args: '04000000', expected: 0, description: 'is_prime(4) = 0' },
      { args: '05000000', expected: 1, description: 'is_prime(5) = 1' },
      { args: '11000000', expected: 1, description: 'is_prime(17) = 1' },
      { args: '19000000', expected: 0, description: 'is_prime(25) = 0' },
      { args: '61000000', expected: 1, description: 'is_prime(97) = 1' },
      { args: '64000000', expected: 0, description: 'is_prime(100) = 0' },
      { args: '65000000', expected: 1, description: 'is_prime(101) = 1' },
    ],
  },
  {
    name: 'div',
    sourceWatOrWasm: 'examples-wat/div.jam.wat',
    tests: [
      { args: '1400000005000000', expected: 4, description: '20 / 5 = 4' },
      { args: '6400000008000000', expected: 12, description: '100 / 8 = 12' },
      { args: '0a00000003000000', expected: 3, description: '10 / 3 = 3' },
    ],
  },
  {
    name: 'call',
    sourceWatOrWasm: 'examples-wat/call.jam.wat',
    tests: [
      { args: '05000000', expected: 10, description: 'double(5) = 10' },
      { args: '0a000000', expected: 20, description: 'double(10) = 20' },
      { args: '00000000', expected: 0, description: 'double(0) = 0' },
    ],
  },
  {
    name: 'i64-ops',
    sourceWatOrWasm: 'examples-wat/i64-ops.jam.wat',
    tests: [
      { args: '00000000', expected: 14, description: 'i64.div_u(100, 7) = 14' },
      { args: '01000000', expected: 2, description: 'i64.rem_u(100, 7) = 2' },
      { args: '02000000', expected: 4080, description: 'i64.shl(0xFF, 4) = 4080' },
      { args: '03000000', expected: 4080, description: 'i64.shr_u(0xFF00, 4) = 4080' },
      { args: '04000000', expected: 240, description: 'i64.and(0xF0F0, 0x0FF0) = 240' },
      { args: '05000000', expected: 65520, description: 'i64.or(0xF0F0, 0x0FF0) = 65520' },
      { args: '06000000', expected: 65280, description: 'i64.xor(0xF0F0, 0x0FF0) = 65280' },
      { args: '07000000', expected: 1, description: 'i64.ge_u(100, 50) = 1' },
      { args: '08000000', expected: 1, description: 'i64.le_u(50, 100) = 1' },
    ],
  },
  {
    name: 'many-locals',
    sourceWatOrWasm: 'examples-wat/many-locals.jam.wat',
    tests: [
      { args: '00000000', expected: 21, description: 'sum with base 0: 1+2+3+4+5+6 = 21' },
      { args: '0a000000', expected: 81, description: 'sum with base 10: 11+12+13+14+15+16 = 81' },
      { args: '64000000', expected: 621, description: 'sum with base 100: 101+102+103+104+105+106 = 621' },
    ],
  },
  {
    name: 'entry-points',
    sourceWatOrWasm: 'examples-wat/entry-points.jam.wat',
    tests: [
      { args: '', expected: 42, description: 'main (PC=0) returns 42' },
      { args: '', expected: 99, description: 'main2 (PC=5) returns 99', pc: 5 },
    ],
  },
  {
    name: 'as-add',
    sourceWatOrWasm: 'examples-as/build/add.wasm',
    tests: [
      { args: '0500000007000000', expected: 12, description: 'AS: 5 + 7 = 12' },
      { args: '0a00000014000000', expected: 30, description: 'AS: 10 + 20 = 30' },
    ],
  },
  {
    name: 'as-factorial',
    sourceWatOrWasm: 'examples-as/build/factorial.wasm',
    tests: [
      { args: '00000000', expected: 1, description: 'AS: 0! = 1' },
      { args: '05000000', expected: 120, description: 'AS: 5! = 120' },
      { args: '07000000', expected: 5040, description: 'AS: 7! = 5040' },
    ],
  },
  {
    name: 'as-fibonacci',
    sourceWatOrWasm: 'examples-as/build/fibonacci.wasm',
    tests: [
      { args: '00000000', expected: 0, description: 'AS: fib(0) = 0' },
      { args: '01000000', expected: 1, description: 'AS: fib(1) = 1' },
      { args: '0a000000', expected: 55, description: 'AS: fib(10) = 55' },
    ],
  },
  {
    name: 'as-gcd',
    sourceWatOrWasm: 'examples-as/build/gcd.wasm',
    tests: [
      { args: '3000000012000000', expected: 6, description: 'AS: gcd(48, 18) = 6' },
      { args: '6400000038000000', expected: 4, description: 'AS: gcd(100, 56) = 4' },
      { args: '1100000011000000', expected: 17, description: 'AS: gcd(17, 17) = 17' },
    ],
  },
];

async function main() {
  console.log('=== WASM-PVM Test Suite ===\n');

  console.log('Building AssemblyScript examples...');
  try {
    execSync('npm run build', {
      cwd: path.join(projectRoot, 'examples-as'),
      stdio: 'pipe',
    });
    console.log('AssemblyScript build complete.\n');
  } catch (err) {
    console.error('Failed to build AssemblyScript examples. Run: cd examples-as && npm install');
    process.exit(1);
  }

  let totalTests = 0;
  let passedTests = 0;
  let failedTests = 0;
  const failures: string[] = [];

  for (const testCase of testCases) {
    console.log(`Testing ${testCase.name}...`);
    
    const sourcePath = path.join(projectRoot, testCase.sourceWatOrWasm);
    const jamPath = `/tmp/${testCase.name}.jam`;

    try {
      execSync(`cargo run -p wasm-pvm-cli --quiet -- compile "${sourcePath}" -o "${jamPath}"`, {
        cwd: projectRoot,
        stdio: 'pipe',
      });
    } catch (err) {
      console.log(`  ❌ COMPILE FAILED: ${testCase.sourceWatOrWasm}`);
      failures.push(`${testCase.name}: compilation failed`);
      failedTests += testCase.tests.length;
      totalTests += testCase.tests.length;
      continue;
    }

    for (const test of testCase.tests) {
      totalTests++;
      
      try {
        const pcArg = test.pc !== undefined ? ` --pc=${test.pc}` : '';
        const result = execSync(
          `npx tsx scripts/run-jam.ts "${jamPath}" --args=${test.args}${pcArg}`,
          { cwd: projectRoot, stdio: 'pipe', encoding: 'utf-8' }
        );

        const u32Match = result.match(/As U32:\s*(\d+)/);
        const actual = u32Match ? parseInt(u32Match[1], 10) : null;

        if (actual === null) {
          console.log(`  ❌ ${test.description} - Could not parse result`);
          failures.push(`${testCase.name}: ${test.description} - no result in output`);
          failedTests++;
          continue;
        }

        if (actual === test.expected) {
          console.log(`  ✓ ${test.description}`);
          passedTests++;
        } else {
          console.log(`  ❌ ${test.description} - expected ${test.expected}, got ${actual}`);
          failures.push(`${testCase.name}: ${test.description} - expected ${test.expected}, got ${actual}`);
          failedTests++;
        }
      } catch (err: any) {
        console.log(`  ❌ ${test.description} - execution failed`);
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
