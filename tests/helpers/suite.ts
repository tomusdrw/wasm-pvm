import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "./paths";
import { runJam } from "./run";
import { runJamPvmInPvm } from "./pvm-in-pvm";

export interface TestSpec {
  args: string;
  expected: number;
  description: string;
  pc?: number;
}

export interface SuiteSpec {
  name: string;
  tests: TestSpec[];
}

/** Global registry of all defined suites (populated at import time). */
const suiteRegistry: SuiteSpec[] = [];

/** PVM-in-PVM timeout: these tests are slow (~30-120s each). */
const PVM_IN_PVM_TIMEOUT = 300_000;

export function defineSuite(suite: SuiteSpec) {
  suiteRegistry.push(suite);

  const jamFile = path.join(JAM_DIR, `${suite.name}.jam`);
  describe(suite.name, () => {
    for (const t of suite.tests) {
      test(t.description, () => {
        const actual = runJam(jamFile, t.args, t.pc);
        expect(actual).toBe(t.expected);
      });
    }
  });

  // Also register pvm-in-pvm variants
  describe(`pvm-in-pvm: ${suite.name}`, () => {
    for (const t of suite.tests) {
      test(
        t.description,
        () => {
          const actual = runJamPvmInPvm(jamFile, t.args, t.pc);
          expect(actual).toBe(t.expected);
        },
        PVM_IN_PVM_TIMEOUT,
      );
    }
  });
}

/** Returns all suites that have been defined via defineSuite(). */
export function getRegisteredSuites(): SuiteSpec[] {
  return [...suiteRegistry];
}
