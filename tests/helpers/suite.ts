import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "./paths";
import { runJam } from "./run";

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
}

/** Returns all suites that have been defined via defineSuite(). */
export function getRegisteredSuites(): SuiteSpec[] {
  return [...suiteRegistry];
}
