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
  wat: string;
  tests: Array<{
    args: string;
    expected: number;
    description: string;
  }>;
}

const testCases: TestCase[] = [
  {
    name: 'add',
    wat: 'examples-wat/add.jam.wat',
    tests: [
      { args: '0500000007000000', expected: 12, description: '5 + 7 = 12' },
      { args: '00000000ffffffff', expected: 0xffffffff, description: '0 + MAX = MAX' },
      { args: '01000000ffffffff', expected: 0, description: '1 + MAX = 0 (overflow)' },
    ],
  },
  {
    name: 'factorial',
    wat: 'examples-wat/factorial.jam.wat',
    tests: [
      { args: '00000000', expected: 1, description: '0! = 1' },
      { args: '01000000', expected: 1, description: '1! = 1' },
      { args: '05000000', expected: 120, description: '5! = 120' },
      { args: '0a000000', expected: 3628800, description: '10! = 3628800' },
    ],
  },
  {
    name: 'fibonacci',
    wat: 'examples-wat/fibonacci.jam.wat',
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
    wat: 'examples-wat/gcd.jam.wat',
    tests: [
      { args: '3000000012000000', expected: 6, description: 'gcd(48, 18) = 6' },
      { args: '6400000038000000', expected: 4, description: 'gcd(100, 56) = 4' },
      { args: '1100000011000000', expected: 17, description: 'gcd(17, 17) = 17' },
      { args: '01000000ff000000', expected: 1, description: 'gcd(1, 255) = 1' },
    ],
  },
  {
    name: 'is-prime',
    wat: 'examples-wat/is-prime.jam.wat',
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
];

async function main() {
  console.log('=== WASM-PVM Test Suite ===\n');

  let totalTests = 0;
  let passedTests = 0;
  let failedTests = 0;
  const failures: string[] = [];

  for (const testCase of testCases) {
    console.log(`Testing ${testCase.name}...`);
    
    const watPath = path.join(projectRoot, testCase.wat);
    const jamPath = `/tmp/${testCase.name}.jam`;

    try {
      execSync(`cargo run -p wasm-pvm-cli --quiet -- compile "${watPath}" -o "${jamPath}"`, {
        cwd: projectRoot,
        stdio: 'pipe',
      });
    } catch (err) {
      console.log(`  ❌ COMPILE FAILED: ${testCase.wat}`);
      failures.push(`${testCase.name}: compilation failed`);
      failedTests += testCase.tests.length;
      totalTests += testCase.tests.length;
      continue;
    }

    for (const test of testCase.tests) {
      totalTests++;
      
      try {
        const result = execSync(
          `npx tsx scripts/run-jam.ts "${jamPath}" --args=${test.args}`,
          { cwd: projectRoot, stdio: 'pipe', encoding: 'utf-8' }
        );

        let actual: number | null = null;
        
        const u32Match = result.match(/As U32:\s*(\d+)/);
        if (u32Match) {
          actual = parseInt(u32Match[1], 10);
        } else {
          const r11Match = result.match(/r11:\s*(\d+)/);
          if (r11Match) {
            actual = parseInt(r11Match[1], 10);
          }
        }

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
