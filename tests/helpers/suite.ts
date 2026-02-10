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

export function defineSuite(suite: SuiteSpec) {
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
