import path from "node:path";
import { describe, test, expect } from "bun:test";
import { defineSuite } from "../helpers/suite";
import { JAM_DIR } from "../helpers/paths";
import { runJamWithOutput } from "../helpers/run";

// Tests that console.log import is mapped via adapter WAT to JIP-1 log (ecalli 100).
// The anan-as runner handles ecalli 100 as JIP-1 logging and continues execution.

// Return value test via defineSuite (program returns 42).
defineSuite({
  name: "as-console-log-test",
  tests: [
    { args: "0000000000000000", expected: 42, description: "program returns expected result (42)" },
  ],
});

// Stdout-based assertions can't use defineSuite since it only checks numeric return values.
describe("as-console-log-test (stdout)", () => {
  const jamFile = path.join(JAM_DIR, "as-console-log-test.jam");

  test("console.log mapped to ecalli:100 completes successfully", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    expect(result.stdout).toContain("Exit code: 0");
  });

  test("JIP-1 log message appears in output", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    expect(result.stdout).toMatch(/H.?e.?l.?l.?o/);
  });
});
