import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "./paths";
import { runJam } from "./run";
import { runJamPvmInPvm } from "./pvm-in-pvm";
import { runWasmForSuite } from "./wasm-runner";

export interface TestSpec {
  args: string;
  expected: number;
  description: string;
  pc?: number;
}

export interface SuiteSpec {
  name: string;
  tests: TestSpec[];
  /** Skip pvm-in-pvm variants (e.g. tests using unhandled ecalli host calls). */
  skipPvmInPvm?: boolean;
  /** Skip differential testing (e.g. modules with PVM-specific imports). */
  skipDifferential?: boolean;
  /** Custom timeout in ms for normal (non-pvm-in-pvm) tests. */
  timeout?: number;
}

/** Global registry of all defined suites (populated at import time). */
const suiteRegistry: SuiteSpec[] = [];

/** Bun test timeout for pvm-in-pvm tests. */
const PVM_IN_PVM_TEST_TIMEOUT = 300_000;

/** Register a suite and create normal (non-pvm-in-pvm) test cases. */
export function defineSuite(suite: SuiteSpec) {
  suiteRegistry.push(suite);

  const jamFile = path.join(JAM_DIR, `${suite.name}.jam`);
  describe(suite.name, () => {
    for (const t of suite.tests) {
      test(
        t.description,
        () => {
          const actual = runJam(jamFile, t.args, t.pc);
          expect(actual).toBe(t.expected);
        },
        suite.timeout,
      );
    }
  });
}

/** Create pvm-in-pvm test variants for a suite. Used by layer5. */
export function definePvmInPvmSuite(suite: SuiteSpec) {
  if (suite.skipPvmInPvm) return;

  const jamFile = path.join(JAM_DIR, `${suite.name}.jam`);
  describe(`pvm-in-pvm: ${suite.name}`, () => {
    for (const t of suite.tests) {
      test(
        t.description,
        () => {
          const actual = runJamPvmInPvm(jamFile, t.args, t.pc);
          expect(actual).toBe(t.expected);
        },
        PVM_IN_PVM_TEST_TIMEOUT,
      );
    }
  });
}

/**
 * Create differential test variants for a suite.
 *
 * Runs each test case through both PVM (via runJam) and native WASM
 * (via Bun's WebAssembly engine) and asserts the results match.
 */
export function defineDifferentialSuite(suite: SuiteSpec) {
  if (suite.skipDifferential) return;

  const jamFile = path.join(JAM_DIR, `${suite.name}.jam`);
  describe(`differential: ${suite.name}`, () => {
    for (const t of suite.tests) {
      test(t.description, async () => {
        // Skip tests with custom PC — PVM-specific entry points
        // have no equivalent in native WASM execution.
        if (t.pc !== undefined) return;

        const wasmResult = await runWasmForSuite(suite.name, t.args);
        if (wasmResult === null) {
          // Module has imports or WASM not found — skip silently
          return;
        }

        // Run through PVM
        let pvmResult: number;
        let pvmTrapped = false;
        try {
          pvmResult = runJam(jamFile, t.args, t.pc);
        } catch {
          pvmTrapped = true;
          pvmResult = -1;
        }

        if (wasmResult.trapped) {
          // Both should trap
          expect(pvmTrapped).toBe(true);
        } else {
          // Neither should trap, and values should match
          expect(pvmTrapped).toBe(false);
          expect(pvmResult!).toBe(wasmResult.value!);
        }
      });
    }
  });
}

/** Returns all suites that have been defined via defineSuite(). */
export function getRegisteredSuites(): SuiteSpec[] {
  return [...suiteRegistry];
}
